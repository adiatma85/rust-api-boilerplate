use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set};

use crate::entity::card;

// --- Trait and Usecase Struct

// --- DTOs (Parameters) ---
pub struct CreateCardParams {
    pub title: String,
    pub description: Option<String>,
    pub user_id: i32,
}

pub struct UpdateCardParams {
    pub id: i32,
    pub status: String,
}

// --- The Usecase Struct ---
pub struct CardUsecase {
    db: DatabaseConnection,
}

pub struct CardUseInitParam {
    pub db: DatabaseConnection,
}

// --- Implementation ---

impl CardUsecase {
    pub fn new(params: CardUseInitParam) -> Self {
        Self { db: params.db }
    }

    // 1. Create Card
    pub async fn create_card(&self, params: CreateCardParams) -> Result<i32, String> {
        let new_card = card::ActiveModel {
            title: Set(params.title),
            description: Set(params.description),
            user_id: Set(params.user_id),
            card_status: Set("todo".to_string()), // Default status
            ..Default::default()
        };

        let result = card::Entity::insert(new_card)
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.last_insert_id)
    }

    // 2. Update Status
    pub async fn update_card_status(&self, params: UpdateCardParams) -> Result<(), String> {
        // Find the card first
        let card_model = card::Entity::find_by_id(params.id)
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Card not found".to_string())?;

        // Convert to ActiveModel to update it
        let mut active_card = card_model.into_active_model();
        active_card.card_status = Set(params.status);

        active_card
            .update(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    // 3. Delete Card
    pub async fn delete_card(&self, id: i32) -> Result<(), String> {
        let result = card::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        if result.rows_affected == 0 {
            return Err("Card not found".to_string());
        }

        Ok(())
    }
}
