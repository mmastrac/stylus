use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use warp::path;
use warp::Filter;

use crate::config::Config;
use crate::css::generate_css_for_state;
use crate::monitor::Monitor;
use crate::status::Status;

const VERSION: &str = env!("CARGO_PKG_VERSION");

async fn css_request(monitor: Arc<Monitor>) -> Result<String, Infallible> {
    Ok(generate_css_for_state(
        &monitor.config.css,
        &monitor.status(),
    ))
}

async fn status_request(monitor: Arc<Monitor>) -> Result<Status, Infallible> {
    Ok(monitor.status())
}

async fn log_request(monitor: Arc<Monitor>, s: String) -> Result<String, Infallible> {
    for monitor in monitor.status().monitors {
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

/// Generate an ETag from file content hash
fn generate_etag_from_file(file: &warp::filters::fs::File) -> String {
    let mut hasher = DefaultHasher::new();

    // Hash the file path and modification time for a more stable ETag
    if let Ok(metadata) = std::fs::metadata(file.path()) {
        if let Ok(modified) = metadata.modified() {
            modified.hash(&mut hasher);
        }
        metadata.len().hash(&mut hasher);
    }

    // Also hash the path as a fallback
    file.path().hash(&mut hasher);

    format!("\"{:x}\"", hasher.finish())
}

pub async fn run(config: Config) {
    let monitor = Arc::new(Monitor::new(&config).expect("Unable to create monitor"));
    let with_monitor = warp::any().map(move || monitor.clone());

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

    // static files with ETags
    let static_files =
        warp::fs::dir(config.server.r#static).map(|file: warp::filters::fs::File| {
            let etag = generate_etag_from_file(&file);
            warp::reply::with_header(file, "ETag", etag)
        });

    let routes = warp::get().and(style.or(status).or(log).or(static_files));

    let ip_addr = config
        .server
        .listen_addr
        .parse::<IpAddr>()
        .expect("Failed to parse listen address");
    let addr = SocketAddr::new(ip_addr, config.server.port);

    // We print one and only one message
    eprintln!("Stylus {} is listening on {}!", VERSION, addr);

    warp::serve(routes).run(addr).await;
}
