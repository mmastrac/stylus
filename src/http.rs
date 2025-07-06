use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use tower::make::Shared;
use tower::util::MapRequestLayer;
use tower::ServiceBuilder;
use warp::http::StatusCode;
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
                logs += log;
                logs += "\n";
            }
            return Ok(logs);
        }
    }
    Ok("Not found".to_owned())
}

/// Generate an ETag from file content hash
fn generate_etag_from_file(file: &warp::filters::fs::File) -> String {
    if let Ok(content) = std::fs::read_to_string(file.path()) {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash1 = hasher.finish();
        // Stretch the hash by using the file size. This isn't
        // cryptographic by any means.
        content.len().hash(&mut hasher);
        content.hash(&mut hasher);
        let hash2 = hasher.finish();
        format!("\"{:x}{:x}\"", hash1, hash2)
    } else {
        let mut hasher = DefaultHasher::new();
        if let Ok(metadata) = std::fs::metadata(file.path()) {
            if let Ok(modified) = metadata.modified() {
                modified.hash(&mut hasher);
            }
            metadata.len().hash(&mut hasher);
        }

        file.path().hash(&mut hasher);
        format!("\"W/{:x}\"", hasher.finish())
    }
}

/// Handle ETag cache validation
async fn handle_etag_cache(
    reply: warp::filters::fs::File,
    if_none_match: Option<String>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let etag = generate_etag_from_file(&reply);
    let cache_control = "no-cache, must-revalidate";

    // Check if client sent If-None-Match header
    if let Some(client_etag) = if_none_match {
        // Remove quotes if present for comparison
        let client_etag = client_etag.trim_matches('"');
        let server_etag = etag.trim_matches('"');

        if client_etag == server_etag {
            // ETags match, return 304 Not Modified
            let reply = warp::reply::with_header(
                warp::reply::with_header("", "ETag", etag),
                "Cache-Control",
                cache_control,
            );
            return Ok(Box::new(warp::reply::with_status(
                reply,
                StatusCode::NOT_MODIFIED,
            )));
        }
    }

    Ok(Box::new(warp::reply::with_header(
        warp::reply::with_header(reply, "Cache-Control", cache_control),
        "ETag",
        etag,
    )))
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

    // static files with ETags and cache validation
    let static_files = warp::fs::dir(config.server.r#static)
        .and(warp::header::optional::<String>("if-none-match"))
        .and_then(
            move |file: warp::filters::fs::File, if_none_match: Option<String>| {
                handle_etag_cache(file, if_none_match)
            },
        );

    let routes = warp::get().and(style.or(status).or(log).or(static_files));

    // Convert warp routes to tower service
    let warp_service = warp::service(routes);

    // Apply tower middleware to remove If-Modified-Since headers
    let service = ServiceBuilder::new()
        .layer(MapRequestLayer::new(|mut req: hyper::Request<_>| {
            req.headers_mut().remove("if-modified-since");
            req
        }))
        .service(warp_service);

    // Wrap with Shared to make it a MakeService
    let make_service = Shared::new(service);

    let ip_addr = config
        .server
        .listen_addr
        .parse::<IpAddr>()
        .expect("Failed to parse listen address");
    let addr = SocketAddr::new(ip_addr, config.server.port);

    // We print one and only one message
    eprintln!("Stylus {} is listening on {}!", VERSION, addr);

    // Run with hyper instead of warp::serve
    _ = hyper::Server::bind(&addr).serve(make_service).await;
}
