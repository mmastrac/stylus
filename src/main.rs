#![warn(clippy::all)]
use std::convert::Infallible;
use std::sync::Arc;

use env_logger::Env;
use monitor::Monitor;
use status::Status;
use warp::path;
use warp::Filter;

mod config;
mod interpolate;
mod monitor;
mod status;
mod worker;

#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_more;

use crate::config::{parse_config_from_args, Config, OperationMode};

const VERSION: &str = env!("CARGO_PKG_VERSION");

async fn css_request(monitor: Arc<Monitor>) -> Result<String, Infallible> {
    Ok(monitor.generate_css())
}

async fn status_request(monitor: Arc<Monitor>) -> Result<Status, Infallible> {
    Ok(monitor.status())
}

async fn log_request(monitor: Arc<Monitor>, s: String) -> Result<String, Infallible> {
    for monitor in monitor.status().monitors {
        let monitor = monitor.lock().expect("Poisoned mutex");
        if monitor.id == s {
            let mut logs = String::new();
            for log in &monitor.log {
                logs += &log;
                logs += "\n";
            }
            return Ok(logs);
        }
    }
    Ok("Not found".to_owned())
}

/// Provide the given arc to a warp chain
fn provide_monitor(monitor: &Arc<Monitor>) -> impl Fn() -> Arc<Monitor> + Clone {
    let monitor = Box::new(monitor.clone());
    move || *monitor.clone()
}

/// Provide the given arc to a warp chain
fn provide_monitor_2<T>(monitor: &Arc<Monitor>) -> impl Fn(T) -> (T, Arc<Monitor>) + Clone {
    let monitor = Box::new(monitor.clone());
    move |t| (t, *monitor.clone())
}

async fn run(config: Config) {
    let monitor = Arc::new(Monitor::new(&config).expect("Unable to create monitor"));

    // style.css for formatting
    let style = path!("style.css")
        .map(provide_monitor(&monitor))
        .and_then(css_request)
        .with(warp::reply::with::header("Content-Type", "text/css"));
    // status.json for advanced integrations
    let status = path!("status.json")
        .map(provide_monitor(&monitor))
        .and_then(status_request)
        .map(|s| warp::reply::json(&s))
        .with(warp::reply::with::header(
            "Content-Type",
            "application/json",
        ));
    let log = path!("log" / String)
        .map(provide_monitor_2(&monitor))
        .and_then(|(s, m)| log_request(m, s))
        .with(warp::reply::with::header("Content-Type", "text/plain"));
    // static pages
    let r#static = warp::fs::dir(config.server.r#static);

    let routes = warp::get().and(style.or(status).or(log).or(r#static));

    // We print one and only one message
    eprintln!(
        "Stylus {} is listening on 0.0.0.0:{}!",
        VERSION, config.server.port
    );

    warp::serve(routes)
        .run(([0, 0, 0, 0], config.server.port))
        .await;
}

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
        OperationMode::Run(config) => run(config).await,
        OperationMode::Dump(config) => {
            let monitor = Monitor::new(&config).expect("Unable to create monitor");
            println!(
                "{}",
                serde_json::to_string_pretty(&monitor.status())
                    .expect("Unable to pretty-print configuration")
            );
        }
    }
}
