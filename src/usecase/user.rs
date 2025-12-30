use sea_orm::{DatabaseConnection, EntityTrait, Set};

use crate::{entity::user, usecase::auth::AuthUsecase}; // Import Auth to use hash_password

pub struct CreateUserParams {
    pub name: String,
    pub email: String,
    pub password: String,
}

pub struct UserUsecase {
    db: DatabaseConnection,
}

impl UserUsecase {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_user(&self, params: CreateUserParams) -> Result<i32, String> {
        // 1. Hash the password using the helper we just wrote
        // Notice we call AuthUsecase::hash_password directly
        let hashed_pwd = AuthUsecase::hash_password(&params.password)?;

        // 2. Prepare Data
        let new_user = user::ActiveModel {
            name: Set(params.name),
            email: Set(params.email),
            hashed_password: Set(hashed_pwd),
            status: Set(1),
            ..Default::default()
        };

        // 3. Save to DB
        let result = user::Entity::insert(new_user)
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.last_insert_id)
    }
}
