# Correlation
Correlation attempts to solve the problem of keeping track of events created during a session and within a unit of work. There are two components that make up correlation - session tracking and correlation ids.

## Session Tracking
Session tracking consists of the generation of a random (globally unique) number that lives for the length of a session (the length of a given websocket connection) and is added to all Kafka messages. This number can be used to link all events that were created in that session together.

## Correlation Ids
Correlation ids requires modifications to both the event bus and client library. Correlation ids are random (globally unique) numbers that identify a unit of work that spans a series of events (such as a pending transaction going to a completed transaction or rejected transaction). It consists of a field on incoming messages, Kafka messages and outgoing messages that is unmodified by the event bus.

The event bus client will generate the random number and include it in the message, the event bus client will then be able to supply a "relevant event" when adding future events and the correlation id will be taken from that event if supplied (else generated). Through this, events can easily be linked by providing the previous event when creating the next.
