# Samples
This document contains some valid sample messages that can be used to test functionality of the event bus.

## Sticky Round Robin
```
// register to all
{ "message_type": "register", "event_types": ["deposit", "withdrawal"], "client_type": "transaction" }

// send to one and sticky for 234
{ "message_type": "new", "events": [ { "consistency": { "key": "acc-234", "value": 0 }, "correlation_id": 28374938, "event_type": "deposit", "data": { "blue": "green" } } ] }

// send to second and sticky for 358
{ "message_type": "new", "events": [ { "consistency": { "key": "acc-358", "value": 0 }, "correlation_id": 28374938, "event_type": "deposit", "data": { "blue": "green" } } ] }

// go to first
{ "message_type": "new", "events": [ { "consistency": { "key": "acc-234", "value": 1 }, "correlation_id": 28374938, "event_type": "deposit", "data": { "blue": "green" } } ] }

// go to third and sticky for 876
{ "message_type": "new", "events": [ { "consistency": { "key": "acc-876", "value": 0 }, "correlation_id": 28374938, "event_type": "deposit", "data": { "blue": "green" } } ] }

// go to second
{ "message_type": "new", "events": [ { "consistency": { "key": "acc-358", "value": 1 }, "correlation_id": 28374938, "event_type": "deposit", "data": { "blue": "green" } } ] }

// go to first
{ "message_type": "new", "events": [ { "consistency": { "key": "acc-234", "value": 2 }, "correlation_id": 28374938, "event_type": "deposit", "data": { "blue": "green" } } ] }

// go to first again and sticky for 987
{ "message_type": "new", "events": [ { "consistency": { "key": "acc-987", "value": 0 }, "correlation_id": 28374938, "event_type": "deposit", "data": { "blue": "green" } } ] }

// go to second again and sticky for 236
{ "message_type": "new", "events": [ { "consistency": { "key": "acc-236", "value": 0 }, "correlation_id": 28374938, "event_type": "deposit", "data": { "blue": "green" } } ] }
```
