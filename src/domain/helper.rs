use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, FromQueryResult, IntoActiveModel,
    PaginatorTrait, QueryFilter,
};

use crate::entity::{Filterable, Updatable, response::Pagination, util::Paginatable};

// --- General Helper function is below ---

// A generic function to fetch a list for ANY Entity
// E = The Entity (e.g., user::Entity)
// M = The Model (e.g., user::Model)
// F = The Filter Param (e.g., UserDomainParam)
pub async fn fetch_list<E, M, F>(
    db: &DatabaseConnection,
    filter: F, // Clean signature!
) -> Result<(Vec<M>, Pagination), sea_orm::DbErr>
where
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync,
    F: Filterable + Paginatable, // We now require the Paginatable trait
{
    // 1. Build Condition
    let condition = filter.to_condition();

    // 2. Build Base Query
    let query = E::find().filter(condition);

    // 3. Handle Branching Logic (Unlimited vs Paginated)
    if filter.is_limit_disabled() {
        // --- BRANCH A: FETCH ALL ---

        let data = query.all(db).await?;
        let total_elements = data.len() as u64;

        let pagination = Pagination {
            current_page: 0,
            current_elements: total_elements,
            total_pages: 1, // Only 1 page exists when limit is disabled
            total_elements,
            sort_by: vec![],
        };

        Ok((data, pagination))
    } else {
        // --- BRANCH B: PAGINATED ---

        let page = filter.get_page();
        let limit = filter.get_limit();

        // Safety: If limit is sent as 0 by accident, force a default to avoid DB panic
        let safe_limit = if limit == 0 { 10 } else { limit };

        let paginator = query.paginate(db, safe_limit);

        let total_elements = paginator.num_items().await?;
        let data = paginator.fetch_page(page).await?;

        // Optimization: Manual calc
        let total_pages = total_elements.div_ceil(safe_limit);

        let pagination = Pagination {
            current_page: page,
            current_elements: data.len() as u64,
            total_pages,
            total_elements,
            sort_by: vec![],
        };

        Ok((data, pagination))
    }
}

// Generic function to fetch EXACTLY ONE record
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

// ----------- UNIT TESTS -----------

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sea_orm::{Condition, DatabaseBackend, MockDatabase, entity::prelude::*};

    use super::*; // Import your fetch_list and traits // <-- ADD THIS LINE

    // --- 1. Define a Dummy Entity (Standard SeaORM Boilerplate) ---
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "bakery")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub profit: f64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}

    // --- 2. Define a Dummy Filter Struct ---
    pub struct TestFilter {
        pub page: u64,
        pub limit: u64,
        pub disable_limit: bool,
        pub name_contains: Option<String>,
    }

    // Implement your 'Paginatable' trait
    impl Paginatable for TestFilter {
        fn get_page(&self) -> u64 {
            self.page
        }
        fn get_limit(&self) -> u64 {
            self.limit
        }
        fn is_limit_disabled(&self) -> bool {
            self.disable_limit
        }
    }

    // Implement your 'Filterable' trait
    impl Filterable for TestFilter {
        fn to_condition(&self) -> Condition {
            let mut condition = Condition::all();
            if let Some(name) = &self.name_contains {
                condition = condition.add(Column::Name.contains(name));
            }
            condition
        }
    }

    #[tokio::test]
    async fn test_fetch_list_paginated_postgres() {
        // 1. Setup the Filter
        let filter = TestFilter {
            page: 0,
            limit: 10,
            disable_limit: false,
            name_contains: Some("Happy".to_string()),
        };

        // 2. Setup Mock Results manually (No maplit needed)

        // Create the Count Map manually
        let mut count_map = BTreeMap::new();
        count_map.insert("num_items".to_string(), Into::<sea_orm::Value>::into(50i64));

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                // Query 1 Result: COUNT(*)
                // We pass the map we created above
                vec![count_map],
            ])
            .append_query_results(vec![
                // Query 2 Result: The actual data
                vec![
                    Model {
                        id: 1,
                        name: "Happy Bakery".to_owned(),
                        profit: 100.0,
                    },
                    Model {
                        id: 2,
                        name: "Happy Cookies".to_owned(),
                        profit: 200.0,
                    },
                ],
            ])
            .into_connection();

        // 3. Execute
        let result = fetch_list::<Entity, Model, TestFilter>(&db, filter).await;

        // 4. Assertions
        assert!(result.is_ok());
        let (data, pagination) = result.unwrap();

        assert_eq!(data.len(), 2);
        assert_eq!(pagination.total_elements, 50);
    }

    #[tokio::test]
    async fn test_fetch_list_unlimited_postgres() {
        // 1. Setup Filter (Limit Disabled)
        let filter = TestFilter {
            page: 0,
            limit: 10, // Should be ignored
            disable_limit: true,
            name_contains: None,
        };

        // 2. Setup Mock Database
        // We expect only 1 query (SELECT ALL)
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![
                Model {
                    id: 1,
                    name: "Bakery A".to_owned(),
                    profit: 10.0,
                },
                Model {
                    id: 2,
                    name: "Bakery B".to_owned(),
                    profit: 20.0,
                },
                Model {
                    id: 3,
                    name: "Bakery C".to_owned(),
                    profit: 30.0,
                },
            ]])
            .into_connection();

        // 3. Execute
        let result = fetch_list::<Entity, Model, TestFilter>(&db, filter).await;

        // 4. Assertions
        let (data, pagination) = result.unwrap();

        assert_eq!(data.len(), 3);

        // Check hardcoded values for unlimited branch
        assert_eq!(pagination.total_pages, 1);
        assert_eq!(pagination.current_page, 0);
        assert_eq!(pagination.total_elements, 3);
    }

    #[tokio::test]
    async fn test_fetch_list_generates_correct_sql_postgres() {
        use std::collections::BTreeMap; // Import BTreeMap

        // 1. Setup Filter
        let filter = TestFilter {
            page: 1,
            limit: 5,
            disable_limit: false,
            name_contains: Some("Test".to_string()),
        };

        // 2. Setup Mock with Manual BTreeMap
        let mut count_row = BTreeMap::new();
        count_row.insert("num_items".to_string(), Into::<sea_orm::Value>::into(1i64));

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![
                count_row, // Pass the manually created map here
            ]]) // Count Query Result
            .append_query_results(vec![vec![Model {
                id: 1,
                name: "Test".to_string(),
                profit: 0.0,
            }]]) // Data Query Result
            .into_connection();

        // 3. Execute
        let _ = fetch_list::<Entity, Model, TestFilter>(&db, filter).await;

        // 4. Verify SQL
        let log = db.into_transaction_log();

        // FIXED: Use format!("{:?}") because Transaction doesn't implement Display
        // This converts the log object (SQL + Params) into a debug string.
        let count_log_str = format!("{:?}", log[0]);

        // Print it if you want to see what it looks like:
        // println!("Count SQL: {}", count_log_str);

        assert!(count_log_str.contains("SELECT COUNT(*)"));
        // Note: SeaORM quotes might vary ("bakery"."name" vs "name"), so checking a partial substring is safer
        assert!(count_log_str.contains("name"));

        let fetch_log_str = format!("{:?}", log[1]);
        assert!(fetch_log_str.contains("LIMIT $2 OFFSET $3"));
    }
}
