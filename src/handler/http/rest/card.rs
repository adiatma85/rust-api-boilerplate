use axum::{
    Extension, // Required to get the Auth Token Claims
    extract::{Json, Path, State},
    response::IntoResponse,
};

use crate::{
    entity::{
        auth::Claims,
        card::{
            CardDomParam, CreateCardRequest, CreateCardUseParam, UpdateCardStatusRequest,
            UpdateCardUseParam,
        },
        response::AppCode,
    },
    handler::http::middleware::context::RequestContext,
    state::AppState,
}; // Import Claims to read the user_id

// --- Handlers ---

/// Create a new card
#[utoipa::path(
    post,
    path = "/api/v1/cards",
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
    Extension(user): Extension<Claims>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateCardRequest>,
) -> impl IntoResponse {
    let params = CreateCardUseParam {
        title: payload.title,
        description: payload.description,
        user_id: user.user_id, // We use the ID from the valid Token
    };

    match state.usecase.card.create(params).await {
        Ok(model) => ctx.success(
            AppCode::Success,
            format!("Card created with model: {:?}", model),
            None,
        ),
        Err(e) => ctx.error(e.clone(), e.to_string()),
    }
}

/// Update card status
#[utoipa::path(
    patch, // Using PATCH is standard for partial updates
    path = "/api/v1/cards/{id}/status",
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
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateCardStatusRequest>,
) -> impl IntoResponse {
    let params = UpdateCardUseParam {
        id,
        status: payload.status,
    };

    match state.usecase.card.update_one(params).await {
        Ok(_) => ctx.success(AppCode::Success, "Card status updated", None),
        Err(e) => ctx.error(e.clone(), e.to_string()),
    }
}

/// Delete a card
#[utoipa::path(
    delete,
    path = "/api/v1/cards/{id}",
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
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    match state
        .usecase
        .card
        .delete_one(CardDomParam {
            id: Some(id),
            ..Default::default()
        })
        .await
    {
        Ok(_) => ctx.success(AppCode::Success, "Card deleted successfully", None),
        Err(e) => ctx.error(e.clone(), e.to_string()),
    }
}
