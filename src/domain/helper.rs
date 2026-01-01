use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, FromQueryResult, IntoActiveModel,
    PaginatorTrait, QueryFilter,
};

use crate::entity::{Filterable, Updatable, response::Pagination};

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

#[allow(dead_code)]
// Temporary for the fixing in the future. For now, we need to implement other generics function
pub async fn fetch_list_2<E, M, F>(
    db: &DatabaseConnection,
    filter: F,
    page: u64,
    limit: u64,
) -> Result<(Vec<M>, Pagination), sea_orm::DbErr>
where
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync,
    F: Filterable,
{
    // 1. Build Condition
    let condition = filter.to_condition();

    // 2. Build Query
    let query = E::find().filter(condition);

    // 3. Create Paginator
    let paginator = query.paginate(db, limit);

    // 4. Execute Queries
    // We run the COUNT query first
    let total_elements = paginator.num_items().await?;

    // We run the SELECT data query second
    let data = paginator.fetch_page(page).await?;

    // 5. Calculate Total Pages Manually
    // (Optimization: doing this in Rust saves a DB round-trip vs calling paginator.num_pages().await)
    let total_pages = if limit > 0 {
        total_elements.div_ceil(limit)
    } else {
        0
    };

    // 6. Build Pagination Struct
    let pagination = Pagination {
        current_page: page,
        current_elements: data.len() as u64,
        total_pages,
        total_elements,
        sort_by: vec![], // Ignored for now as requested
    };

    Ok((data, pagination))
}

// Generic function to fetch EXACTLY ONE record
#[allow(dead_code)]
pub async fn fetch_one<E, M, F>(db: &DatabaseConnection, filter: F) -> Result<M, sea_orm::DbErr>
where
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync,
    F: Filterable,
{
    // 1. Build Condition
    let condition = filter.to_condition();

    // 2. Execute Query
    // .one() returns Result<Option<M>, DbErr>
    let result = E::find().filter(condition).one(db).await?;

    // 3. Handle "Not Found" manually
    match result {
        Some(model) => Ok(model),
        None => Err(sea_orm::DbErr::RecordNotFound(
            "Record not found".to_string(),
        )),
    }
}

// Generic function to CREATE EXACTLY ONE record
// E = The Entity (e.g., user::Entity)
// A = The ActiveModel (e.g., user::ActiveModel)
// M = The Model (e.g., user::Model)
// D = The Data Param (e.g., CreateUserParam)
#[allow(dead_code)]
pub async fn create_one<E, A, M, D>(db: &DatabaseConnection, data: D) -> Result<M, sea_orm::DbErr>
where
    E: EntityTrait<Model = M>,
    A: ActiveModelTrait<Entity = E> + Send,
    M: FromQueryResult + Sized + Send + Sync + IntoActiveModel<A>,
    D: IntoActiveModel<A>, // <--- Constraint: The Param must be convertible to ActiveModel
{
    // 1. Convert DTO to ActiveModel
    let active_model = data.into_active_model();

    // 2. Insert and return the Model in one go
    // Note: exec_with_returning works natively on Postgres.
    // On MySQL/SQLite, SeaORM emulates this by doing Insert + Select automatically.
    let result = E::insert(active_model).exec_with_returning(db).await?;

    Ok(result)
}

// Generic function to UPDATE EXACTLY ONE record
// E = Entity
// A = ActiveModel
// M = Model
// F = Filter (to find the record)
// U = Update DTO (containing new data)
#[allow(dead_code)]
pub async fn update_one<E, A, M, F, U>(
    db: &DatabaseConnection,
    filter: F,
    data: U,
) -> Result<M, sea_orm::DbErr>
where
    E: EntityTrait<Model = M>,
    A: ActiveModelTrait<Entity = E> + Send,
    M: FromQueryResult + Sized + Send + Sync + IntoActiveModel<A>,
    F: Filterable,
    U: Updatable<A>, // <--- Constraint: The Data must know how to update the ActiveModel
{
    // 1. Find the existing record
    // We reuse the logic of "fetch_one" inline here
    let model = E::find()
        .filter(filter.to_condition())
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(
            "Record not found".to_string(),
        ))?;

    // 2. Convert the Model to an ActiveModel (so we can modify it)
    let mut active_model = model.into_active_model();

    // 3. Apply the changes from the DTO
    active_model = data.apply_to(active_model);

    // 4. Execute the update and return the fresh Model
    // SeaORM's update().exec() automatically returns the updated Model
    let updated_model = E::update(active_model).exec(db).await?;

    Ok(updated_model)
}

// Generic function to UPDATE MULTIPLE records based on a filter
// Returns the number of rows affected.
#[allow(dead_code)]
pub async fn update_many<E, A, F, D>(
    db: &DatabaseConnection,
    filter: F,
    data: D,
) -> Result<u64, sea_orm::DbErr>
where
    E: EntityTrait, // Note: We don't need 'Model' here, just the Entity
    A: ActiveModelTrait<Entity = E> + Send,
    F: Filterable,
    D: IntoActiveModel<A>, // DTO must convert to a "Partial" ActiveModel
{
    // 1. Build Condition (WHERE ...)
    let condition = filter.to_condition();

    // 2. Convert DTO to ActiveModel (SET ...)
    // This AM should only have 'Set' for fields we are changing.
    let active_model = data.into_active_model();

    // 3. Execute Update
    let res = E::update_many()
        .set(active_model)
        .filter(condition)
        .exec(db)
        .await?;

    Ok(res.rows_affected)
}

pub async fn delete_one<E, A, M, F>(db: &DatabaseConnection, filter: F) -> Result<M, sea_orm::DbErr>
where
    E: EntityTrait<Model = M>,
    A: ActiveModelTrait<Entity = E> + Send,
    M: FromQueryResult + Sized + Send + Sync + IntoActiveModel<A> + Clone,
    F: Filterable,
{
    // 1. Find the existing record
    let model = E::find()
        .filter(filter.to_condition())
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(
            "Record not found".to_string(),
        ))?;

    // 2. Convert to ActiveModel to perform the delete
    // We clone the model so we can return the original data later
    let active_model = model.clone().into_active_model();

    // 3. Delete
    // SeaORM uses the Primary Key inside the ActiveModel to delete the specific row
    E::delete(active_model).exec(db).await?;

    // 4. Return the data of the deleted record
    Ok(model)
}

// --- We can write other functions in here also, for example generic function to fetch a single entity instead ---
// --- Or we can also create a generic function to update an entity ---
