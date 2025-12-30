use axum::{
    Extension, // Required to get the Auth Token Claims
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    state::AppState,
    usecase::{
        auth::Claims,
        card::{CreateCardParams, UpdateCardParams},
    },
}; // Import Claims to read the user_id

// --- Request DTOs ---
#[derive(Deserialize, ToSchema)]
pub struct CreateCardRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateCardStatusRequest {
    pub status: String,
}

// --- Handlers ---

/// Create a new card
#[utoipa::path(
    post,
    path = "/cards",
    tag = "Cards",
    request_body = CreateCardRequest,
    // This line links to the SecuritySchema defined in your main OpenApi struct
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = StatusCode::CREATED, description = "Card created successfully", body = String),
        (status = 500, description = "Internal Server Error", body = String)
    )
)]
pub async fn create_card_handler(
    State(state): State<AppState>,
    Extension(user): Extension<Claims>, // <--- Securely get User ID from Token
    Json(payload): Json<CreateCardRequest>,
) -> impl IntoResponse {
    let params = CreateCardParams {
        title: payload.title,
        description: payload.description,
        user_id: user.user_id, // We use the ID from the valid Token
    };

    match state.card_usecase.create_card(params).await {
        Ok(id) => (StatusCode::CREATED, format!("Card created with ID: {}", id)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

/// Update card status
#[utoipa::path(
    patch, // Using PATCH is standard for partial updates
    path = "/cards/{id}/status",
    tag = "Cards",
    request_body = UpdateCardStatusRequest,
    // Explicitly document the Path ID variable
    params(
        ("id" = i32, Path, description = "The ID of the card to update")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = StatusCode::OK, description = "Card status updated"),
        (status = StatusCode::NOT_FOUND, description = "Card not found", body = String),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String)
    )
)]
pub async fn update_card_status_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>, // Extract ID from URL (/cards/:id/status)
    Json(payload): Json<UpdateCardStatusRequest>,
) -> impl IntoResponse {
    let params = UpdateCardParams {
        id,
        status: payload.status,
    };

    match state.card_usecase.update_card_status(params).await {
        Ok(_) => (StatusCode::OK, "Card status updated").into_response(),
        Err(e) => {
            let status = if e == "Card not found" {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, e).into_response()
        }
    }
}

/// Delete a card
#[utoipa::path(
    delete,
    path = "/cards/{id}",
    tag = "Cards",
    params(
        ("id" = i32, Path, description = "The ID of the card to delete")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = StatusCode::OK, description = "Card deleted successfully"),
        (status = StatusCode::NOT_FOUND, description = "Card not found", body = String),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String)
    )
)]
pub async fn delete_card_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.card_usecase.delete_card(id).await {
        Ok(_) => (StatusCode::OK, "Card deleted successfully").into_response(),
        Err(e) => {
            let status = if e == "Card not found" {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, e).into_response()
        }
    }
}
