// use axum::Json;
use axum::{
    extract::Extension, // Import Extension
    response::IntoResponse,
};

// use crate::state::AppState; // Import our context
use crate::{
    business::entity::response::AppCode,
    // handler::user::RegisterRequest,
    business::handler::http::middleware::context::RequestContext,
};

#[utoipa::path(
    get,
    path = "/api/v1/ping",
    tag = "Util",
    responses(
        (status = StatusCode::OK, description = "Login successful"),
    )
)]
pub async fn ping_handler(Extension(ctx): Extension<RequestContext>) -> impl IntoResponse {
    ctx.success(AppCode::Success, "PONG!", None)
}
