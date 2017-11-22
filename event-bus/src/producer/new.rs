use producer::Message;
use common::{Event, EventContents};

use chrono::Local;
use serde_json::to_string;

use rdkafka::client::EmptyContext;
use rdkafka::producer::FutureProducer;

#[derive(Serialize, Deserialize)]
pub struct NewEvent {
    pub event_type: String,
    pub data: EventContents,
}

#[derive(Serialize, Deserialize)]
pub struct NewEventMessage {
    pub events: Vec<NewEvent>,
}

impl Message for NewEventMessage {
    fn process(&self, addr: String, producer: FutureProducer<EmptyContext>, topic: String) {
        for raw_event in self.events.iter() {
            let event = Event {
                timestamp: Local::now().to_rfc2822(),
                addr: addr.clone(),
                event_type: raw_event.event_type.clone(),
                data: raw_event.data.clone()
            };

            info!("Sending {:?} to key {:?} on topic {:?}", event, event.event_type, topic);
            let serialized_event = to_string(&event).unwrap();
            producer.send_copy::<String, String>(&topic, None,
                                                 Some(&serialized_event), Some(&event.event_type),
                                                 None, 1000);
        }
    }
}
