use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::domain::{card::CardDomainTrait, user::UserDomainTrait};

pub mod card;
pub mod helper;
pub mod user;

// --- Structs and Initiation

pub struct Domain {
    pub user: Arc<dyn UserDomainTrait>,
    pub card: Arc<dyn CardDomainTrait>,
}

pub struct InitParam {
    pub db: DatabaseConnection,
}

pub fn init(param: InitParam) -> Domain {
    // Initialize list of all domain
    let user = Arc::new(user::init(user::InitParam {
        db: param.db.clone(),
    }));
    let card = Arc::new(card::init(card::InitParam { db: param.db }));

    // Return the value in here
    Domain { user, card }
}
