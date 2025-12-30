use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// --- Model for the Database ---

// 1. Define the Table Name
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)] // Tells SeaORM this is the PK
    pub id: i32, // Rust's i32 maps to SQL INT
    pub email: String,
    pub name: String,
    pub hashed_password: String,
    pub status: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

// 2. Define Relationships (We have none yet, so it's empty)
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

// 3. Define ActiveModel
// This trait enables the "Active Record" behavior (save, delete, etc.)
impl ActiveModelBehavior for ActiveModel {}

// --- Public structs for the Request and Response (DTO) ---

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
