use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use tokio::net::TcpListener;

use crate::config::Config;
use crate::css::generate_css_for_state;
use crate::monitor::Monitor;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
struct AppState {
    monitor: Arc<Monitor>,
}

async fn css_request(State(state): State<AppState>) -> impl IntoResponse {
    let css = generate_css_for_state(&state.monitor.config.css, &state.monitor.status());
    (StatusCode::OK, [("Content-Type", "text/css")], css)
}

async fn status_request(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.monitor.status();
    Json(status)
}

async fn log_request(
    State(state): State<AppState>,
    Path(monitor_id): Path<String>,
) -> impl IntoResponse {
    for monitor in state.monitor.status().monitors {
        let monitor = monitor.read();
        if monitor.id == monitor_id {
            let mut logs = String::new();
            for log in &monitor.status.log {
                logs += log;
                logs += "\n";
            }
            return (StatusCode::OK, [("Content-Type", "text/plain")], logs);
        }
    }
    (
        StatusCode::NOT_FOUND,
        [("Content-Type", "text/plain")],
        "Not found".to_string(),
    )
}

#[cfg(not(feature = "builtin-ui"))]
async fn default_index(State(state): State<AppState>) -> impl IntoResponse {
    use crate::status::MonitorState;
    use handlebars::Handlebars;
    use keepcalm::SharedMut;
    use serde::Serialize;
    use std::convert::identity;

    let mut handlebars = Handlebars::new();
    if let Err(e) = handlebars.register_template_string("t", include_str!("./index.html")) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            [("Content-Type", "text/html")],
            e.to_string(),
        );
    }

    let mut monitors = vec![];
    for monitor in state.monitor.status().monitors {
        monitors.push(monitor);
    }

    #[derive(Serialize)]
    struct Model {
        monitors: Vec<SharedMut<MonitorState>>,
    }

    let html = handlebars
        .render("t", &Model { monitors })
        .map_or_else(|e| e.to_string(), identity)
        .trim()
        .to_owned();

    (
        StatusCode::OK,
        [("Content-Type", "text/html; charset=utf-8")],
        html,
    )
}

/// Generate an ETag from file content hash
fn generate_etag_from_path(path: &std::path::Path) -> String {
    if let Ok(content) = std::fs::read_to_string(path) {
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
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                modified.hash(&mut hasher);
            }
            metadata.len().hash(&mut hasher);
        }

        path.hash(&mut hasher);
        format!("\"W/{:x}\"", hasher.finish())
    }
}

/// Handle ETag cache validation for static files
async fn handle_static_file_with_etag(headers: HeaderMap, file_path: PathBuf) -> impl IntoResponse {
    let etag = generate_etag_from_path(&file_path);
    let extension = file_path
        .extension()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let content_type = match extension.as_str() {
        "js" | "jsx" | "ts" | "tsx" => "text/javascript; charset=utf-8",
        "svg" => "image/svg+xml; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "html" | "htm" | "xhtml" => "text/html; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    };

    let cache_control = "no-cache, must-revalidate";

    // Check if client sent If-None-Match header
    if let Some(client_etag) = headers.get("if-none-match") {
        if let Ok(client_etag_str) = client_etag.to_str() {
            // Remove quotes if present for comparison
            let client_etag = client_etag_str.trim_matches('"');
            let server_etag = etag.trim_matches('"');

            if client_etag == server_etag {
                // ETags match, return 304 Not Modified
                let mut response = Response::new(axum::body::Body::empty());
                *response.status_mut() = StatusCode::NOT_MODIFIED;
                response
                    .headers_mut()
                    .insert("ETag", HeaderValue::from_str(&etag).unwrap());
                response
                    .headers_mut()
                    .insert("Cache-Control", HeaderValue::from_static(cache_control));
                return response;
            }
        }
    }

    // Read file content
    match std::fs::read(&file_path) {
        Ok(content) => {
            let mut response = Response::new(axum::body::Body::from(content));
            response
                .headers_mut()
                .insert("Content-Type", HeaderValue::from_static(content_type));
            response
                .headers_mut()
                .insert("Cache-Control", HeaderValue::from_static(cache_control));
            response
                .headers_mut()
                .insert("ETag", HeaderValue::from_str(&etag).unwrap());
            response
        }
        Err(_) => {
            let mut response = Response::new(axum::body::Body::from("File not found"));
            *response.status_mut() = StatusCode::NOT_FOUND;
            response
                .headers_mut()
                .insert("Content-Type", HeaderValue::from_static("text/plain"));
            response
        }
    }
}

/// Custom static file handler with ETag support
async fn static_files_handler(
    headers: HeaderMap,
    Path(file): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(static_path) = &state.monitor.config.server.static_path {
        let full_path = static_path.join(&file);

        // Security check: ensure the file is within the static directory
        if !full_path.starts_with(static_path) {
            return (
                StatusCode::FORBIDDEN,
                [("Content-Type", "text/plain")],
                "Access denied".to_string(),
            )
                .into_response();
        }

        if full_path.exists() && full_path.is_file() {
            return handle_static_file_with_etag(headers, full_path)
                .await
                .into_response();
        }
    }

    if file.is_empty() {
        #[cfg(not(feature = "builtin-ui"))]
        {
            return index_handler(state.monitor.config.clone())
                .await
                .into_response();
        }
        #[cfg(feature = "builtin-ui")]
        {
            return (
                StatusCode::OK,
                [("Content-Type", "text/html; charset=utf-8")],
                stylus_ui::STYLUS_HTML,
            )
                .into_response();
        }
    }

    (
        StatusCode::NOT_FOUND,
        [("Content-Type", "text/plain")],
        "File not found".to_string(),
    )
        .into_response()
}

async fn index_handler(config: Config) -> impl IntoResponse {
    if let Some(static_path) = &config.server.static_path {
        let full_path = static_path.join("index.html");
        if full_path.exists() {
            return handle_static_file_with_etag(HeaderMap::new(), full_path)
                .await
                .into_response();
        }
    }
    (
        StatusCode::OK,
        [("Content-Type", "text/html; charset=utf-8")],
        stylus_ui::STYLUS_HTML,
    )
        .into_response()
}

pub async fn run(config: Config, dry_run: bool) {
    let monitor = Arc::new(Monitor::new(&config).expect("Unable to create monitor"));
    let state = AppState { monitor };

    // Build the router
    let mut app = Router::new()
        .route("/style.css", get(css_request))
        .route("/status.json", get(status_request))
        .route("/log/:monitor_id", get(log_request));

    #[cfg(not(feature = "builtin-ui"))]
    {
        app = app.route("/", get(default_index));
    }

    #[cfg(feature = "builtin-ui")]
    {
        let config = config.clone();
        app = app
            .route("/", get(move || index_handler(config.clone())))
            .route(
                "/stylus.js",
                get(|| async {
                    (
                        StatusCode::OK,
                        [("Content-Type", "text/javascript; charset=utf-8")],
                        stylus_ui::STYLUS_JAVASCRIPT,
                    )
                }),
            )
            .route(
                "/stylus.css",
                get(|| async {
                    (
                        StatusCode::OK,
                        [("Content-Type", "text/css; charset=utf-8")],
                        stylus_ui::STYLUS_CSS,
                    )
                }),
            );
    }

    // Add static files route if configured
    if config.server.static_path.is_some() {
        app = app.route("/*file", get(static_files_handler));
    }

    let ip_addr = config
        .server
        .listen_addr
        .parse::<IpAddr>()
        .expect("Failed to parse listen address");
    let addr = SocketAddr::new(ip_addr, config.server.port);

    // We print one and only one message
    eprintln!("Stylus {} is listening on {}!", VERSION, addr);

    if dry_run {
        eprintln!("Dry run complete. Exiting.");
        return;
    }

    // Run the server
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app.with_state(state))
        .await
        .expect("Server error");
}
