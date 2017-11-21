mod new;
mod query;
mod register;

use self::new::NewEventMessage;
use self::query::QueryMessage;
use self::register::RegisterMessage;

use std::str::from_utf8;
use serde_json::{from_str, Value};

use rdkafka::client::EmptyContext;
use rdkafka::producer::FutureProducer;
use websocket::message::OwnedMessage;

pub trait Message {
    fn process(&self, producer: FutureProducer<EmptyContext>, topic: String);
}

/// Process a message that is received from a WebSocket connection.
pub fn parse_message(incoming_message: OwnedMessage) -> Box<Message> {
    // Process the message we just got by forwarding on to Kafka.
    let converted_message = match incoming_message {
        OwnedMessage::Binary(bytes) => Ok(from_utf8(&bytes).unwrap().to_string()),
        OwnedMessage::Text(message) => Ok(message),
        _ => Err(())
    }.unwrap();

    let parsed_message: Value = from_str(&converted_message).unwrap();
    let message_type = parsed_message["type"].as_str().unwrap();

    match message_type {
        "query" => {
            let event: QueryMessage = from_str(&converted_message).unwrap();
            Box::new(event)
        },
        "new" => {
            let event: NewEventMessage = from_str(&converted_message).unwrap();
            Box::new(event)
        },
        "register" => {
            let event: RegisterMessage = from_str(&converted_message).unwrap();
            Box::new(event)
        },
        _ => panic!("Invalid message type")
    }
}
