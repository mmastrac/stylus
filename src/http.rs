use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};

use keepcalm::{Shared, SharedMut};
use warp::path;
use warp::Filter;

use crate::config::Config;
use crate::css::generate_css_for_state;
use crate::monitor::Monitor;
use crate::status::Status;

const VERSION: &str = env!("CARGO_PKG_VERSION");

async fn css_request(monitor: Shared<Monitor>) -> Result<String, Infallible> {
    Ok(generate_css_for_state(
        &monitor.read().config.css,
        &monitor.read().status(),
    ))
}

async fn status_request(monitor: Shared<Monitor>) -> Result<Status, Infallible> {
    Ok(monitor.read().status())
}

async fn log_request(monitor: Shared<Monitor>, s: String) -> Result<String, Infallible> {
    for monitor in monitor.read().status().monitors {
        let monitor = monitor.read();
        if monitor.id == s {
            let mut logs = String::new();
            for log in &monitor.status.log {
                logs += &log;
                logs += "\n";
            }
            return Ok(logs);
        }
    }
    Ok("Not found".to_owned())
}

pub async fn run(config: Config) {
    let monitor = SharedMut::new(Monitor::new(&config).expect("Unable to create monitor"));
    let monitor_shared = monitor.shared_copy();
    let with_monitor = warp::any().map(move || monitor_shared.clone());

    // style.css for formatting
    let style = path!("style.css")
        .and(with_monitor.clone())
        .and_then(css_request)
        .with(warp::reply::with::header("Content-Type", "text/css"));

    // status.json for advanced integrations
    let status = path!("status.json")
        .and(with_monitor.clone())
        .and_then(status_request)
        .map(|s| warp::reply::json(&s))
        .with(warp::reply::with::header(
            "Content-Type",
            "application/json",
        ));

    // logging endpoint
    let log = path!("log" / String)
        .and(with_monitor.clone())
        .and_then(|s, m| log_request(m, s))
        .with(warp::reply::with::header("Content-Type", "text/plain"));

    // static pages
    let r#static = warp::fs::dir(config.server.r#static);

    let routes = warp::get().and(style.or(status).or(log).or(r#static));

    let ip_addr = config
        .server
        .listen_addr
        .parse::<IpAddr>()
        .expect("Failed to parse listen address");
    let addr = SocketAddr::new(ip_addr, config.server.port);

    let once = AtomicBool::new(false);
    ctrlc::set_handler(move || {
        if once
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            eprintln!("Ctrl-C pressed, attempting to shut down. Press Ctrl-C again to force.");
            monitor.write().close();
            std::process::exit(0);
        } else {
            eprintln!("Ctrl-C pressed again, exiting.");
            std::process::exit(1);
        }
    })
    .expect("Failed to set Ctrl-C handler");

    // We print one and only one message
    eprintln!("Stylus {} is listening on {}", VERSION, addr);

    warp::serve(routes).run(addr).await;
}
