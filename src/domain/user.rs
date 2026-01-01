use async_trait::async_trait;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::{
    domain::helper::fetch_list,
    entity::{
        error::AppError,
        user::{self, CreateUserDomParam, UserDomParam},
    },
};

// 1. Defining the interface or trait for the Domain
// "Send + Sync" is required so this trait can be shared across threads (Axum requirement)
#[async_trait]
pub trait UserDomainTrait: Send + Sync {
    async fn create(&self, params: CreateUserDomParam) -> Result<i32, String>;
    async fn get_list(&self, params: UserDomParam) -> Result<(Vec<user::Model>, i64), AppError>;
}

// 1. The Struct holding the DB Connection
pub struct UserDomainImpl {
    db: DatabaseConnection,
    // In below, we can add many of the things in here such as redis cache, logs, or anything else
    // But it will big chance that this actually need a redis instead
}

pub struct InitParam {
    pub db: DatabaseConnection,
}

pub fn init(param: InitParam) -> impl UserDomainTrait {
    UserDomainImpl { db: param.db }
}

// 2. Implement the Trait
#[async_trait]
impl UserDomainTrait for UserDomainImpl {
    async fn create(&self, params: CreateUserDomParam) -> Result<i32, String> {
        let new_user = user::ActiveModel {
            name: Set(params.name),
            email: Set(params.email),
            hashed_password: Set(params.hashed_password),
            status: Set(1),
            ..Default::default()
        };

        let result = user::Entity::insert(new_user)
            .exec(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.last_insert_id)
    }

    async fn get_list(&self, params: UserDomParam) -> Result<(Vec<user::Model>, i64), AppError> {
        // You can use '?' here because fetch_list returns DbErr,
        // and we implemented From<DbErr> for AppError
        let (users, total) = fetch_list::<user::Entity, _, _>(&self.db, params, 0, 10)
            .await
            .map_err(AppError::from)?;

        Ok((users, total as i64))
    }
}
