mod api_doc;
mod config;
mod domain;
mod entity;
mod handler;
mod middleware;
mod state;
mod usecase;

use std::net::SocketAddr;

use axum::{
    Router, middleware as axum_middleware,
    routing::{delete, get, patch, post},
};
use sea_orm::Database;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi; // Import the trait

use crate::{
    config::app_settings::AppSettings,
    handler::{
        card::{create_card_handler, delete_card_handler, update_card_status_handler},
        user::{create_user_handler, login_handler},
        util::health_check_handler,
    },
    middleware::auth::auth_middleware,
};

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
    // Ideally, get this from AppSettings too!
    let jwt_secret = "super_secret_key_from_env".to_string();
    let state = state::AppState::new(db, jwt_secret);

    // Open API
    let api_doc = api_doc::ApiDoc::openapi();

    // 4. Build Application with State
    // .with_state(state) injects the DB into every handler that asks for it
    let app = Router::new()
        // This serves the UI at http://localhost:PORT/swagger-ui
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api_doc))
        // Related to util function
        .route("/ping", get(health_check_handler))
        // Related to user
        .route("/login", post(login_handler))
        .route("/register", post(create_user_handler))
        // With middleware
        // --- Protected Routes ---
        // Everything inside this .merge() will run through the middleware
        .merge(
            Router::new()
                .route("/cards", post(create_card_handler))
                .route("/cards/{id}", patch(update_card_status_handler))
                .route("/cards/{id}", delete(delete_card_handler))
                .route_layer(axum_middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                )),
        )
        // Send the state
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], app_settings.port));
    println!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
