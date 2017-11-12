use chrono::Local;
use serde_json::{from_str, to_string};

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    timestamp: String,
    from: String,
    data: String
}

/// Process a message that is received from a WebSocket connection.
///
/// If `None` is returned, then the message is not sent to Kafka.
pub fn process_incoming(from: String, message: String) -> Option<String> {
    let now = Local::now();

    let constructed_message = Message {
        timestamp: now.to_rfc2822(),
        from: from,
        data: message
    };

    info!("Received message on {:?} from {:?} containing: {:?}",
          constructed_message.timestamp, constructed_message.from,
          constructed_message.data);

    Some(to_string(&constructed_message).unwrap())
}

/// Parse a message from Kafka.
///
/// If `None` is returned, it will not be sent to any WebSocket clients.
pub fn parse_message(raw_message: String) -> Result<Message, ()> {
    let message = from_str(&raw_message);

    info!("Parsing message from Kafka: {:?}", raw_message);

    message.map_err(|_| error!("Error parsing {:?}", raw_message))
}

/// Decide whether a message should be sent to a client.
///
/// If `None` is returned, it will not be sent to any WebSocket clients.
pub fn process_outgoing(message: Message, to: &str) -> Option<String> {
    // Don't send a event back to the original author.
    if message.from == to {
        None
    } else {
        info!("Sending message to {:?}: {:?}", to, message.data);
        Some(to_string(&message).unwrap())
    }
}
