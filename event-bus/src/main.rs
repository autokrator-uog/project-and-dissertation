#[macro_use] extern crate log;
extern crate chrono;
#[macro_use] extern crate clap;
extern crate colored;
extern crate fern;

extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate rdkafka;
extern crate websocket;

mod server;
mod state;
mod producer;
mod consumer;
mod common;

use colored::*;
use clap::{Arg, ArgMatches, App, AppSettings, SubCommand};
use log::{LogLevel, LogLevelFilter};

use server::bootstrap;

fn main() {
    let matches = App::new(crate_name!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("bind")
             .short("b")
             .long("bind")
             .help("Host and port to bind websocket server to")
             .default_value("localhost:8081")
             .takes_value(true))
        .arg(Arg::with_name("brokers")
             .long("broker")
             .help("Broker list in Kafka format")
             .default_value("localhost:9092")
             .takes_value(true))
        .arg(Arg::with_name("group")
             .short("g")
             .long("group")
             .help("Consumer group name")
             .default_value(crate_name!())
             .takes_value(true))
        .arg(Arg::with_name("log-level")
             .short("l")
             .long("log-level")
             .help("Log level")
             .default_value("debug")
             .possible_values(&["off", "trace", "debug", "info", "warn", "error"])
             .takes_value(true))
        .subcommand(SubCommand::with_name("server")
                    .about("Start the event bus daemon")
                    .version(crate_version!())
                    .author(crate_authors!())
                    .arg(Arg::with_name("topic")
                         .short("t")
                         .long("topic")
                         .help("Topic to send and receive messages on")
                         .required(true)
                         .takes_value(true))
        ).get_matches();

    logging(&matches);

    match matches.subcommand() {
        ("server", Some(sub)) => {
            let bind = matches.value_of("bind").unwrap();
            let brokers = matches.value_of("brokers").unwrap();
            let group = matches.value_of("group").unwrap();
            let topic = sub.value_of("topic").unwrap();

            bootstrap(bind, brokers, group, topic);
        },
        _ => { }
    };
}

fn logging(matches: &ArgMatches) {
    let level = value_t!(matches, "log-level", LogLevelFilter).unwrap_or(LogLevelFilter::Trace);

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
        .level(level)
        .chain(std::io::stdout())
        .apply().unwrap();
}
