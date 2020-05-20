use std::convert::Infallible;
use std::sync::Arc;

use monitor::Monitor;
use status::Status;
use warp::path;
use warp::Filter;

mod config;
mod interpolate;
mod linebuf;
mod monitor;
mod status;
mod worker;

#[macro_use]
extern crate log;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

async fn css_request(monitor: Arc<Monitor>) -> Result<String, Infallible> {
    Ok(monitor.generate_css())
}

async fn status_request(monitor: Arc<Monitor>) -> Result<Status, Infallible> {
    Ok(monitor.status())
}

async fn log_request(monitor: Arc<Monitor>, s: String) -> Result<String, Infallible> {
    for monitor in monitor.status().monitors {
        let monitor = monitor.lock().expect("Poisoned mutex");
        if monitor.config.id == s {
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

#[tokio::main]
async fn main() -> () {
    env_logger::init();
    let args = std::env::args().collect::<Vec<_>>();

    // TODO: Proper command parser
    if args.len() < 2 {
        eprintln!("Usage: stylus [path-to-config.yaml]");
        return;
    }

    let file = args.get(1).unwrap().clone();
    let config = config::parse_config(file).expect("Unable to parse configuration");
    debug!("{:?}", config);
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
