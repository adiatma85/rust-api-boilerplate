use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{state::AppState, usecase::user::CreateUserParams};

// 1. The Request DTO
#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

// 2. The Handler
#[utoipa::path(
    post,
    path = "/register",
    tag = "Auth",
    request_body = RegisterRequest,
    responses(
        (status = StatusCode::CREATED, description = "User created successfully", body = String),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    )
)]
pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Map HTTP Request -> Usecase Params
    let params = CreateUserParams {
        name: payload.name,
        email: payload.email,
        password: payload.password,
    };

    // Call the Usecase (Logic Layer)
    // We use 'state.user_usecase' which is the Arc<UserUsecase> we set up in main.rs
    match state.user_usecase.create_user(params).await {
        Ok(user_id) => (
            StatusCode::CREATED,
            format!("User created with ID: {}", user_id),
        ),
        Err(e) => {
            // In a real app, you might distinguish between "Duplicate Email" (409)
            // vs "Database Error" (500) here.
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create user: {}", e),
            )
        }
    }
}

#[utoipa::path(
    post,
    path = "/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = StatusCode::OK, description = "Login successful", body = LoginResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Invalid credentials")
    )
)]
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let params = crate::usecase::auth::LoginParams {
        email: payload.email,
        password: payload.password,
    };

    // Call AuthUsecase
    match state.auth_usecase.login(params).await {
        Ok(token) => {
            let resp = LoginResponse { token };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => {
            // Simple error handling: generic 401 for security
            (StatusCode::UNAUTHORIZED, e).into_response()
        }
    }
}
