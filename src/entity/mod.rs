pub mod card;
pub mod error;
pub mod response;
pub mod user;
pub mod util;

use sea_orm::Condition;

pub trait Filterable {
    fn to_condition(&self) -> Condition;
}
