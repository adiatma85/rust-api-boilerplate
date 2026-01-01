mod config;
mod domain;
mod entity;
mod handler;
mod state;
mod usecase;

use std::net::SocketAddr;

use sea_orm::Database;

use crate::{config::app_settings::AppSettings, handler::http::rest};

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

    // Initialize the domain layer
    let domain = domain::init(domain::InitParam { db: db.clone() });

    // Initialize the usecase layer
    let usecase = usecase::init(usecase::InitParam {
        db: db.clone(),
        domain,
        jwt_secret: app_settings.creds.jwt_secret.clone(),
    });

    // 3. Create the State
    let state = state::AppState::new(state::AppStateInitParam {
        secret_key: app_settings.creds.jwt_secret,
        usecase,
    });

    // 4. Build Application with State
    let app = rest::init_route(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], app_settings.app_metadata.port));
    println!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
