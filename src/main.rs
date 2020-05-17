use std::convert::Infallible;
use std::sync::Arc;

use monitor::Monitor;
use status::Status;
use warp::Filter;

mod config;
mod interpolate;
mod monitor;
mod status;
mod worker;

#[macro_use]
extern crate log;

async fn css_request(monitor: Arc<Monitor>) -> Result<String, Infallible> {
    Ok(monitor.generate_css())
}

async fn status_request(monitor: Arc<Monitor>) -> Result<Status, Infallible> {
    Ok(monitor.status())
}

/// Provide the given arc to a warp chain
fn provide_monitor(monitor: &Arc<Monitor>) -> impl Fn() -> Arc<Monitor> + Clone {
    let monitor = Box::new(monitor.clone());
    move || *monitor.clone()
}

#[tokio::main]
async fn main() -> () {
    env_logger::init();

    let file = std::env::args()
        .collect::<Vec<_>>()
        .get(1)
        .expect("Usage: stylus [path-to-config.yaml]")
        .clone();
    let config = config::parse_config(file).expect("Unable to parse configuration");
    debug!("{:?}", config);
    let monitor = Arc::new(Monitor::new(&config).expect("Unable to create monitor"));

    // style.css for formatting
    let style = warp::path("style.css")
        .map(provide_monitor(&monitor))
        .and_then(css_request);
    // status.json for advanced integrations
    let status = warp::path("status.json")
        .map(provide_monitor(&monitor))
        .and_then(status_request)
        .map(|s| warp::reply::json(&s));
    // static pages
    let r#static = warp::fs::dir(config.server.r#static);

    let routes = warp::get().and(style.or(status).or(r#static));

    warp::serve(routes)
        .run(([127, 0, 0, 1], config.server.port))
        .await;
}
