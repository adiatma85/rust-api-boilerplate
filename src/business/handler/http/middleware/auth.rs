use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::IntoResponse,
};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::{business::entity::auth::Claims, state::AppState}; // Import your Claims struct

// Custom Header extraction helper
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Get the "Authorization" header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing Authorization Header".to_string(),
        ))?;

    // 2. Strip "Bearer " prefix
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid Token Format".to_string()))?;

    // 3. Decode JWT
    // NOTE: You need to access the secret.
    // If you moved AuthUsecase logic, ensure you can access the 'jwt_secret' here.
    // For now, assuming we grab it from state.auth_usecase (you might need to make secret pub or add a getter)
    let secret = &state.jwt_secret;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Token".to_string()))?;

    // 4. Inject the User Data (Claims) into the Request
    // This allows handlers downstream to say: Extension(claims): Extension<Claims>
    req.extensions_mut().insert(token_data.claims);

    // 5. Continue to the next handler
    Ok(next.run(req).await)
}
