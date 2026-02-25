use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use tracing::{debug, error, info};

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
        let title = params.title.clone();
        let user_id = params.user_id;
        debug!("create card attempt: title={}, user_id={}", title, user_id);

        let created_value = create_one::<card::Entity, _, _, _>(&self.db, params)
            .await
            .map_err(|e| {
                error!(
                    "failed to create card: title={}, user_id={}, error={:?}",
                    title, user_id, e
                );
                AppError::from(e)
            })?;

        info!(
            "card created successfully: id={}, title={}",
            created_value.id, created_value.title
        );

        Ok(created_value)
    }

    // For now it's just a total, we will change it in the future
    // It will change to the get_list_2 function in the future
    async fn get_list(
        &self,
        params: card::CardDomParam,
    ) -> Result<(Vec<card::Model>, Pagination), AppError> {
        debug!("get card list request: {:?}", params);

        let (cards, pagination) = fetch_list::<card::Entity, _, _>(&self.db, params)
            .await
            .map_err(|e| {
                error!("failed to get card list: error={:?}", e);
                AppError::from(e)
            })?;

        debug!("card list retrieved: {} cards", pagination.current_elements);

        Ok((cards, pagination))
    }

    async fn get(&self, params: card::CardDomParam) -> Result<card::Model, AppError> {
        debug!("get card request: {:?}", params);

        let card = fetch_one::<card::Entity, _, _>(&self.db, params)
            .await
            .map_err(|e| {
                error!("failed to get card: error={:?}", e);
                AppError::from(e)
            })?;

        debug!("card retrieved: id={}", card.id);

        Ok(card)
    }

    async fn update_one(
        &self,
        params: card::CardDomParam,
        data: card::CardDomUpdateParam,
    ) -> Result<card::Model, AppError> {
        debug!("update card attempt: params={:?}, data={:?}", params, data);

        let updated_value = update_one::<card::Entity, _, _, _, _>(&self.db, params, data)
            .await
            .map_err(|e| {
                error!("failed to update card: error={:?}", e);
                AppError::from(e)
            })?;

        info!("card updated successfully: id={}", updated_value.id);

        Ok(updated_value)
    }

    async fn update_many(
        &self,
        params: card::CardDomParam,
        data: card::CardDomUpdateParam,
    ) -> Result<u64, AppError> {
        debug!(
            "update many cards attempt: params={:?}, data={:?}",
            params, data
        );

        let updated_count = update_many(&self.db, params, data).await.map_err(|e| {
            error!("failed to update many cards: error={:?}", e);
            AppError::from(e)
        })?;

        info!("updated {} cards successfully", updated_count);

        Ok(updated_count)
    }

    async fn delete_one(&self, param: card::CardDomParam) -> Result<card::Model, AppError> {
        debug!("delete card attempt: {:?}", param);

        let result: card::Model = delete_one(&self.db, param).await.map_err(|e| {
            error!("failed to delete card: error={:?}", e);
            AppError::from(e)
        })?;

        info!("card deleted successfully: id={}", result.id);

        Ok(result)
    }
}
