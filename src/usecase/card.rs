use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    domain::card::CardDomainTrait,
    entity::{
        card::{self, CardUseParam, CreateCardUseParam, UpdateCardUseParam},
        response::AppCode,
    },
};

// --- Trait and Usecase Struct
#[async_trait]
pub trait CardUsecaseTrait: Send + Sync {
    async fn create(&self, create_param: CreateCardUseParam) -> Result<card::Model, AppCode>;
    #[allow(dead_code)]
    async fn get_list(&self, param: CardUseParam) -> Result<Vec<card::Model>, AppCode>;

    // This need to be changed in the future
    async fn update_one(&self, update_param: UpdateCardUseParam) -> Result<card::Model, AppCode>;
    async fn delete_one(&self, param: CardUseParam) -> Result<card::Model, AppCode>;
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
        let create_dom_param = card::CreateCardDomParam {
            title: create_param.title,
            description: create_param.description,
        };

        let result = self
            .card_domain
            .create(create_dom_param)
            .await
            .map_err(AppCode::from)?;

        Ok(result)
    }

    async fn get_list(&self, param: CardUseParam) -> Result<Vec<card::Model>, AppCode> {
        let card_dom_param = card::CardDomParam { id: param.id };
        let (result, _total) = self
            .card_domain
            .get_list(card_dom_param)
            .await
            .map_err(AppCode::from)?;

        println!("Total cards: {}", _total);

        Ok(result)
    }

    async fn update_one(&self, update_param: UpdateCardUseParam) -> Result<card::Model, AppCode> {
        let dom_param = card::CardDomParam {
            id: Some(update_param.id),
        };

        let update_dom_param = card::CardDomUpdateParam {
            card_status: Some(update_param.status),
            ..Default::default()
        };

        let result = self
            .card_domain
            .update_one(dom_param, update_dom_param)
            .await
            .map_err(AppCode::from)?;

        Ok(result)
    }

    async fn delete_one(&self, param: CardUseParam) -> Result<card::Model, AppCode> {
        let dom_param = card::CardDomParam { id: param.id };

        let result = self
            .card_domain
            .delete_one(dom_param)
            .await
            .map_err(AppCode::from)?;

        Ok(result)
    }
}
