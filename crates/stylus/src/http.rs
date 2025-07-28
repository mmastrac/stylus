use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json},
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

async fn default_index(state: AppState) -> impl IntoResponse {
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
        generate_etag_from_string(&content)
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

/// Generate an ETag from string content
fn generate_etag_from_string(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let hash1 = hasher.finish();
    content.len().hash(&mut hasher);
    content.hash(&mut hasher);
    let hash2 = hasher.finish();
    format!("\"{:x}{:x}\"", hash1, hash2)
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

    let cache_control = if cfg!(debug_assertions) {
        "no-cache"
    } else {
        // Allow caching for 10 seconds, and stale for 60 seconds
        "max-age=10, stale-while-revalidate=60"
    };

    // Check if client sent If-None-Match header
    if etag_matches(&headers, &etag) {
        return (
            StatusCode::NOT_MODIFIED,
            [
                ("ETag", HeaderValue::from_str(&etag).unwrap()),
                ("Cache-Control", HeaderValue::from_static(cache_control)),
            ],
            "",
        )
            .into_response();
    }

    // Read file content
    match std::fs::read(&file_path) {
        Ok(content) => (
            StatusCode::OK,
            [
                ("Content-Type", HeaderValue::from_static(content_type)),
                ("Cache-Control", HeaderValue::from_static(cache_control)),
                ("ETag", HeaderValue::from_str(&etag).unwrap()),
            ],
            content,
        )
            .into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            [("Content-Type", HeaderValue::from_static("text/plain"))],
            "File not found",
        )
            .into_response(),
    }
}

fn etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    if let Some(client_etag) = headers.get("if-none-match") {
        if let Ok(client_etag_str) = client_etag.to_str() {
            let client_etag = client_etag_str.trim_matches('"');
            let server_etag = etag.trim_matches('"');
            return client_etag == server_etag;
        }
    }
    false
}

fn handle_static_content_with_etag(
    headers: HeaderMap,
    content_type: &'static str,
    sourcemap: Option<&'static str>,
    content: &'static str,
) -> impl IntoResponse {
    let cache_control = if cfg!(debug_assertions) {
        "no-cache"
    } else {
        // Allow caching for 10 seconds, and stale for 60 seconds
        "max-age=10, stale-while-revalidate=60"
    };

    let etag = generate_etag_from_string(content);
    if etag_matches(&headers, &etag) {
        return (
            StatusCode::NOT_MODIFIED,
            [
                ("ETag", HeaderValue::from_str(&etag).unwrap()),
                ("Cache-Control", HeaderValue::from_static(cache_control)),
            ],
            "",
        )
            .into_response();
    }

    let res = (StatusCode::OK,
    [
        ("Content-Type", content_type),
        ("Cache-Control", cache_control),
        ("ETag", etag.as_str()),
    ],
    content);

    if let Some(sourcemap) = sourcemap {
        return (
            res.0,
            res.1,
            [
                ("SourceMap", sourcemap)
            ],
            res.2,
        ).into_response();
    }

    return res.into_response();
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

    (
        StatusCode::NOT_FOUND,
        [("Content-Type", "text/plain")],
        "File not found".to_string(),
    )
        .into_response()
}

async fn index_handler(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    if let Some(static_path) = &state.monitor.config.server.static_path {
        let full_path = static_path.join("index.html");
        if full_path.exists() {
            return handle_static_file_with_etag(headers, full_path)
                .await
                .into_response();
        }
    }

    if cfg!(not(feature = "builtin-ui")) {
        return default_index(state).await.into_response();
    }

    #[cfg(feature = "builtin-ui")]
    {
        return handle_static_content_with_etag(headers, "text/html; charset=utf-8", None, &stylus_ui::STYLUS_HTML)
            .into_response();
    }

    #[allow(unreachable_code)]
    (
        StatusCode::NOT_FOUND,
        [("Content-Type", "text/plain")],
        "File not found".to_string(),
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
        .route("/log/:monitor_id", get(log_request))
        .route("/", get(index_handler));

    #[cfg(feature = "builtin-ui")]
    {
        app = app
            .route(
                "/stylus.js",
                get(|headers: HeaderMap| async {
                    handle_static_content_with_etag(
                        headers,
                        "text/javascript; charset=utf-8",
                        Some("/stylus.js.map"),
                        &stylus_ui::STYLUS_JAVASCRIPT,
                    )
                }),
            )
            .route(
                "/stylus.css",
                get(|headers: HeaderMap| async {
                    handle_static_content_with_etag(
                        headers,
                        "text/css; charset=utf-8",
                        None,
                        &stylus_ui::STYLUS_CSS,
                    )
                }),
            )
            .route("/stylus.js.map", get(|| async {
                (
                    StatusCode::OK,
                    [
                        ("Content-Type", "application/json"),
                        ("Cache-Control", "no-cache"),
                        ("Content-Encoding", "gzip"),
                    ],
                    stylus_ui::STYLUS_JAVASCRIPT_MAP,
                )
            }));
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
