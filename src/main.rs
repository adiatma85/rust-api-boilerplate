mod config;
mod domain;
mod entity;
mod handler;
mod helper;
mod state;
mod usecase;

use std::{net::SocketAddr, sync::Arc};

use sea_orm::Database;
use tokio::signal;

use crate::{config::app_settings::AppSettings, handler::http::rest};

const CONFIG_PATH: &str = "./etc/cfg/conf.json";

// We need to search about envx

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

    let db_conn = Arc::new(db);

    println!("✅ Database connected successfully");

    // Initialize the domain layer
    let domain = domain::init(domain::InitParam {
        db: db_conn.clone(),
    });

    // Initialize the usecase layer
    let usecase = usecase::init(usecase::InitParam {
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
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

// This is the function to make the application shutdown gracefully
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("\n🛑 Signal received, stopping web server...");
}
