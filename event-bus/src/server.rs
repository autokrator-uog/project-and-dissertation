use futures::{Future, Sink, Stream};
use futures_cpupool::CpuPool;
use tokio_core::reactor::{Handle, Core};

use producer;
use consumer;

use std::str::from_utf8;
use std::rc::Rc;
use std::fmt::Debug;

use rdkafka::Message;

use websocket::message::OwnedMessage;
use websocket::async::Server;
use websocket::server::InvalidConnection;

use state::ServerState;

pub fn bootstrap(bind: &str, brokers: &str, group: &str, topic: &str) {
    // Create our event loop.
    let mut core = Core::new().expect("Failed to create Tokio event loop");

    // Create a handle for spawning futures from the main thread.
    let handle = core.handle();
    let remote = core.remote();

    // Create all of the required state.
    let ServerState {state, receive_channel_in, send_channel_in, consumer } =
        ServerState::new(brokers, group, topic);

    // Create the websocket server and add it on the event loop.
    let server = Server::bind(bind, &handle).expect("Failed to create websocket server");
    info!("Running websocket server on {:?}", bind);

    // Create a (single-threaded) reference-counted CPU pool.
    let cpu_pool = Rc::new(CpuPool::new_num_cpus());

    // We must clone the state into state_inner so that when it is captured by the
    // move closure, we still have the state for use outside. We will see this pattern
    // throughout.
    let state_inner = state.clone();
    let connection_handler = server.incoming()
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            let state = state_inner.clone();
            let handle_inner = handle.clone();

            // Accept the websocket connection.
            let f = upgrade.accept().and_then(move |(framed, _)| {
                // Split the channels up into send (sink) and receive (stream).
                let (sink, stream) = framed.split();
                // Add the (addr, stream) mapping to the receive_channel_out. We'll
                // be able to loop over receive_channel_in to spawn threads that will
                // handle the incoming messages.
                info!("Accepted connection from {:?}", addr);
                let f = state.receive_channel_out.send((format!("{:?}", addr), stream));
                spawn_future(f, "Send stream to connection pool", &handle_inner);

                // Add the sink to the HashMap so that we can get it when we want
                // to send a message to the client.
                state.connections.write().unwrap().insert(format!("{:?}", addr), sink);
                Ok(())
            });

            spawn_future(f, "Handle new connection", &handle);
            Ok(())
        })
        .map_err(|_| {});

    let state_inner = state.clone();
    let remote_inner = remote.clone();
    // Spawn a handler for outgoing clients so that this doesn't block the main thread.
    let receive_handler = cpu_pool.spawn_fn(|| {
        receive_channel_in.for_each(move |(addr, stream)| {
            let state = state_inner.clone();
            let remote = remote_inner.clone();

            // For each client, spawn a new thread that will process everything we receive from
            // it.
            remote.spawn(move |handle| {
                let state = state.clone();
                let remote = handle.remote().clone();
                let addr = addr.clone();

                stream.for_each(move |msg| {
                    let state = state.clone();
                    let remote = remote.clone();
                    let addr = addr.clone();

                    if let OwnedMessage::Close(_) = msg {
                        info!("Removing connection {:?} from clients", &addr);
                        state.connections.write().unwrap().remove(&addr);
                    } else {
                        remote.spawn(move |_| {
                            let parsed_message = producer::parse_message(msg);
                            parsed_message.process(addr, state.producer, state.topic);
                            Ok(())
                        });
                    }

                    Ok(())
                }).map_err(|_| ())
            });
            Ok(())
        })
    });

    let state_inner = state.clone();
    let send_handler = cpu_pool.spawn_fn(move || {
        let state = state_inner.clone();

        // send_channel_in contains the messages that we're queuing up to
        // send to the client at addr.
        send_channel_in.and_then({
            let state = state.clone();
            move |(addr, msg): (String, String)| {
                let sink = state.connections.write().unwrap().remove(&addr)
                    .expect("Tried to send to invalid client address.");

                info!("Sending {:?} to {:?}", msg, addr);
                sink.send(OwnedMessage::Text(msg))
                    .map(move |sink| (addr, sink))
                    .map_err(|_| ())
            }
        }).for_each(move |(addr, sink)| {
            state.connections.write().unwrap().insert(addr, sink);
            Ok(())
        })
    }).map_err(|_| ());

    let state_inner = state.clone();
    let remote_inner = remote.clone();
    // Start watching for messages in subscribed topics.
    let consumer_handler = consumer.start().filter_map(|result| {
        // Discard any errors from the messages.
        match result {
            Ok(msg) => Some(msg),
            Err(kafka_error) => {
                warn!("Error while receiving from Kafka {:?}", kafka_error);
                None
            }
        }
    }).for_each(move |msg| {
        // Take ownership of the message and convert the &[u8] to a &str.
        let owned_message = msg.detach();
        let message_as_string = from_utf8(&owned_message.payload().unwrap()).unwrap();

        // Parse the message.
        let parsed_message = consumer::parse_message(message_as_string.to_string());
        let state = state_inner.clone();
        let remote = remote_inner.clone();

        // Spawn a thread to send it to all clients.
        remote.spawn(move |handle| {
            let state = state.clone();
            let parsed_message_inner = parsed_message.clone();

            // Loop over the clients.
            for (addr, _) in state.connections.read().unwrap().iter() {
                let state = state.clone();
                let parsed_message_inner = parsed_message_inner.clone();

                // If we decide to send the message, add it to the outgoing queue.
                if let Some(processed_message) = consumer::process_event(
                        parsed_message_inner, addr) {
                    let f = state.send_channel_out.send((addr.to_string(), processed_message));
                    spawn_future(f, "Send message to write handler", handle);
                }
            }

            Ok(())
        });

        Ok(())
    });

    let handlers = connection_handler.select2(receive_handler.select2(send_handler.select2(consumer_handler)));
    core.run(handlers).map_err(|_| error!("Error while running core loop")).unwrap();
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
    where F: Future<Item = I, Error = E> + 'static, E: Debug
{
    handle.spawn(f.map_err(move |e| error!("Error in {}: '{:?}'", desc, e))
                 .map(move |_| info!("{}: Finished.", desc)));
}
