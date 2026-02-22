use std::sync::Arc;

use crate::business::{
    domain::Domain,
    usecase::{auth::AuthUsecaseTrait, card::CardUsecaseTrait, user::UserUsecaseTrait},
};

pub mod auth;
pub mod card;
pub mod user;

#[derive(Clone)]
pub struct Usecase {
    pub user: Arc<dyn UserUsecaseTrait>,
    pub card: Arc<dyn CardUsecaseTrait>,
    pub auth: Arc<dyn AuthUsecaseTrait>,
}

pub struct InitParam {
    // This is the normal import because we want to standardize this architecture
    pub domain: Domain,

    // Temporaries, we inject the jwt_secret in here
    // Ideally, we want to send a config either reference or owned
    pub jwt_secret: String,
}

// This is the initialize
pub fn init(params: InitParam) -> Usecase {
    let user = Arc::new(user::init(user::InitParam {
        user_domain: params.domain.user.clone(),
    }));

    let card = Arc::new(card::init(card::InitParam {
        card_domain: params.domain.card,
    }));

    let auth = Arc::new(auth::init(auth::InitParam {
        jwt_secret: params.jwt_secret,
        user_domain: params.domain.user,
    }));

    Usecase { user, card, auth }
}
