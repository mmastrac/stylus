#![warn(clippy::all)]
use env_logger::Env;
use std::sync::{Arc, Mutex};

mod config;
mod http;
mod interpolate;
mod monitor;
mod status;
mod worker;

#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_more;

use crate::config::{parse_config_from_args, parse_monitor_configs, OperationMode};
use crate::status::Status;

#[tokio::main]
async fn main() -> () {
    // Manually bootstrap logging from args
    let default = match std::env::args().filter(|s| s == "-v").count() {
        0 => "",
        1 => "stylus=info",
        2 => "stylus=debug",
        _ => "debug",
    };
    env_logger::init_from_env(Env::new().filter_or("STYLUS_LOG", default));

    match parse_config_from_args().expect("Unable to parse configuration") {
        OperationMode::Run(config) => crate::http::run(config).await,
        OperationMode::Dump(config) => {
            let monitors = parse_monitor_configs(&config.monitor.dir)
                .expect("Unable to parse monitor configurations");
            let status = Status {
                config,
                monitors: monitors
                    .iter()
                    .map(|m| Arc::new(Mutex::new(m.into())))
                    .collect(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&status)
                    .expect("Unable to pretty-print configuration")
            );
        }
    }
}
