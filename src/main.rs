mod business;
mod config;
mod helper;
mod state;

use std::{net::SocketAddr, sync::Arc};

use sea_orm::Database;
use tokio::signal;

use crate::{business::handler::http::rest, config::app_settings::AppSettings};

const CONFIG_PATH: &str = "./etc/cfg/conf.json";

// We need to search about envx

#[tokio::main]
async fn main() {
    helper::logger::init_logger();

    let app_settings = match AppSettings::new(CONFIG_PATH) {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("Failed to load app settings: {}", err);
            std::process::exit(1);
        }
    };

    // Print the information about the application
    print_app_info(&app_settings);

    // 2. Connect to Database (SeaORM)
    // SeaORM automatically handles connection pooling (like sql.DB in Go)
    let db = Database::connect(app_settings.database_url())
        .await
        .unwrap();

    let db_conn = Arc::new(db);

    tracing::info!("Database connected successfully");

    // Initialize the domain layer
    let domain = business::domain::init(business::domain::InitParam {
        db: db_conn.clone(),
    });

    // Initialize the usecase layer
    let usecase = business::usecase::init(business::usecase::InitParam {
        domain,
        jwt_secret: app_settings.creds.jwt_secret.clone(),
    });

    // 3. Create the State
    let state = state::AppState::new(state::AppStateInitParam {
        secret_key: app_settings.creds.jwt_secret,
        service_version: app_settings.app_metadata.version.clone(),
        usecase,
    });

    // 4. Build Application with State
    let app = rest::init_route(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], app_settings.app_metadata.port));
    tracing::info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

fn print_app_info(app_settings: &AppSettings) {
    println!("Application Name: {}", app_settings.app_metadata.name);
    println!("Application Version: {}", app_settings.app_metadata.version);
    println!(
        "Application Description: {}",
        app_settings.app_metadata.description
    );
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

    tracing::info!("Signal received, stopping web server...");
}
