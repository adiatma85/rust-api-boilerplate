mod config;
mod domain;
mod entity;
mod handler;
mod helper;
mod state;
mod usecase;

use std::net::SocketAddr;

use sea_orm::{Database, DatabaseConnection};
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

    println!("✅ Database connected successfully");

    // Initialize the domain layer
    let domain = domain::init(domain::InitParam { db: db.clone() });

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

    clean_up(CleanUpResources { db }).await;
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

// Clean up resources that we run in here

struct CleanUpResources {
    db: DatabaseConnection,
}

async fn clean_up(params: CleanUpResources) {
    // --- CLEANUP PHASE ---
    // The code reaches here ONLY after the shutdown signal is received
    // and active HTTP requests have finished.

    println!("Creating graceful shutdown...");

    // Explicitly close the database connection
    // This ensures the connection pool is drained and closed properly
    if let Err(err) = params.db.close().await {
        eprintln!("Error closing database: {}", err);
    } else {
        println!("✅ Database connection closed successfully");
    }

    println!("👋 Bye!");
}
