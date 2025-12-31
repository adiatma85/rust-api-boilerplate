mod config;
mod domain;
mod entity;
mod handler;
mod state;
mod usecase;

use std::net::SocketAddr;

use sea_orm::Database;

use crate::config::app_settings::AppSettings;

const CONFIG_PATH: &str = "./etc/cfg/conf.json";

#[tokio::main]
async fn main() {
    let app_settings = match AppSettings::new(CONFIG_PATH) {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("Failed to load app settings: {}", err);
            std::process::exit(1);
        }
    };

    // 2. Connect to Database (SeaORM)
    // SeaORM automatically handles connection pooling (like sql.DB in Go)
    let db = Database::connect(app_settings.database_url())
        .await
        .unwrap();

    println!("✅ Database connected successfully");

    // 3. Create the State
    let state = state::AppState::new(db, app_settings.creds.jwt_secret);

    // 4. Build Application with State
    let app = crate::handler::http::rest::init_route(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], app_settings.app_metadata.port));
    println!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
