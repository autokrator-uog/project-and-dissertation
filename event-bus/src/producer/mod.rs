use chrono::Local;
use serde_json::to_string;
use common::Message;

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

