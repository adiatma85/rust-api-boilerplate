use axum::{
    Extension,
    extract::{Json, State},
    response::IntoResponse,
};

use crate::{
    business::{
        entity::{
            auth,
            response::AppCode,
            user::{
                CreateUserUseParam, LoginRequest, LoginResponse, RegisterRequest, UserDomParam,
                UserUseResponse,
            },
        },
        handler::http::middleware::context::RequestContext,
    },
    state::AppState,
};

// 2. The Handler
#[utoipa::path(
    post,
    path = "/api/v1/register",
    tag = "Auth",
    request_body = RegisterRequest,
    responses(
        (status = StatusCode::CREATED, description = "User created successfully", body = String),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    )
)]
pub async fn create_user_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Map HTTP Request -> Usecase Params
    let params = CreateUserUseParam {
        name: payload.name,
        email: payload.email,
        password: payload.password,
    };

    // Call the Usecase (Logic Layer)
    // We use 'state.user_usecase' which is the Arc<UserUsecase> we set up in main.rs
    match state.usecase.user.create_user(params).await {
        Ok(user) => {
            let user_response = UserUseResponse {
                id: Some(user.id),
                email: Some(user.email),
            };
            ctx.success(AppCode::Success, user_response, None)
        }
        Err(e) => ctx.error(e.clone(), format!("Failed to create user: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = StatusCode::OK, description = "Login successful", body = LoginResponse),
        (status = StatusCode::UNAUTHORIZED, description = "Invalid credentials")
    )
)]
pub async fn login_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let params = auth::LoginParams {
        email: payload.email,
        password: payload.password,
    };

    // Call AuthUsecase
    match state.usecase.auth.login(params).await {
        Ok(token) => {
            let resp = LoginResponse { token };
            ctx.success(AppCode::Success, resp, None)
        }
        Err(e) => ctx.error(AppCode::Unauthorized, e.to_string()),
    }
}

// This is just for the testing purpose
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "List users successfully", body = Vec<UserUseResponse>),
        (status = 500, description = "Internal Server Error", body = String)
    )
)]
pub async fn get_user_list_handler(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    let params = UserDomParam {
        ..Default::default()
    };

    // Call UserUsecase
    match state.usecase.user.get_list(params).await {
        Ok((users, pagination)) => {
            let resp = users;
            ctx.success(AppCode::Success, resp, Some(pagination))
        }
        Err(e) => ctx.error(AppCode::InternalServerError(e.to_string()), e.to_string()),
    }
}
