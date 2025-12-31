use std::sync::Arc;

use sea_orm::{DatabaseConnection, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter};

use crate::{
    domain::user::{UserDomainImpl, UserDomainTrait},
    entity::Filterable,
};

pub mod user;

// --- Structs and Initiation

pub struct Domain {
    pub user: Arc<dyn UserDomainTrait>,
}

pub struct InitParam {
    pub db: DatabaseConnection,
}

pub fn init(param: InitParam) -> Domain {
    // Initialize list of all domain
    let user = Arc::new(UserDomainImpl::new(user::UserDomainInitParam {
        db: param.db.clone(),
    }));

    // Return the value in here
    Domain { user }
}

// --- General Helper function is below ---

// A generic function to fetch a list for ANY Entity
// E = The Entity (e.g., user::Entity)
// M = The Model (e.g., user::Model)
// F = The Filter Param (e.g., UserDomainParam)
pub async fn fetch_list<E, M, F>(
    db: &DatabaseConnection,
    filter: F,
    page: u64,
    limit: u64,
) -> Result<(Vec<M>, u64), sea_orm::DbErr>
where
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync,
    F: Filterable,
{
    let condition = filter.to_condition();

    // 2. Call find on the Type "E" directly
    let query = E::find().filter(condition);

    let paginator = query.paginate(db, limit);
    let total = paginator.num_items().await?;
    let data = paginator.fetch_page(page).await?;

    Ok((data, total))
}

// --- We can write other functions in here also, for example generic function to fetch a single entity instead ---
// --- Or we can also create a generic function to update an entity ---
