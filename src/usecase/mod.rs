use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::{
    domain::Domain,
    usecase::{
        auth::AuthUsecase,
        card::CardUsecaseTrait,
        user::{UserUsecase, UserUsecaseTrait},
    },
};

pub mod auth;
pub mod card;
pub mod user;

#[derive(Clone)]
pub struct Usecase {
    pub user: Arc<dyn UserUsecaseTrait>,
    // For the moment, we will change the CardUsecase in the future to use trait instead
    pub card: Arc<dyn CardUsecaseTrait>,
    // For the moment, we will change the AuthUsecase in the future to use trait instead
    pub auth: Arc<AuthUsecase>,
}

pub struct InitParam {
    // For now we also import database in here:
    pub db: DatabaseConnection,

    // This is the normal import because we want to standardize this architecture
    pub domain: Domain,

    // Temporaries, we inject the jwt_secret in here
    // Ideally, we want to send a config either reference or owned
    pub jwt_secret: String,
}

// This is the initialize
pub fn init(params: InitParam) -> Usecase {
    let user = Arc::new(UserUsecase::new(user::InitParam {
        user_domain: params.domain.user,
    }));

    let card = Arc::new(card::init(card::InitParam {
        card_domain: params.domain.card,
    }));

    let auth = Arc::new(AuthUsecase::new(auth::InitParam {
        db: params.db,
        jwt_secret: params.jwt_secret,
    }));

    Usecase { user, card, auth }
}
