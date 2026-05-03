use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{ActiveValue::Set, DatabaseConnection};
use tracing::{debug, error, info};

use crate::business::{
    domain::helper::{create_one, fetch_list, fetch_one},
    entity::{
        error::AppError,
        response::Pagination,
        user::{self, CreateUserDomParam, UserDomParam},
    },
};

// 1. Defining the interface or trait for the Domain
// "Send + Sync" is required so this trait can be shared across threads (Axum requirement)
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserDomainTrait: Send + Sync {
    async fn create(&self, params: CreateUserDomParam) -> Result<user::Model, AppError>;
    async fn get_list(
        &self,
        params: UserDomParam,
    ) -> Result<(Vec<user::Model>, Pagination), AppError>;
    async fn get_one(&self, params: UserDomParam) -> Result<user::Model, AppError>;
}

// 1. The Struct holding the DB Connection
pub struct UserDomainImpl {
    db: Arc<DatabaseConnection>,
    // In below, we can add many of the things in here such as redis cache, logs, or anything else
    // But it will big chance that this actually need a redis instead
}

pub struct InitParam {
    pub db: Arc<DatabaseConnection>,
}

pub fn init(param: InitParam) -> impl UserDomainTrait {
    UserDomainImpl { db: param.db }
}

// 2. Implement the Trait
#[async_trait]
impl UserDomainTrait for UserDomainImpl {
    async fn create(&self, params: CreateUserDomParam) -> Result<user::Model, AppError> {
        let email = params.email.clone();
        debug!("create user attempt: email={}", email);

        let new_user = user::ActiveModel {
            name: Set(params.name),
            email: Set(params.email),
            hashed_password: Set(params.hashed_password),
            status: Set(1),
            ..Default::default()
        };

        let result = create_one::<user::Entity, _, _, _>(&self.db, new_user)
            .await
            .map_err(|e| {
                error!("failed to create user: email={}, error={:?}", email, e);
                AppError::from(e)
            })?;

        info!(
            "user created successfully: email={}, id={}",
            result.email, result.id
        );

        Ok(result)
    }

    async fn get_list(
        &self,
        params: UserDomParam,
    ) -> Result<(Vec<user::Model>, Pagination), AppError> {
        debug!("get user list request: {:?}", params);

        let (users, pagination) = fetch_list::<user::Entity, _, _>(&self.db, params)
            .await
            .map_err(|e| {
                error!("failed to get user list: error={:?}", e);
                AppError::from(e)
            })?;

        debug!("user list retrieved: {} users", pagination.current_elements);

        Ok((users, pagination))
    }

    async fn get_one(&self, params: UserDomParam) -> Result<user::Model, AppError> {
        debug!("get user request: {:?}", params);

        let user = fetch_one::<user::Entity, _, _>(&self.db, params)
            .await
            .map_err(|e| {
                error!("failed to get user: error={:?}", e);
                AppError::from(e)
            })?;

        debug!("user retrieved: id={}", user.id);

        Ok(user)
    }
}
