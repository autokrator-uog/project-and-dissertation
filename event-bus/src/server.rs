use futures::{Future, Sink, Stream};
use futures::sync::mpsc;
use futures_cpupool::CpuPool;
use tokio_core::reactor::{Handle, Core};

use std::str::from_utf8;
use std::collections::HashMap;
use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::fmt::Debug;

use rdkafka::Message;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::config::ClientConfig;
use rdkafka::producer::FutureProducer;

use websocket::message::OwnedMessage;
use websocket::async::Server;
use websocket::server::InvalidConnection;

pub fn bootstrap(bind: &str, brokers: &str, group: &str,
                 input: &str, output: &str) {
    // Create our event loop.
    let mut core = Core::new().expect("Failed to create Tokio event loop");

    // Create a handle for spawning futures from the main thread.
    let handle = core.handle();
    // Create a remote for spawning futures from outside the main thread.
    let remote = core.remote();

    // Create the websocket server and add it on the event loop.
    let server = Server::bind(bind, &handle).expect("Failed to create websocket server");
    info!("Running websocket server on {:?}", bind);

    // Create a (single-threaded) reference-counted CPU pool.
    let cpu_pool = Rc::new(CpuPool::new_num_cpus());
    // Create a thread-safe reference counted, hashmap with a read-write lock. Used to contain the
    // thread id to output sink mappings.
    let connections = Arc::new(RwLock::new(HashMap::new()));

    // Multiple producer, single-consumer FIFO queue. Messages added to receive_channel_out will
    // appear in receive_channel_in.
    let (receive_channel_out, receive_channel_in) = mpsc::unbounded();

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
    let producer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("produce.offset.report", "true")
        .create::<FutureProducer<_>>()
        .expect("Producer creation error");

    let connections_inner = connections.clone();
    let remote_inner = remote.clone();
    let output_inner = output.to_string();
    let producer_inner = producer.clone();
    // Spawn a handler for outgoing clients so that this doesn't block the main thread.
    let receive_handler = cpu_pool.spawn_fn(|| {
        receive_channel_in.for_each(move |(addr, stream)| {
            let connections_inner = connections_inner.clone();
            let remote_inner = remote_inner.clone();
            let remote_inner2 = remote_inner.clone();
            let output_inner = output_inner.clone();
            let producer_inner = producer_inner.clone();
            // For each client, spawn a new thread that will process everything we receive from
            // it.
            remote_inner.spawn(move |_| {
                let connections_inner = connections_inner.clone();
                let remote_inner = remote_inner2.clone();
                let output_inner = output_inner.clone();
                let producer_inner = producer_inner.clone();
                stream.for_each(move |msg| {
                    let connections_inner = connections_inner.clone();
                    let remote_inner = remote_inner.clone();
                    let output_inner = output_inner.clone();
                    let producer_inner = producer_inner.clone();
                    // Process the message we just got by forwarding on to Kafka.
                    let converted = match msg {
                        OwnedMessage::Binary(bytes) => Some(from_utf8(&bytes).unwrap().to_string()),
                        OwnedMessage::Text(message) => Some(message),
                        OwnedMessage::Close(_) => {
                            connections_inner.write().unwrap().remove(&addr);
                            None
                        },
                        _ => None
                    };

                    if let Some(message_as_string) = converted {
                        remote_inner.spawn(move |_| {
                            info!("Sending {:?} to topic {:?}", output_inner, message_as_string);
                            producer_inner.send_copy::<String, ()>(&output_inner, None, Some(&message_as_string),
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

    let (send_channel_out, send_channel_in) = mpsc::unbounded();

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

    let consumer = ClientConfig::new()
        .set("group.id", group)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create::<StreamConsumer<_>>()
        .expect("Consumer creation failed");

    consumer.subscribe(&[input]).expect("Can't subscribe to topic");

    let connections_inner = connections.clone();
    let remote_inner = remote.clone();
    let channel = send_channel_out.clone();
    let input_inner = input.to_string();
    let consumer_handler = consumer.start().filter_map(|result| {
        match result {
            Ok(msg) => Some(msg),
            Err(kafka_error) => {
                warn!("Error while receiving from Kafka {:?}", kafka_error);
                None
            }
        }
    }).for_each(move |msg| {
        let input_inner = input_inner.clone();
        let connections_inner = connections_inner.clone();
        let channel = channel.clone();
        let owned_message = msg.detach();

        remote_inner.spawn(move |handle| {
            let channel = channel.clone();
            let connections_inner = connections_inner.clone();
            let input_inner = input_inner.clone();

            for (addr, _) in connections_inner.read().unwrap().iter() {
                let channel_inner = channel.clone();
                let input_inner = input_inner.clone();

                let message_as_string = from_utf8(&owned_message.payload().unwrap()).unwrap();
                info!("Got {:?} from Kafka on topic {:?}, forwarding to {:?}", message_as_string,
                      input_inner, addr);
                let f = channel_inner.send((addr.to_string(), message_as_string.to_string()));
                spawn_future(f, "Send message to write handler", handle);
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
