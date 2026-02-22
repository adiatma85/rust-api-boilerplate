use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::business::{
    domain::helper::{create_one, delete_one, fetch_list, fetch_one, update_many, update_one},
    entity::{
        card::{self, CreateCardDomParam},
        error::AppError,
        response::Pagination,
    },
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CardDomainTrait: Send + Sync {
    async fn create(&self, params: CreateCardDomParam) -> Result<card::Model, AppError>;

    async fn get_list(
        &self,
        params: card::CardDomParam,
    ) -> Result<(Vec<card::Model>, Pagination), AppError>;

    async fn get(&self, params: card::CardDomParam) -> Result<card::Model, AppError>;

    async fn update_one(
        &self,
        params: card::CardDomParam,
        data: card::CardDomUpdateParam,
    ) -> Result<card::Model, AppError>;

    #[allow(dead_code)]
    async fn update_many(
        &self,
        params: card::CardDomParam,
        data: card::CardDomUpdateParam,
    ) -> Result<u64, AppError>;

    async fn delete_one(&self, param: card::CardDomParam) -> Result<card::Model, AppError>;
}

pub struct CardDomainImpl {
    db: Arc<DatabaseConnection>,
}

pub struct InitParam {
    pub db: Arc<DatabaseConnection>,
}

pub fn init(param: InitParam) -> impl CardDomainTrait {
    // Initialize the CardDomain with the provided parameters
    CardDomainImpl { db: param.db }
}

#[async_trait]
impl CardDomainTrait for CardDomainImpl {
    async fn create(&self, params: CreateCardDomParam) -> Result<card::Model, AppError> {
        let created_value = create_one::<card::Entity, _, _, _>(&self.db, params)
            .await
            .map_err(AppError::from)?;

        Ok(created_value)
    }

    // For now it's just a total, we will change it in the future
    // It will change to the get_list_2 function in the future
    async fn get_list(
        &self,
        params: card::CardDomParam,
    ) -> Result<(Vec<card::Model>, Pagination), AppError> {
        let (cards, pagination) = fetch_list::<card::Entity, _, _>(&self.db, params)
            .await
            .map_err(AppError::from)?;

        Ok((cards, pagination))
    }

    async fn get(&self, params: card::CardDomParam) -> Result<card::Model, AppError> {
        let card = fetch_one::<card::Entity, _, _>(&self.db, params)
            .await
            .map_err(AppError::from)?;

        Ok(card)
    }

    async fn update_one(
        &self,
        params: card::CardDomParam,
        data: card::CardDomUpdateParam,
    ) -> Result<card::Model, AppError> {
        let updated_value = update_one::<card::Entity, _, _, _, _>(&self.db, params, data)
            .await
            .map_err(AppError::from)?;

        Ok(updated_value)
    }

    async fn update_many(
        &self,
        params: card::CardDomParam,
        data: card::CardDomUpdateParam,
    ) -> Result<u64, AppError> {
        let updated_count = update_many(&self.db, params, data)
            .await
            .map_err(AppError::from)?;

        Ok(updated_count)
    }

    async fn delete_one(&self, param: card::CardDomParam) -> Result<card::Model, AppError> {
        let result = delete_one(&self.db, param).await.map_err(AppError::from)?;

        Ok(result)
    }
}
