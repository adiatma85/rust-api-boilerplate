use axum::Json;

use crate::entity;

#[utoipa::path(
    get,
    path = "/ping",
    tag = "Util",
    responses(
        (status = StatusCode::OK, description = "Login successful"),
    )
)]
pub async fn health_check_handler() -> Json<entity::util::PingResponse> {
    const MESSAGE: &str = "Simple REST-API in Rust";

    let response = entity::util::PingResponse {
        message: MESSAGE.to_string(),
        status: "success".to_string(),
    };

    Json(response)
}
