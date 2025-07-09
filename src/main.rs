#![warn(clippy::all)]
use std::path::PathBuf;
use std::time::Duration;

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

use crate::config::{
    parse_config_from_args, parse_monitor_configs, Config, MonitorDirConfig, OperationMode,
    ServerConfig,
};
use crate::status::{MonitorState, Status};
use crate::worker::monitor_run;

#[tokio::main]
async fn main() {
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

    let operation = match parse_config_from_args() {
        Ok(operation) => operation,
        Err(e) => {
            eprintln!();
            eprintln!("Fatal error parsing configuration:");
            eprintln!("{e}");
            return;
        }
    };
    match operation {
        OperationMode::Run(config, dry_run) => crate::http::run(config, dry_run).await,
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

                    monitor_run(monitor, &mut |_, msg| {
                        state
                            .process_message(&monitor.id, msg, &config.css.metadata, &mut |m| {
                                println!("{}", m);
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
        OperationMode::Init(path) => {
            println!("Initializing directory: {path:?}...");
            if !path.exists() {
                std::fs::create_dir_all(&path)
                    .expect(&format!("Unable to create directory {path:?}"));
            }
            let mut config = Config::default();
            config.version = 1;
            config.server.port = 8000;
            std::fs::write(
                path.join("config.yaml"),
                serde_yaml::to_string(&config).expect("Unable to write configuration"),
            )
            .expect("Unable to write configuration");

            let static_path = path.join("static");
            if !static_path.exists() {
                std::fs::create_dir_all(&static_path)
                    .expect(&format!("Unable to create directory {static_path:?}"));
            }
            std::fs::write(static_path.join("README.md"), "Create your index.html here")
                .expect("Unable to write README.md");
            let monitor_dir = path.join("monitor.d").join("monitor");
            if !monitor_dir.exists() {
                std::fs::create_dir_all(&monitor_dir)
                    .expect(&format!("Unable to create directory {monitor_dir:?}"));
            }
            let mut monitor_config = MonitorDirConfig::default();
            monitor_config.root.test_mut().command = PathBuf::from("test.sh");
            monitor_config.root.test_mut().interval = Duration::from_secs(30);
            monitor_config.root.test_mut().timeout = Duration::from_secs(10);
            std::fs::write(
                monitor_dir.join("config.yaml"),
                serde_yaml::to_string(&monitor_config).expect("Unable to write configuration"),
            )
            .expect("Unable to write configuration");
            std::fs::write(
                monitor_dir.join("test.sh"),
                "#!/bin/sh\necho 'Write your test script here'",
            )
            .expect("Unable to write test.sh");
            if cfg!(unix) {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(
                    monitor_dir.join("test.sh"),
                    std::fs::Permissions::from_mode(0o755),
                )
                .expect("Unable to set permissions on test.sh");
            }
            println!("Done!");
            println!();
            println!("Run `stylus {path:?}` to start the server");
        }
    }
}
