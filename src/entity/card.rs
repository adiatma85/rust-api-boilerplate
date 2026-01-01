use sea_orm::{Condition, IntoActiveModel, entity::prelude::*};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::{Filterable, Updatable, card};

// --- Model for the Database ---

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "card")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    #[sea_orm(column_type = "Text", nullable)] // Maps to SQL TEXT and Option<String>
    pub description: Option<String>,
    pub card_status: String, // Storing "todo", "in-progress", "done" as simple strings
    pub user_id: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // We will define the relationship to User later if we need to do Join queries.
    // For now, leaving this empty is perfectly fine for basic CRUD.
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

// Optional: If you want to use the relation in queries
impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// --- Structs that used in Domain ---

pub struct CreateCardDomParam {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Default, Debug)]
pub struct CardDomParam {
    pub id: Option<i32>,
    // --- Other attributes is not added in here for now
}

#[derive(Default, Debug)]
#[allow(dead_code)]
pub struct CardDomUpdateParam {
    pub title: Option<String>,
    pub description: Option<String>,
    pub card_status: Option<String>,
}

// --- Structs that used in Usecase ---
#[allow(dead_code)]
pub struct CreateCardUseParam {
    pub title: String,
    pub description: Option<String>,
    pub user_id: i32,
}

#[derive(Default, Debug)]
pub struct CardUseParam {
    pub id: Option<i32>,
}

// This will be changed
#[derive(Default, Debug)]
pub struct UpdateCardUseParam {
    pub id: i32,
    pub status: String,
}

// --- Public structs for the Request and Response (DTO) ---

#[derive(Deserialize, ToSchema)]
pub struct CreateCardRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateCardStatusRequest {
    pub status: String,
}

// --- Helper implementation is in below ---

// Map the Param to the DB ActiveModel
// This is boilerplate, but you write it once and it guarantees type safety.
impl IntoActiveModel<ActiveModel> for CreateCardDomParam {
    fn into_active_model(self) -> ActiveModel {
        ActiveModel {
            title: sea_orm::Set(self.title),
            description: sea_orm::Set(self.description),
            // Don't set ID here, the DB handles auto-increment
            ..Default::default()
        }
    }
}

impl Filterable for CardDomParam {
    fn to_condition(&self) -> sea_orm::Condition {
        // Condition::all() is equivalent to "WHERE 1=1" (AND logic)
        // Condition::any() is equivalent to OR logic
        let mut condition = Condition::all();

        // 1. Exact Match (Equivalent to your standard tag handling)
        if let Some(id) = self.id {
            condition = condition.add(Column::Id.eq(id));
        }

        condition
    }
}

// Implement the to make the card updatable
impl Updatable<card::ActiveModel> for CardDomUpdateParam {
    fn apply_to(self, mut active_model: card::ActiveModel) -> card::ActiveModel {
        if let Some(title) = self.title {
            active_model.title = sea_orm::Set(title);
        }

        if let Some(description) = self.description {
            active_model.description = sea_orm::Set(Some(description));
        }

        active_model
    }
}

// Implement this to make the card also updateable BUT in many case
impl IntoActiveModel<card::ActiveModel> for CardDomUpdateParam {
    fn into_active_model(self) -> card::ActiveModel {
        let mut am: card::ActiveModel = Default::default();

        if let Some(title) = self.title {
            am.title = sea_orm::Set(title);
        }
        if let Some(description) = self.description {
            am.description = sea_orm::Set(Some(description));
        }

        am
    }
}
