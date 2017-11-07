#[macro_use] extern crate log;
extern crate chrono;
extern crate clap;
extern crate colored;
extern crate fern;
extern crate rdkafka;

use colored::*;
use clap::{App, Arg};
use log::{LogLevel, LogLevelFilter};

fn main() {
    let matches = App::new("event-bus")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .about("Event Bus")
        .arg(Arg::with_name("brokers")
             .short("b")
             .long("brokers")
             .help("Broker list in Kafka format")
             .takes_value(true)
             .default_value("localhost:9092"))
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

    info!("Hello, world!");
}
