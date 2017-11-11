#[macro_use] extern crate log;
extern crate chrono;
#[macro_use] extern crate clap;
extern crate colored;
extern crate fern;

extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

extern crate rdkafka;

mod context;
mod server;

use colored::*;
use clap::{Arg, App, AppSettings, SubCommand};
use log::{LogLevel, LogLevelFilter};

use context::Context;
use server::bootstrap_server;

fn main() {
    let matches = App::new(crate_name!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("brokers")
             .short("b")
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
             .default_value("trace")
             .possible_values(&["off", "trace", "debug", "info", "warn", "error"])
             .takes_value(true))
        .subcommand(SubCommand::with_name("server")
                    .about("Start the event bus daemon")
                    .version(crate_version!())
                    .author(crate_authors!())
                    .arg(Arg::with_name("input-topic")
                         .short("i")
                         .long("input")
                         .help("Input topic")
                         .required(true)
                         .takes_value(true))
                    .arg(Arg::with_name("output-topic")
                         .short("o")
                         .long("output")
                         .help("Output topic")
                         .required(true)
                         .takes_value(true))
        ).get_matches();

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

    match matches.subcommand_name() {
        Some("server") => {
            let ctx = Context::new(matches);
            bootstrap_server(ctx);
        },
        None => { },
        _ => { }
    };
}
