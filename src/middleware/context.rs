use std::time::Instant;

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

use crate::types::response::{ApiResponse, AppCode};

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
    pub fn success<T>(self, code: AppCode, data: T) -> Response
    where
        T: Serialize,
    {
        ApiResponse::new(
            code,
            data,
            None, // Assuming no pagination for standard success
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

pub async fn context_middleware(mut req: Request, next: Next) -> Response {
    // 1. Generate ID and Start Timer
    let request_id = Uuid::new_v4().to_string();
    let start_time = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let ctx = RequestContext {
        request_id,
        start_time,
        path,
        method,
    };

    // 2. Insert into Request Extensions
    // This allows the Handler to retrieve it later using "Extension<RequestContext>"
    req.extensions_mut().insert(ctx);

    // 3. Call the next handler
    next.run(req).await
}
