#[macro_use] extern crate log;
extern crate chrono;
extern crate clap;
extern crate colored;
extern crate fern;

extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

extern crate rdkafka;

mod context;
mod server;

use colored::*;
use clap::{App, Arg};
use log::{LogLevel, LogLevelFilter};

use context::Context;
use server::bootstrap_server;

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

    let ctx = Context::new(matches);
    bootstrap_server(ctx);
}
