#[macro_use] extern crate log;
extern crate chrono;
extern crate clap;
extern crate colored;
extern crate fern;

extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

extern crate rdkafka;

use colored::*;
use clap::{App, Arg};
use log::{LogLevel, LogLevelFilter};

use futures::Future;
use futures::stream::Stream;
use futures_cpupool::Builder;
use tokio_core::reactor::Core;

use std::str;

use rdkafka::message::Message;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::config::ClientConfig;
use rdkafka::producer::FutureProducer;

fn server(brokers: &str, group: &str, input: &str, output: &str) {
    // Create event loop. Runs on a single thread and drives pipeline.
    let mut core = Core::new().unwrap();

    // Create a CPU pool.
    let cpu_pool = Builder::new().pool_size(4).create();

    // Create the `StreamConsumer`, to receive the messages from the topic in form of a `Stream`.
    let consumer = ClientConfig::new()
        .set("group.id", group)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create::<StreamConsumer<_>>()
        .expect("Consumer creation failed");

    consumer.subscribe(&[input]).expect("Can't subscribe to specified topic");

    // Create the `FutureProducer` to produce asynchronously.
    let producer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("produce.offset.report", "true")
        .create::<FutureProducer<_>>()
        .expect("Producer creation error");

    // Create a handle to the core, that will be used to provide additional asynchronous work
    // to the event loop.
    let handle = core.handle();

    // Create the outer pipeline on the message stream.
    let processed_stream = consumer.start()
        .filter_map(|result| {
            // Filter out errors.
            match result {
                Ok(msg) => Some(msg),
                Err(kafka_error) => {
                    warn!("Error while receiving from Kafka: {:?}", kafka_error);
                    None
                }
            }
        }).for_each(|msg| {
            info!("Enqueuing message for computation");
            let producer = producer.clone();
            let output_topic = output.to_owned();
            let owned_message = msg.detach();

            // Create the inner pipeline, that represents the processing of a single event.
            let process_message = cpu_pool.spawn_fn(move || {
                // Don't do any processing for now, just forward.
                let payload = &owned_message.payload().unwrap();
                let message_as_string = str::from_utf8(payload).unwrap();
                info!("Found message on topic {:?}: {:?}",
                      owned_message.topic(), message_as_string);
                Ok(message_as_string.to_string())
            }).and_then(move |processed_message| {
                // Send the result of the computation to Kafka, asynchronously.
                info!("Sending result");
                producer.send_copy::<String, ()>(&output_topic,
                                                 None, Some(&processed_message),
                                                 None, None, 1000)
            }).and_then(|delivery_result| {
                // Once the message has been produced, print the delivery report and terminate
                // the pipeline.
                info!("Delivery report for result: {:?}", delivery_result);
                Ok(())
            }).or_else(|err| {
                // In case of error, this closure will be executed instead.
                warn!("Error while processing message: {:?}", err);
                Ok(())
            });

            // Spawns the inner pipeline in the same event pool.
            handle.spawn(process_message);
            Ok(())
        });

    info!("Starting event loop");
    // Runs the event pool until the consumer terminates.
    core.run(processed_stream).unwrap();
    info!("Stream processing terminated");
}

fn main() {
    let matches = App::new("event-bus")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .about("Event Bus Research Prototype for Avaloq - University of Glasgow Team Project SED")
        .arg(Arg::with_name("brokers")
             .short("b")
             .long("brokers")
             .help("Broker list in Kafka format")
             .takes_value(true)
             .default_value("localhost:9092"))
        .arg(Arg::with_name("group")
             .short("g")
             .long("group")
             .help("Consumer group")
             .takes_value(true)
             .default_value("default_consumer_group"))
        .arg(Arg::with_name("input-topic")
             .short("i")
             .long("input-topic")
             .help("Input topic for subscription")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("output-topic")
             .short("o")
             .long("output-topic")
             .help("Output topic for consumption")
             .takes_value(true)
             .required(true))
        .get_matches();

    fern::Dispatch::new()
        .format(|out, message, record| {
            let now = chrono::Local::now();

            let level_colour = match record.level() {
                LogLevel::Debug => "blue",
                LogLevel::Info => "green",
                LogLevel::Warn => "yellow",
                LogLevel::Error => "red",
                _ => "white"
            };
            let level = format!("{:?}", record.level()).to_uppercase().color(level_colour);

            out.finish(format_args!(
                "[{} {}] [{}] {} {}",
                now.format("%Y-%m-%d"),
                now.format("%H:%M:%S"),
                record.target(),
                level,
                message
            ))
        })
        .level(LogLevelFilter::Trace)
        .chain(std::io::stdout())
        .apply().unwrap();

    let brokers = matches.value_of("brokers").unwrap();
    let group = matches.value_of("group").unwrap();
    let input = matches.value_of("input-topic").unwrap();
    let output = matches.value_of("output-topic").unwrap();
    server(brokers, group, input, output);
}
