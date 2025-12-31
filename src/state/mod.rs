use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::{
    domain::user::{UserDomainImpl, UserDomainInitParam},
    usecase::{
        auth::AuthUsecase,
        card::CardUsecase,
        user::{UserUseInitParam, UserUsecase},
    },
};

#[derive(Clone)]
pub struct AppState {
    pub auth_usecase: Arc<AuthUsecase>,
    pub user_usecase: Arc<UserUsecase>,
    pub card_usecase: Arc<CardUsecase>,
}

impl AppState {
    pub fn new(db: DatabaseConnection, secret_key: String) -> Self {
        // Initialize the usecases
        let auth = AuthUsecase::new(db.clone(), secret_key);
        let user = UserUsecase::new(UserUseInitParam {
            user_domain: Arc::new(UserDomainImpl::new(UserDomainInitParam { db: db.clone() })),
        });
        let card = CardUsecase::new(db.clone());

        Self {
            auth_usecase: Arc::new(auth),
            user_usecase: Arc::new(user),
            card_usecase: Arc::new(card),
        }
    }
}
