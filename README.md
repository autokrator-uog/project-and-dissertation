# SED Project (Autokrator) - An Event Sourced Financial Platform
This repository acts as a central hub for the SED team project, containing all documentation and specifications related to the child projects as well as the final dissertation. This repository also contains all of the issues for the project.

View [presentation slides](https://docs.google.com/presentation/d/1e55Ijo_pt1K18ZhhTJlBnb1YKaaL5c-ZVjO15hiiWi8/edit?usp=sharing) for this project.

## About the project
This project is the result of the third year team project at the University of Glasgow's Computing Science/Software Engineering course. The team was set the task of producing a event sourced financial platform by [Avaloq](https://avaloq.com/). Avaloq are a Swiss company, that specialize in the building of software for financial institutions world-wide.

Avaloq wanted a proof-of-concept event sourcing platform that would allow for the replay and distribution of arbitrary events across multiple subscribed services. In order to demonstrate this it was requested that a simple financial application capable of money transfers be created, consisting of at least three microservices communicating around a core event bus.

Event sourcing is an architecture where unlike traditional architectures the state of objects is not persisted, instead the sequence of events which created that state are. Event sourcing has numerous benefits:

  - State of any event bus client can be rebuilt entirely from the events.
  - The application state can be inspected for any point in time by composing the events up until that time. This has major benefits for auditing.

Within the requested demo application, clients should be able to subscribe and publish events to the platform:

  - Subscribers and publishers need not be on the same machine.
  - Subscribers should only receive events that interest them.
  - Events should be generic in that they can represent any textual data.
  - Events should be persistent and immutable.

Further, there were various requirements on the microservices themselves that would demonstrate the advantages of a event sourced architecture:

  - Multiple instances of a given service should be able to run in parallel and therefore provide horizontal scaling.
  - Services should be able to rebuild their state when destroyed and re-created.
  - Events should have an ordering/consistency throughout the platform.

It is also requested that a user-facing client that interacts with the three microservices be created.

There were various requirements from the university on the development processes and methodologies followed, for more information on that, please read the dissertation.

## Repositories
This project is composed of many different smaller projects that make up the full working system. Each of those projects is described and listed below.

### Core (Event Bus/Superclient)
This repository contains two core components of the system - the event bus and the superclient. The event bus is the core of the entire system, it is the single source of truth where all events are processed and distributed throughout the system, it boasts the following features:

  - [Rust](https://www.rust-lang.org/en-US/) application built using an actor architecture (using the [fantastic actix library](https://github.com/actix/actix/)).
  - Provides a websockets API for clients to register, query, acknowledge, observe and produce events.
  - Persists events to [Kafka](https://kafka.apache.org/) as a primary datastore for the events and to [Couchbase](https://www.couchbase.com/) for later querying.
  - Ensures events are processed and re-delivers to services if they are not.
  - Manages consistency/ordering between events without needing knowledge of event contents.
  - Works with multiple instances of services to avoid duplication of processing and allow for horizontal scaling of clients.

The superclient replaced earlier iterations of event bus clients and client libraries by providing a framework for building services in [Lua](https://www.lua.org/). Motivated by a desire to increase productivity and reduce iteration time when working on event bus clients, the superclient vastly decreases complexity of services and the amount of code that needed to be maintained. This application provides a stable API in Lua for handling incoming events, producing new events, registering HTTP routes and re-building state from previous events. It boasts the following features:

  - [Rust](https://www.rust-lang.org/en-US/) application built using an actor architecture (using the [fantastic actix library](https://github.com/actix/actix/)).
  - Handles communication with the event bus using shared validated JSON schemas (thanks to [serde_json](https://github.com/serde-rs/json)) over websockets!
  - Provides a HTTP server for services to advertise REST endpoints for the various operations they provide.
  - Works with [Redis](https://redis.io/) to provide services with persistent storage of state.
  - Dynamically loads and runs Lua scripts (using the excellent [rlua](https://github.com/chucklefish/rlua/)) containing only service business logic allowing for vast code re-use between services and quick iteration.
  - Builds upon event bus functionality to allow easy re-building of entire state from previous events.

### UI Backend (or Backend for a Frontend/BFAF)
This repository contains a Flask application written in Python that acts as a middleman between the various microservices and the user interface.

### UI
This repository contains a React frontend that acts as the central user interface for the project through which end-users can use the financial platform.

### Demo
This repository contains submodules for each of the various in-use repositories and takes advantage of the Docker containers of each to provide a simple way to start the entire application.

### Jackie (Reporting Service)
This repository contains a small web application written in [Rust](https://www.rust-lang.org/en-US/) with [Nickel](https://github.com/nickel-org/nickel.rs) that allows for browsing of events queried from [Couchbase](https://www.couchbase.com/). In particular, events can be browsed through their correlation and consistency relationships to other events.

## Legacy Repositories
There are also multiple repositories that contain legacy code that is no longer used in the final system but served as experimentations or initial implementations during the project's development.

### Java Client Library
Before the development of the superclient, the three required microservices were written in Java. In order to encourage code re-use, a shared client library was created that would handle the interactions between the services and the event bus, it contained the following:

  - Provides sane API for Java-based services to communicate with the event bus, including registration, consistency, receipt handling, event production and processing.
  - Fully unit tested with 100% coverage.

### Accounts/Transactions/Users Service
Before the development of the superclient, the three required microservices were written in Java supported by a client library. These three services contained the following functionality:

  - Each service provided a REST API to abstract complex business logic and service interactions from the user interface backend.
  - Behaviour-driven testing.

### Persistence
Before a revamped implementation of querying was added to the event bus proper, this repository made use of [Kafka Connect](https://www.confluent.io/product/connectors/) to persist events sent to Kafka by the event bus into Couchbase for later querying.

### Websockets Demo
In the earliest version of the event bus that only communicated with Kafka and hosted a websockets server, this proof of concept web page was used to demonstrate very basic communication with the event bus from a user-facing page.

## Specifications
Various specifications and documentation relating to how certain requirements are implemented as well as details on the various services can be found in the `specifications` folder.

### Client Apps
![Diagram](./specifications/ClientSideUpdated_5_12_17.png)

### Client Event Schemas
Please see [this document](./specifications/EVENTSCHEMAS.md).

### Event Bus Message Schemas
Please see [this document](./specifications/MESSAGES.md).

### Consistency
Please see [this document](./specifications/CONSISTENCY.md).

### Correlation
Please see [this document](./specifications/CORRELATION.md).
