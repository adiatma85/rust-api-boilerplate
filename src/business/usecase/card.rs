use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, error, info};

use crate::business::{
    domain::card::CardDomainTrait,
    entity::{
        card::{self, CardDomParam, CreateCardUseParam, UpdateCardUseParam},
        response::AppCode,
    },
};

// --- Trait and Usecase Struct
#[async_trait]
pub trait CardUsecaseTrait: Send + Sync {
    async fn create(&self, create_param: CreateCardUseParam) -> Result<card::Model, AppCode>;
    async fn get(&self, param: CardDomParam) -> Result<card::Model, AppCode>;
    async fn get_list(&self, param: CardDomParam) -> Result<Vec<card::Model>, AppCode>;

    // This need to be changed in the future
    async fn update_one(
        &self,
        update_param: UpdateCardUseParam,
        select_param: CardDomParam,
    ) -> Result<card::Model, AppCode>;
    async fn delete_one(&self, param: CardDomParam) -> Result<card::Model, AppCode>;
}

pub struct CardUsecase {
    card_domain: Arc<dyn CardDomainTrait>,
}

pub struct InitParam {
    pub card_domain: Arc<dyn CardDomainTrait>,
}

pub fn init(param: InitParam) -> impl CardUsecaseTrait {
    CardUsecase {
        card_domain: param.card_domain,
    }
}

#[async_trait]
impl CardUsecaseTrait for CardUsecase {
    async fn create(&self, create_param: CreateCardUseParam) -> Result<card::Model, AppCode> {
        debug!(
            "create card attempt: title={}, user_id={}",
            create_param.title, create_param.user_id
        );

        let create_dom_param = card::CreateCardDomParam {
            title: create_param.title.clone(),
            description: create_param.description,
            user_id: create_param.user_id,
        };

        let created_card = self
            .card_domain
            .create(create_dom_param)
            .await
            .map_err(|e| {
                error!(
                    "failed to create card: title={}, user_id={}, error: {:?}",
                    create_param.title, create_param.user_id, e
                );
                AppCode::from(e)
            })?;

        info!(
            "card created successfully: id={}, title={}",
            created_card.id, created_card.title
        );
        Ok(created_card)
    }

    async fn get(&self, param: CardDomParam) -> Result<card::Model, AppCode> {
        let card_id = param.id;
        debug!("get card request: id={:?}", card_id);

        let card = self.card_domain.get(param).await.map_err(|e| {
            error!("failed to retrieve card: id={:?}, error: {:?}", card_id, e);
            AppCode::from(e)
        })?;

        info!("card retrieved successfully: id={}", card.id);
        Ok(card)
    }

    async fn get_list(&self, param: CardDomParam) -> Result<Vec<card::Model>, AppCode> {
        debug!("get card list request: {:?}", param);

        let (result, pagination) = self.card_domain.get_list(param).await.map_err(|e| {
            error!("failed to retrieve card list: {:?}", e);
            AppCode::from(e)
        })?;

        debug!("pagination details: {:?}", pagination);

        info!(
            "card list retrieved successfully: {} cards (page: {}/{})",
            pagination.current_elements, pagination.current_page, pagination.total_pages
        );

        Ok(result)
    }

    async fn update_one(
        &self,
        update_param: UpdateCardUseParam,
        select_param: CardDomParam,
    ) -> Result<card::Model, AppCode> {
        let card_id = select_param.id;
        let status = update_param.status.clone();

        debug!("update card attempt: id={:?}, status={}", card_id, status);

        let update_dom_param = card::CardDomUpdateParam {
            card_status: Some(status.clone()),
            ..Default::default()
        };

        let updated_card = self
            .card_domain
            .update_one(select_param, update_dom_param)
            .await
            .map_err(|e| {
                error!(
                    "failed to update card: id={:?}, status={}, error: {:?}",
                    card_id, status, e
                );
                AppCode::from(e)
            })?;

        info!(
            "card updated successfully: id={}, new_status={}",
            updated_card.id, updated_card.card_status
        );
        Ok(updated_card)
    }

    async fn delete_one(&self, param: CardDomParam) -> Result<card::Model, AppCode> {
        debug!("delete card attempt: id={:?}", param.id);

        let dom_param = card::CardDomParam {
            id: param.id,
            ..Default::default()
        };

        let deleted_card = self.card_domain.delete_one(dom_param).await.map_err(|e| {
            error!("failed to delete card: id={:?}, error: {:?}", param.id, e);
            AppCode::from(e)
        })?;

        info!("card deleted successfully: id={}", deleted_card.id);
        Ok(deleted_card)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mockall::predicate;
    use sea_orm::prelude::DateTimeUtc;

    use super::*;
    use crate::business::{domain::card::MockCardDomainTrait, entity::response::Pagination};

    fn create_test_card() -> card::Model {
        card::Model {
            id: 1,
            title: "Test Card".to_string(),
            description: Some("Description".to_string()),
            card_status: "todo".to_string(),
            user_id: 1,
            created_at: DateTimeUtc::from(Utc::now()),
            updated_at: DateTimeUtc::from(Utc::now()),
        }
    }

    #[tokio::test]
    async fn test_create_card_success() {
        let mut mock_card_domain = MockCardDomainTrait::new();
        let card = create_test_card();
        let card_clone = card.clone();

        let expected_param = card::CreateCardDomParam {
            title: "Test Card".to_string(),
            description: Some("Description".to_string()),
            user_id: 1,
        };

        mock_card_domain
            .expect_create()
            .with(predicate::eq(expected_param))
            .times(1)
            .returning(move |_| Ok(card_clone.clone()));

        let card_usecase = init(InitParam {
            card_domain: Arc::new(mock_card_domain),
        });

        let result = card_usecase
            .create(CreateCardUseParam {
                title: "Test Card".to_string(),
                description: Some("Description".to_string()),
                user_id: 1,
            })
            .await;

        assert!(result.is_ok());
        let created_card = result.unwrap();
        assert_eq!(created_card.title, "Test Card");
    }

    #[tokio::test]
    async fn test_get_card_success() {
        let mut mock_card_domain = MockCardDomainTrait::new();
        let card = create_test_card();
        let card_clone = card.clone();

        let param = CardDomParam {
            id: Some(1),
            ..Default::default()
        };
        let param_clone = CardDomParam {
            id: Some(1),
            ..Default::default()
        };

        mock_card_domain
            .expect_get()
            .with(predicate::eq(param_clone))
            .times(1)
            .returning(move |_| Ok(card_clone.clone()));

        let card_usecase = init(InitParam {
            card_domain: Arc::new(mock_card_domain),
        });

        let result = card_usecase.get(param).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, 1);
    }

    #[tokio::test]
    async fn test_get_list_card_success() {
        let mut mock_card_domain = MockCardDomainTrait::new();
        let card = create_test_card();
        let pagination = Pagination {
            current_page: 1,
            current_elements: 1,
            total_pages: 1,
            total_elements: 1,
            sort_by: vec![],
        };

        mock_card_domain
            .expect_get_list()
            .times(1)
            .returning(move |_| Ok((vec![card.clone()], pagination.clone())));

        let card_usecase = init(InitParam {
            card_domain: Arc::new(mock_card_domain),
        });

        let result = card_usecase.get_list(CardDomParam::default()).await;

        assert!(result.is_ok());
        let cards = result.unwrap();
        assert_eq!(cards.len(), 1);
    }

    #[tokio::test]
    async fn test_update_one_success() {
        let mut mock_card_domain = MockCardDomainTrait::new();
        let mut card = create_test_card();
        card.card_status = "done".to_string();
        let card_clone = card.clone();

        let expected_dom_param = CardDomParam {
            id: Some(1),
            ..Default::default()
        };

        // Match update data carefully
        let expected_update_data = card::CardDomUpdateParam {
            card_status: Some("done".to_string()),
            ..Default::default()
        };

        mock_card_domain
            .expect_update_one()
            .with(
                predicate::eq(expected_dom_param),
                predicate::eq(expected_update_data),
            )
            .times(1)
            .returning(move |_, _| Ok(card_clone.clone()));

        let card_usecase = init(InitParam {
            card_domain: Arc::new(mock_card_domain),
        });

        let result = card_usecase
            .update_one(
                UpdateCardUseParam {
                    status: "done".to_string(),
                    ..Default::default()
                },
                CardDomParam {
                    id: Some(1),
                    ..Default::default()
                },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().card_status, "done");
    }

    #[tokio::test]
    async fn test_delete_one_success() {
        let mut mock_card_domain = MockCardDomainTrait::new();
        let card = create_test_card();
        let card_clone = card.clone();

        let expected_param = CardDomParam {
            id: Some(1),
            ..Default::default()
        };

        mock_card_domain
            .expect_delete_one()
            .with(predicate::eq(expected_param))
            .times(1)
            .returning(move |_| Ok(card_clone.clone()));

        let card_usecase = init(InitParam {
            card_domain: Arc::new(mock_card_domain),
        });

        let param = CardDomParam {
            id: Some(1),
            ..Default::default()
        };

        let result = card_usecase.delete_one(param).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, 1);
    }
}
