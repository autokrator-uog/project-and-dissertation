use futures::{Future, Sink, Stream};
use futures_cpupool::CpuPool;
use tokio_core::reactor::{Handle, Core};

use producer;

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

    let (state, receive_channel_in, send_channel_in, consumer) = ServerState::create(
        &core, brokers, group, topic);

    // Create a remote for spawning futures from outside the main thread.
    let remote = state.remote;

    // Create the websocket server and add it on the event loop.
    let server = Server::bind(bind, &handle).expect("Failed to create websocket server");
    info!("Running websocket server on {:?}", bind);

    // Create a (single-threaded) reference-counted CPU pool.
    let cpu_pool = Rc::new(CpuPool::new_num_cpus());
    // Create a thread-safe reference counted, hashmap with a read-write lock. Used to contain the
    // thread id to output sink mappings.
    let connections = state.connections;

    // Multiple producer, single-consumer FIFO queue. Messages added to receive_channel_out will
    // appear in receive_channel_in.
    let receive_channel_out = state.receive_channel_out;

    let topic = state.topic;

    // We must clone the connections into connections_inner so that when it is captured by the
    // move closure, we still have the connections for use outside. We will see this pattern
    // throughout.
    let connections_inner = connections.clone();
    let connection_handler = server.incoming()
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            let connections_inner = connections_inner.clone();
            let handle_inner = handle.clone();
            let channel = receive_channel_out.clone();

            // Accept the websocket connection.
            let f = upgrade.accept().and_then(move |(framed, _)| {
                // Split the channels up into send (sink) and receive (stream).
                let (sink, stream) = framed.split();
                // Add the (addr, stream) mapping to the receive_channel_out. We'll
                // be able to loop over receive_channel_in to spawn threads that will
                // handle the incoming messages.
                info!("Accepted connection from {:?}", addr);
                let f = channel.send((format!("{:?}", addr), stream));
                spawn_future(f, "Send stream to connection pool", &handle_inner);

                // Add the sink to the HashMap so that we can get it when we want
                // to send a message to the client.
                connections_inner.write().unwrap().insert(format!("{:?}", addr), sink);
                Ok(())
            });

            spawn_future(f, "Handle new connection", &handle);
            Ok(())
        })
        .map_err(|_| {});

    // Create a Kafka producer for use when sending messages from websocket clients.
    let producer = state.producer;

    let connections_inner = connections.clone();
    let remote_inner = remote.clone();
    let topic_inner = topic.to_string();
    let producer_inner = producer.clone();

    // Spawn a handler for outgoing clients so that this doesn't block the main thread.
    let receive_handler = cpu_pool.spawn_fn(|| {
        receive_channel_in.for_each(move |(addr, stream)| {
            let connections_inner = connections_inner.clone();
            let remote_inner = remote_inner.clone();
            let topic_inner = topic_inner.clone();
            let producer_inner = producer_inner.clone();

            // For each client, spawn a new thread that will process everything we receive from
            // it.
            remote_inner.spawn(move |handle| {
                let connections_inner = connections_inner.clone();
                let topic_inner = topic_inner.clone();
                let producer_inner = producer_inner.clone();
                let addr_inner = addr.clone();
                let remote_inner = handle.remote().clone();

                stream.for_each(move |msg| {
                    let connections_inner = connections_inner.clone();
                    let remote_inner = remote_inner.clone();
                    let topic_inner = topic_inner.clone();
                    let producer_inner = producer_inner.clone();
                    let addr_inner = addr_inner.clone();

                    // Process the message we just got by forwarding on to Kafka.
                    let converted = match msg {
                        OwnedMessage::Binary(bytes) => Some(from_utf8(&bytes).unwrap().to_string()),
                        OwnedMessage::Text(message) => Some(message),
                        OwnedMessage::Close(_) => {
                            info!("Removing connection {:?} from clients", &addr);
                            connections_inner.write().unwrap().remove(&addr);
                            None
                        },
                        _ => None
                    };

                    if let Some(message_as_string) = converted {
                        let message = producer::process_incoming(addr_inner, message_as_string);

                        remote_inner.spawn(move |_| {
                            info!("Sending {:?} to topic {:?}", topic_inner, message);
                            producer_inner.send_copy::<String, ()>(
                                &topic_inner, None, Some(&message.unwrap()),
                                None, None, 1000);
                            Ok(())
                        });
                    }
                    Ok(())
                }).map_err(|_| ())
            });
            Ok(())
        })
    });

    let send_channel_out = state.send_channel_out;

    let connections_inner = connections.clone();
    let send_handler = cpu_pool.spawn_fn(move || {
        let connections = connections_inner.clone();
        // send_channel_in contains the messages that we're queuing up to
        // send to the client at addr.
        send_channel_in.and_then({
            let connections = connections.clone();
            move |(addr, msg): (String, String)| {
                let sink = connections.write().unwrap().remove(&addr)
                    .expect("Tried to send to invalid client address.");

                info!("Sending {:?} to {:?}", msg, addr);
                sink.send(OwnedMessage::Text(msg))
                    .map(move |sink| (addr, sink))
                    .map_err(|_| ())
            }
        }).for_each(move |(addr, sink)| {
            connections.write().unwrap().insert(addr, sink);
            Ok(())
        })
    }).map_err(|_| ());

    let connections_inner = connections.clone();
    let remote_inner = remote.clone();
    let channel = send_channel_out.clone();

    let consumer_handler = consumer.start().filter_map(|result| {
        match result {
            Ok(msg) => Some(msg),
            Err(kafka_error) => {
                warn!("Error while receiving from Kafka {:?}", kafka_error);
                None
            }
        }
    }).for_each(move |msg| {
        let owned_message = msg.detach();
        let message_as_string = from_utf8(&owned_message.payload().unwrap()).unwrap();

        if let Ok(parsed_message) = producer::parse_message(message_as_string.to_string()) {
            let connections_inner = connections_inner.clone();
            let channel = channel.clone();

            remote_inner.spawn(move |handle| {
                let channel = channel.clone();
                let connections_inner = connections_inner.clone();
                let parsed_message_inner = parsed_message.clone();

                for (addr, _) in connections_inner.read().unwrap().iter() {
                    let channel_inner = channel.clone();
                    let parsed_message_inner = parsed_message_inner.clone();

                    if let Some(processed_message) = producer::process_outgoing(parsed_message_inner, addr) {
                        let f = channel_inner.send((addr.to_string(), processed_message));
                        spawn_future(f, "Send message to write handler", handle);
                    }
                }

                Ok(())
            });
        }

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
