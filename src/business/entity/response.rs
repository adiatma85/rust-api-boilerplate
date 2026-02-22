use axum::{Json, http::StatusCode};
use chrono::Utc;
use serde::Serialize;

use crate::business::entity::error::AppError;

// 1. The Main Wrapper
// We use <T> to mimic "interface{}" or "any"
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub message: Message,
    pub metadata: Metadata,
    pub data: T,
    // Skip serialization if None (for endpoints without pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
}

// 2. Message Component
#[derive(Serialize)]
pub struct Message {
    pub title: String,
    pub body: String,
}

// 3. Metadata Component
#[derive(Serialize)]
pub struct Metadata {
    pub path: String,
    pub status_code: u16,
    pub status: String,
    pub message: String,
    pub timestamp: String,
    pub request_id: String,
    pub time_elapsed: String,
}

// 4. Pagination Component (Optional)
#[derive(Serialize, Debug, Clone)]
pub struct Pagination {
    pub current_page: u64,
    pub current_elements: u64,
    pub total_pages: u64,
    pub total_elements: u64,
    pub sort_by: Vec<String>,
}

// --- The "Code Pattern" Enum ---
// This replaces your "codes.Compile(code)" logic
#[derive(Debug, Clone)]
pub enum AppCode {
    Success,
    // Created,
    Unauthorized,
    NotFound,
    InternalServerError(String),
    // Add more as needed
}

impl AppCode {
    // Map the enum to your standard strings
    pub fn details(&self) -> (StatusCode, String, String) {
        match self {
            AppCode::Success => (
                StatusCode::OK,
                "OK".to_string(),
                "Request successful".to_string(),
            ),
            // AppCode::Created => (
            //     StatusCode::CREATED,
            //     "Created".to_string(),
            //     "Resource created successfully".to_string(),
            // ),
            AppCode::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized".to_string(),
                "Unauthorized access".to_string(),
            ),
            AppCode::NotFound => (
                StatusCode::NOT_FOUND,
                "Not Found".to_string(),
                "Resource not found".to_string(),
            ),
            AppCode::InternalServerError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
                format!("An internal server error occurred: {}", err),
            ),
        }
    }
}

// The Bridge: Convert Logic Error -> Presentation Code
impl From<AppError> for AppCode {
    fn from(err: AppError) -> Self {
        match err {
            // AppError::Unauthorized => AppCode::Unauthorized,
            AppError::NotFound => AppCode::NotFound,
            // We swallow the internal error string here because AppCode
            // is for the public JSON response
            AppError::InternalServerError(err) => AppCode::InternalServerError(err),
        }
    }
}

impl std::fmt::Display for AppCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppCode::Success => write!(f, "Success"),
            // AppCode::Created => write!(f, "Created"),
            AppCode::NotFound => write!(f, "Not Found"),
            AppCode::Unauthorized => write!(f, "Unauthorized"),
            // AppCode::Forbidden => write!(f, "Forbidden"),
            // AppCode::BadRequest => write!(f, "Bad Request"),
            AppCode::InternalServerError(err) => write!(f, "Internal Server Error. {}", err),
        }
    }
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn new(
        code: AppCode,
        data: T,
        pagination: Option<Pagination>,
        // Contextual info passed from Handler
        path: &str,
        method: &str,
        request_id: &str,
        start_time: std::time::Instant,
    ) -> (StatusCode, Json<Self>) {
        // 1. Get standardized strings from the Enum
        let (status_code, title, body) = code.details();

        // 2. Calculate Elapsed Time
        let elapsed = start_time.elapsed();
        let time_elapsed = format!("{:?}", elapsed); // e.g., "2.4ms"

        // 3. Build Metadata
        let metadata = Metadata {
            path: path.to_string(),
            status_code: status_code.as_u16(),
            status: status_code
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            message: format!("{} {} [{}] {}", method, path, status_code.as_u16(), title),
            timestamp: Utc::now().to_rfc3339(),
            request_id: request_id.to_string(),
            time_elapsed,
        };

        // 4. Return tuple (Status, JSON Body)
        // This tuple automatically implements IntoResponse in Axum
        (
            status_code,
            Json(ApiResponse {
                message: Message { title, body },
                metadata,
                data,
                pagination,
            }),
        )
    }
}
