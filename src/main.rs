mod config;
mod entity;
mod usecase;

use std::net::SocketAddr;

use axum::{Router, routing::get};

use crate::config::app_settings::AppSettings;
use crate::usecase::util::health_check_handler;

const CONFIG_PATH: &str = "./cfg/config.json";

#[tokio::main]
async fn main() {
    let app_settings = AppSettings::new(CONFIG_PATH).unwrap();

    println!("{}", app_settings.port);

    // Build our application with a single route
    let app = Router::new().route("/ping", get(health_check_handler));

    // Define the address to listen on
    let addr = SocketAddr::from(([127, 0, 0, 1], app_settings.port));
    println!("🚀 Server listening on http://{}", addr);

    // Run the server
    // axum::serve is the modern way to run the router
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
