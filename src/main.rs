use std::convert::Infallible;
use warp::Filter;

mod config;
mod interpolate;
mod monitor;
mod status;
mod worker;

async fn css_request() -> Result<String, Infallible> {
    Ok("css".to_owned())
}

async fn status_request() -> Result<String, Infallible> {
    Ok("status".to_owned())
}

#[tokio::main]
async fn main() -> () {
    let file = std::env::args()
        .collect::<Vec<_>>()
        .get(1)
        .expect("Usage: stylus [path-to-config.yaml]")
        .clone();
    let config = config::parse_config(file).expect("Unable to parse configuration");
    println!("{:?}", config);
    let monitor = monitor::Monitor::new(&config);
    println!("{:?}", monitor);

    // style.css for formatting
    let style = warp::path("style.css").and_then(css_request);
    // status.json for advanced integrations
    let status = warp::path("status.json").and_then(status_request);
    // static pages
    let r#static = warp::fs::dir("monitor.d");

    let routes = warp::get().and(style.or(status).or(r#static));

    warp::serve(routes)
        .run(([127, 0, 0, 1], config.server.port))
        .await;
}
