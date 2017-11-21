use rdkafka::client::EmptyContext;
use rdkafka::producer::FutureProducer;
use producer::Message;

#[derive(Serialize, Deserialize)]
pub struct RegisterMessage {
    pub event_types: Vec<String>,
}

impl Message for RegisterMessage {
    fn process(&self, _addr: String, _producer: FutureProducer<EmptyContext>, _topic: String) {
        unimplemented!();
    }
}
