#![warn(clippy::all)]
use env_logger::Env;
use keepcalm::SharedMut;

mod config;
mod css;
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
use crate::status::{MonitorState, Status};
use crate::worker::monitor_run;

#[tokio::main]
async fn main() -> () {
    run().await
}

async fn run() {
    // Manually bootstrap logging from args
    let default = match std::env::args().filter(|s| s == "-v").count() {
        0 => "stylus=warn",
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
                monitors: monitors.iter().map(|m| SharedMut::new(m.into())).collect(),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&status)
                    .expect("Unable to pretty-print configuration")
            );
        }
        OperationMode::Test(config, id) => {
            let monitors = parse_monitor_configs(&config.monitor.dir)
                .expect("Unable to parse monitor configurations");
            for monitor in monitors.iter() {
                if monitor.id == id {
                    let mut state: MonitorState = monitor.into();
                    println!("Monitor Log");
                    println!("-----------");
                    println!();

                    monitor_run(&monitor, &mut |_, msg| {
                        state
                            .process_message(&monitor.id, msg, &config.css.metadata, &mut |m| {
                                eprintln!("{}", m);
                            })
                            .expect("Failed to process message");
                        Ok(())
                    })
                    .1
                    .expect("Failed to run the monitor");

                    println!();
                    println!("State");
                    println!("-----");
                    println!();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&state)
                            .expect("Unable to pretty-print configuration")
                    );

                    println!();
                    println!("CSS");
                    println!("---");
                    println!();

                    println!(
                        "{}",
                        crate::css::generate_css_for_monitor(&config.css, &state)
                    );
                    return;
                }
            }

            panic!("Unable to locate monitor with id '{}'", id)
        }
    }
}
