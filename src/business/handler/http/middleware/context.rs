use std::time::Instant;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::{Instrument, field, info_span};
use uuid::Uuid;

use crate::{
    business::entity::response::{ApiResponse, AppCode, Pagination},
    state::AppState,
};

// Define a struct to hold our "Request Context"
#[derive(Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub start_time: Instant,
    pub path: String,
    pub method: String,
}

impl RequestContext {
    // Helper 1: Success Response
    pub fn success<T>(self, code: AppCode, data: T, pagination: Option<Pagination>) -> Response
    where
        T: Serialize,
    {
        ApiResponse::new(
            code,
            data,
            pagination,
            &self.path,
            &self.method,
            &self.request_id,
            self.start_time,
        )
        .into_response()
    }

    // Helper 2: Error Response (Generic)
    // We use "()" as the data type because errors usually don't have data payload
    pub fn error(self, code: AppCode, message: String) -> Response {
        // You might need to adjust ApiResponse to accept String data or handle errors specifically
        // But assuming your ApiResponse is flexible:
        ApiResponse::new(
            code,
            message, // or however you structure error bodies
            None,
            &self.path,
            &self.method,
            &self.request_id,
            self.start_time,
        )
        .into_response()
    }
}

pub async fn context_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let start_time = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let ctx = RequestContext {
        request_id: request_id.clone(),
        start_time,
        path,
        method,
    };

    req.extensions_mut().insert(ctx);

    let span = info_span!(
        "request",
        request_id = %request_id,
        user_agent = %user_agent,
        user_id = field::Empty,
        service_version = %state.service_version,
    );

    next.run(req).instrument(span).await
}
