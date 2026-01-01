pub mod auth;
pub mod card;
pub mod error;
pub mod response;
pub mod user;
pub mod util;

use sea_orm::Condition;

pub trait Filterable {
    fn to_condition(&self) -> Condition;
}

pub trait Updatable<A> {
    fn apply_to(self, active_model: A) -> A;
}
