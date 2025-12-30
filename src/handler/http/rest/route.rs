use axum::{
    Router, middleware as axum_middleware,
    routing::{delete, get, patch, post},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    handler::http::{
        doc::api_doc,
        middleware::{auth::auth_middleware, context::context_middleware},
        rest::{
            card::{create_card_handler, delete_card_handler, update_card_status_handler},
            user::{create_user_handler, login_handler},
            util::ping_handler,
        },
    },
    state::AppState,
};

pub fn init_route(state: AppState) -> Router {
    // Open API
    let api_doc = api_doc::ApiDoc::openapi();

    // Routes
    let v1_route = v1_routes(state.clone());

    Router::new()
        // This serves the UI at http://localhost:PORT/swagger-ui
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api_doc))
        // Mount the v1 router under the "/v1" prefix
        .nest("/api/v1", v1_route)
        .layer(axum_middleware::from_fn(context_middleware))
        .with_state(state)
}

// --- 2. V1 Route Definition ---
// This function builds the specific router for V1 logic
fn v1_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Public V1 Routes
        .route("/ping", get(ping_handler))
        .route("/login", post(login_handler))
        .route("/register", post(create_user_handler))
        // Merge the protected routes (Separated for clarity)
        .merge(v1_protected_routes(state))
}

// --- 3. V1 Protected Routes Helper ---
// Isolating the protected logic makes the main v1 function much easier to read
fn v1_protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/cards", post(create_card_handler))
        .route("/cards/{id}/status", patch(update_card_status_handler))
        .route("/cards/{id}", delete(delete_card_handler))
        // Apply the auth middleware only to this sub-router
        .route_layer(axum_middleware::from_fn_with_state(
            state, // State passed explicitly for middleware construction
            auth_middleware,
        ))
}
