mod config;
mod domain;
mod entity;
mod usecase;

use std::net::SocketAddr;

use axum::{Router, routing::get};
use sea_orm::{Database, DatabaseConnection};

use crate::{config::app_settings::AppSettings, usecase::util::health_check_handler};

const CONFIG_PATH: &str = "./etc/cfg/conf.json";

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    let app_settings = match AppSettings::new(CONFIG_PATH) {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("Failed to load app settings: {}", err);
            std::process::exit(1);
        }
    };

    println!("{}", app_settings.port);

    // 2. Connect to Database (SeaORM)
    // SeaORM automatically handles connection pooling (like sql.DB in Go)
    let db = Database::connect(app_settings.database_url())
        .await
        .unwrap();

    println!("✅ Database connected successfully");

    // 3. Create the State
    let state = AppState { db };

    // 4. Build Application with State
    // .with_state(state) injects the DB into every handler that asks for it
    let app = Router::new()
        .route("/ping", get(health_check_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], app_settings.port));
    println!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
