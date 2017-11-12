use futures::sync::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use futures::stream::{SplitSink, SplitStream};

use tokio_core::net::TcpStream;
use tokio_core::reactor::{Core, Remote};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use rdkafka::client::EmptyContext;
use rdkafka::consumer::{Consumer, EmptyConsumerContext};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::config::ClientConfig;
use rdkafka::producer::FutureProducer;

use websocket::OwnedMessage;
use websocket::client::async::Framed;
use websocket::async::MessageCodec;

type WebSocketSink = SplitSink<Framed<TcpStream, MessageCodec<OwnedMessage>>>;
type WebSocketStream = SplitStream<Framed<TcpStream, MessageCodec<OwnedMessage>>>;

#[derive(Clone)]
pub struct ServerState {
    pub remote: Remote,

    pub connections: Arc<RwLock<HashMap<String, WebSocketSink>>>,

    pub receive_channel_out: UnboundedSender<(String, WebSocketStream)>,
    pub send_channel_out: UnboundedSender<(String, String)>,

    pub producer: FutureProducer<EmptyContext>,

    pub topic: String,
}

impl ServerState {
    pub fn create(core: &Core, brokers: &str, group: &str, topic: &str)
        -> (ServerState,
            UnboundedReceiver<(String, WebSocketStream)>,
            UnboundedReceiver<(String, String)>,
            StreamConsumer<EmptyConsumerContext>) {
        // Multiple producer, single-consumer FIFO queue. Messages added to receive_channel_out will
        // appear in receive_channel_in.
        let (receive_channel_out, receive_channel_in) = unbounded();
        let (send_channel_out, send_channel_in) = unbounded();

        let state = Self::new(core, brokers, topic, receive_channel_out, send_channel_out);
        let consumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .create::<StreamConsumer<_>>()
            .expect("Consumer creation error");
        consumer.subscribe(&[topic]).expect("Can't subscribe to specified topic");

        (state, receive_channel_in, send_channel_in, consumer)
    }

    fn new(core: &Core, brokers: &str, topic: &str,
           receive_channel_out: UnboundedSender<(String, WebSocketStream)>,
           send_channel_out: UnboundedSender<(String, String)>) -> ServerState {

        // Create a Kafka producer for use when sending messages from websocket clients.
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("produce.offset.report", "true")
            .create::<FutureProducer<_>>()
            .expect("Producer creation error");

        ServerState {
            remote: core.remote(),
            connections: Arc::new(RwLock::new(HashMap::new())),
            receive_channel_out: receive_channel_out,
            send_channel_out: send_channel_out,
            producer: producer,
            topic: topic.to_string(),
        }
    }
}
