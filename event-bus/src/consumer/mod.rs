use serde_json::{from_str, to_string};
use common::Event;

/// Parse a message from Kafka.
///
/// If `None` is returned, it will not be sent to any WebSocket clients.
pub fn parse_message(raw_message: String) -> Event {
    info!("Parsing message from Kafka: {:?}", raw_message);

    let event: Result<Event, _> = from_str(&raw_message);
    event.unwrap()
}

/// Decide whether a message should be sent to a client.
///
/// If `None` is returned, it will not be sent to any WebSocket clients.
pub fn process_event(event: Event, to: &str) -> Option<String> {
    // Don't send a event back to the original author.
    if event.addr == to {
        None
    } else {
        info!("Sending message to {:?}: {:?}", to, event.data);
        Some(to_string(&event).unwrap())
    }
}
