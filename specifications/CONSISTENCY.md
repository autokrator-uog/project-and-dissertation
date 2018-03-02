# Event Consistency
**Note:** This information isn't 100% up-to-date and may have changed in the actual implementation. Please review `core` repository for up-to-date information.

The major issues in building a decentralized application (including one centred around an event bus) is in ensuring that events sent have a consistent ordering (to avoid double spending and invalid states).

In our implementation, we make use of a pseudo-blockchain to ensure a global ordering across all messages of all event types and then further use a per-event-type sequence numbering to force clients to be up to date with messages on a given event type without requiring they read all messages. This is explained further below.

## The Problem
The issue we are trying to solve is the situation in which two clients, *Alice* and *Bob*, are sending messages to the event bus. We wish to avoid the situation in which two messages are sent simultaneously that mutate a state that the other message depends on.

An example might involve a payment being sent to an account and afterwards, a withdrawal is performed on that account. The intended situation is that the payment would be received and then the withdrawal occurs, but if these events happen in the wrong order, then the user may end up overdrawn for a period of time.

However, as it is impossible for the event bus to ensure that messages sent to it are received in the intended order (due to network latency, etc.), we can ensure that any message that has been sent was sent with knowledge of the most recent message received - this allows clients to adjust messages in response to messages that beat them to the event bus.

## The Solution
In order to ensure a global ordering on all messages, we borrow ideas from blockchains such as Bitcoin. Our ordering is irrespective of event type, since messages from different event types might mutate the same state - a deposit and a withdrawal could both mutate the same balance, despite being different event types.

If this solution was implemented, the event bus would maintain a hash of the most recent event received, and each event received would contain the hash of the event preceding it. The event bus would only accept a new event if the hash contained of the previous event matches the hash known by the event bus.

If a message was received that contained an older hash, then a response would be sent prompting the client to resend the message. This response would contain the most recent hash (so that the client does not need to subscribe to all event types to figure that hash out). This situation could occur if a client does not receive events of all event types (therefore it couldn't possibly always have the hash of the most recent event) or in the case where two messages were sent simultaneously.

However, in the case that a client subscribes to an event type and has a backlog of events of that type to process then it is possible that a client could send a new event of that type while having not finished processed all of the previous events of that type (since they'd get the hash from the event bus on a failed send).

In order to mitigate this situation, we also include a sequential number per event type, this is not provided on a failed send like the hash, and therefore requires that the client has read all the events of that type in order to obtain this value.

## Summary
We believe that this will solve the event consistency problem in our implementation.
