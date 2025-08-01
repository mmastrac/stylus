#![warn(clippy::all)]
use std::path::Path;

use env_logger::Env;
use include_directory::{include_directory, Dir};
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
        OperationMode::Init(path, docker) => {
            println!("Initializing directory: {path:?}...");
            if !path.exists() {
                std::fs::create_dir_all(&path)
                    .expect(&format!("Unable to create directory {path:?}"));
            }
            write_template(&path);

            println!("Done!");
            println!();
            if docker {
                println!("Re-run the container with no arguments to start the server");
            } else {
                println!("Run `stylus run {path:?}` to start the server");
            }
        }
    }
}

fn write_template(path: &Path) {
    static TEMPLATE: Dir = include_directory!("$CARGO_MANIFEST_DIR/src/template");
    write_template_recursive(path, &TEMPLATE);
}

fn write_template_recursive(root_path: &Path, dir: &Dir) {
    let dir_path = root_path.join(dir.path());
    std::fs::create_dir_all(&dir_path).expect(&format!("Unable to create directory {dir_path:?}"));

    for dir in dir.dirs() {
        write_template_recursive(root_path, &dir);
    }

    for file in dir.files() {
        let file_path = root_path.join(file.path());
        std::fs::write(&file_path, file.contents())
            .expect(&format!("Unable to write file {file:?}"));

        if file.mimetype() == "application/x-shellscript" {
            if cfg!(unix) {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(file_path, std::fs::Permissions::from_mode(0o755))
                    .expect("Unable to set permissions on test.sh");
            }
        }
    }
}
