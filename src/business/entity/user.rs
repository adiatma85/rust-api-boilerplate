use sea_orm::{Condition, IntoActiveModel, entity::prelude::*};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::business::entity::{
    Filterable,
    util::{Paginatable, PaginationParams},
};

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

// --- Structs that used in domain ---

pub struct CreateUserDomParam {
    pub name: String,
    pub email: String,
    pub hashed_password: String,
}

#[derive(Default, Debug, Deserialize, IntoParams)]
pub struct UserDomParam {
    pub id: Option<i32>,
    pub ids: Option<Vec<i32>>,
    pub email_eq: Option<String>,

    // Flatten allows ?page=1 to work at the root level
    // instead of ?pagination[page]=1
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

// --- Structs that used in the usecase ---

pub struct CreateUserUseParam {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct UserUseResponse {
    pub id: Option<i32>,
    pub email: Option<String>,
}

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

// --- Helper Functions if needed ---

impl Filterable for UserDomParam {
    // This function replaces your 'qb.Build' logic
    fn to_condition(&self) -> Condition {
        // Condition::all() is equivalent to "WHERE 1=1" (AND logic)
        // Condition::any() is equivalent to OR logic
        let mut condition = Condition::all();

        // 1. Exact Match (Equivalent to your standard tag handling)
        if let Some(id) = self.id {
            condition = condition.add(Column::Id.eq(id));
        }

        // 2. Exact Match (Equivalent to your standard tag handling)
        if let Some(ids) = &self.ids {
            condition = condition.add(Column::Id.is_in(ids.iter().copied()));
        }

        // 3 Exact match for the email
        if let Some(email) = &self.email_eq {
            condition = condition.add(Column::Email.eq(email.as_str()));
        }

        condition
    }
}

// Implement the Trait
impl Paginatable for UserDomParam {
    fn get_page(&self) -> u64 {
        // Default to page 0 or 1 depending on your preference (SeaOrm uses 0-index usually)
        self.pagination.page.unwrap_or(0)
    }

    fn get_limit(&self) -> u64 {
        // Default to 10 if not specified
        self.pagination.limit.unwrap_or(10)
    }

    fn is_limit_disabled(&self) -> bool {
        self.pagination.disable_limit
    }
}

// Map the Param to the DB ActiveModel
// This is boilerplate, but you write it once and it guarantees type safety.
impl IntoActiveModel<ActiveModel> for CreateUserDomParam {
    fn into_active_model(self) -> ActiveModel {
        ActiveModel {
            email: sea_orm::Set(self.email),
            name: sea_orm::Set(self.name),
            // Don't set ID here, the DB handles auto-increment
            ..Default::default()
        }
    }
}
