use axum::{
    Extension,
    extract::{Json, State},
    response::IntoResponse,
};

use crate::{
    entity::{
        response::AppCode,
        user::{
            CreateUserUseParam, LoginRequest, LoginResponse, RegisterRequest, UserDomParam,
            UserUseResponse,
        },
    },
    handler::http::middleware::context::RequestContext,
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
    match state.user_usecase.create_user(params).await {
        Ok(user_id) => ctx.success(
            AppCode::Success,
            format!("User created successfully with ID {}", user_id),
        ),
        Err(e) => ctx.error(
            AppCode::InternalServerError(e.clone()),
            format!("Failed to create user: {}", e),
        ),
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
    let params = crate::usecase::auth::LoginParams {
        email: payload.email,
        password: payload.password,
    };

    // Call AuthUsecase
    match state.auth_usecase.login(params).await {
        Ok(token) => {
            let resp = LoginResponse { token };
            ctx.success(AppCode::Success, resp)
        }
        Err(e) => ctx.error(AppCode::Unauthorized, e.to_string()),
    }
}

// This is just for the testing purpose
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "User",
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
        id: Some(2),
        email_eq: None,
    };

    // Call UserUsecase
    match state.user_usecase.get_list_user(params).await {
        Ok((users, total)) => {
            println!("The total is: {}", total);
            let resp = users;
            println!("The users are: {:?}", &resp);
            ctx.success(AppCode::Success, resp)
        }
        Err(e) => ctx.error(AppCode::InternalServerError(e.to_string()), e.to_string()),
    }
}
