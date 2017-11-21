# Message Specification
The event bus can currently handle three distinct types of messages: query, new event and register.

All different types of message contain at least the following fields:

Field Name | Purpose
---------- | -------
`type`     | Informs the event bus of what type of message this is and how it should be handled.

## Subscription
Once connected to the event bus, a client will receive messages for any subscribed event types, each message will be in the following format:

```json
{
      "type": "event",
      "event_type": "deposit",
      "timestamp": "2010-06-09T15:20:00-07:00"
      "sender": "V4(127.0.0.1:45837)"
      "data": {}
}
```

## Query Message
A query message is used to indicate to the event bus that the user wants every message since a given timestamp on specific topic. The following example demonstrates a message of this type:

```json
{
    "type": "query",
    "event_type": [
        "deposit"
    ],
    "since": "2010-06-09T15:20:00-07:00"
}
```

In response to a query, the event bus will return a stream of messages of the following format:

```json
{
      "type": "event",
      "event_type": "deposit",
      "timestamp": "2010-06-09T15:20:00-07:00"
      "sender": "V4(127.0.0.1:45837)"
      "data": {}
}
```

It is intended that these messages are handled in the same way as any other incoming message. There should not need to be a particular code path for handling query responses.

## New Event Message
A new event message is used to submit a new event(s) to the event bus. The following example demonstrates a message of this type:

```json
{
    "type": "new",
    "events": [
        {
            "event_type": "deposit",
            "data": {}
        },
        {
            "event_type": "withdraw1",
            "data": {}
        }
    ]
}
```

The data field is arbitrary and is intended to contain the data from the user. In response to new events, the event bus will return a single message containing the following:

```json
{
      "type": "receipt",
      "timestamp": "2010-06-09T15:20:00-07:00"
      "sender": "V4(127.0.0.1:45837)"
      "events": [
            {
                "checksum": "6f56b5595749e8fa737927a9f8a1ed5e50a9d600",
                "status": "success"
            }
      ]
}
```

This contains the SHA-1 checksum of each distinct event and whether it was successfully parsed and processed.

## Register Message
A register message is used to initialise a session with the event bus and inform of the event types of interest. The following example demonstrates a message of this type:

```json
{
    "type: "register",
    "event_types" [
        "deposit",
        "withdrawl"
    }
}
```

In response to a register message, the event bus will send a single message containing the following:

```json
{
    "type: "registration",
    "event_types" [
        "deposit",
        "withdrawl"
    }
}
```
