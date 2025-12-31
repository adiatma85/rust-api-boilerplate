use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::{
    domain::Domain,
    usecase::{
        auth::AuthUsecase,
        card::{CardUseInitParam, CardUsecase},
        user::{UserUseInitParam, UserUsecase, UserUsecaseTrait},
    },
};

pub mod auth;
pub mod card;
pub mod user;

#[derive(Clone)]
pub struct Usecase {
    pub user: Arc<dyn UserUsecaseTrait>,
    // For the moment, we will change the CardUsecase in the future to use trait instead
    pub card: Arc<CardUsecase>,
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
    let user = Arc::new(UserUsecase::new(UserUseInitParam {
        user_domain: params.domain.user,
    }));

    let card = Arc::new(CardUsecase::new(CardUseInitParam {
        db: params.db.clone(),
    }));

    let auth = Arc::new(AuthUsecase::new(crate::usecase::auth::AuthUseInitParam {
        db: params.db,
        jwt_secret: params.jwt_secret,
    }));

    Usecase { user, card, auth }
}
