use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub message: String,
    pub status: String,
}
