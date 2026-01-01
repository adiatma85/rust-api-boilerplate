use serde::{Deserialize, Serialize};

// Struct that used in the Usecase Level

// 1. JWT Claims Struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // Subject (Email)
    pub user_id: i32,
    pub exp: usize, // Expiration
    pub iat: usize, // Issued At
}

pub struct LoginParams {
    pub email: String,
    pub password: String,
}
