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

    use sea_orm::{
        ActiveValue::Set, Condition, DatabaseBackend, MockDatabase, MockExecResult,
        entity::prelude::*,
    };

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

    // 1. Define the DTO (Data Transfer Object)
    struct CreateBakeryParam {
        pub name: String,
        pub profit: f64,
    }

    // 2. Implement the Trait required by your generic function
    impl IntoActiveModel<ActiveModel> for CreateBakeryParam {
        fn into_active_model(self) -> ActiveModel {
            ActiveModel {
                // Note: We don't set ID, because the DB auto-increments it
                name: Set(self.name),
                profit: Set(self.profit),
                ..Default::default()
            }
        }
    }

    // 1. Define the Update DTO
    struct UpdateBakeryParam {
        pub new_profit: f64,
    }

    // 2. Implement the Trait
    // This logic says: "Here is how you apply my data to the ActiveModel"
    impl Updatable<ActiveModel> for UpdateBakeryParam {
        fn apply_to(self, mut active_model: ActiveModel) -> ActiveModel {
            active_model.profit = Set(self.new_profit);
            active_model
        }
    }

    // 1. Define a DTO for batch updates
    struct UpdateProfitParam {
        pub new_profit: f64,
    }

    // 2. Implement IntoActiveModel
    // This creates an "incomplete" ActiveModel where only the 'profit' field is Set.
    // SeaORM uses this to generate the "SET profit = ?" SQL.
    impl IntoActiveModel<ActiveModel> for UpdateProfitParam {
        fn into_active_model(self) -> ActiveModel {
            ActiveModel {
                profit: Set(self.new_profit),
                ..Default::default()
            }
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
        let (data, pagination) = result.expect("Fetch list failed");

        assert_eq!(data.len(), 2);
        assert_eq!(pagination.total_elements, 50);
    }

    #[tokio::test]
    async fn test_fetch_list_paginated_mysql() {
        // 1. Setup Filter
        let filter = TestFilter {
            page: 0,
            limit: 10,
            disable_limit: false,
            name_contains: Some("Happy".to_string()),
        };

        // 2. Setup Count Map (The "Shotgun" Fix)
        let mut count_map = BTreeMap::new();
        count_map.insert("num_items".to_string(), Value::Int(Some(50)));

        // 3. Setup Data
        let models = vec![
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
        ];

        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![count_map]]) // Queue 1: Count
            .append_query_results(vec![models]) // Queue 2: Data
            .into_connection();

        // 4. Execute
        let result = fetch_list::<Entity, Model, TestFilter>(&db, filter).await;

        // 5. Assertions
        let (data, pagination) = result.expect("Fetch list failed");

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
        let (data, pagination) = result.expect("Fetch list failed");

        assert_eq!(data.len(), 3);

        // Check hardcoded values for unlimited branch
        assert_eq!(pagination.total_pages, 1);
        assert_eq!(pagination.current_page, 0);
        assert_eq!(pagination.total_elements, 3);
    }

    #[tokio::test]
    async fn test_fetch_list_unlimited_mysql() {
        // 1. Setup Filter (Limit Disabled)
        let filter = TestFilter {
            page: 0,
            limit: 10, // Should be ignored
            disable_limit: true,
            name_contains: None,
        };

        // 2. Setup Mock Database
        // We expect only 1 query (SELECT ALL)
        let db = MockDatabase::new(DatabaseBackend::MySql)
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
        let (data, pagination) = result.expect("Fetch list failed");

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

    #[tokio::test]
    async fn test_fetch_list_generates_correct_sql_mysql() {
        // 1. Setup Filter
        let filter = TestFilter {
            page: 1,
            limit: 5,
            disable_limit: false,
            name_contains: Some("Test".to_string()),
        };

        // 2. Setup Mock with Manual BTreeMap (Using the "Shotgun" Fix)
        let mut count_row = BTreeMap::new();

        // Insert both keys to be safe against quoting behavior
        count_row.insert("num_items".to_string(), Value::Int(Some(1)));

        let db = MockDatabase::new(DatabaseBackend::MySql) // <--- MySQL Backend
            .append_query_results(vec![vec![count_row]]) // Count Query Result
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

        // --- CHECK COUNT QUERY ---
        let count_log_str = format!("{:?}", log[0]);
        // println!("Count SQL: {}", count_log_str); // Uncomment to debug

        assert!(count_log_str.contains("SELECT COUNT(*)"));
        // MySQL uses '?' for parameters (vs Postgres '$1')
        // It generates: WHERE `bakery`.`name` LIKE ?
        assert!(count_log_str.contains("LIKE ?"));

        // --- CHECK FETCH QUERY ---
        let fetch_log_str = format!("{:?}", log[1]);
        // println!("Fetch SQL: {}", fetch_log_str); // Uncomment to debug

        // MySQL Pagination Syntax: LIMIT ? OFFSET ?
        // (Postgres was LIMIT $2 OFFSET $3)
        assert!(fetch_log_str.contains("LIMIT ? OFFSET ?"));
    }

    #[tokio::test]
    async fn test_fetch_one_success_postgres() {
        // 1. Setup Filter
        let filter = TestFilter {
            page: 0,
            limit: 10,
            disable_limit: false,
            name_contains: Some("Specific Bakery".to_string()),
        };

        // 2. Setup Mock Database (Return 1 Record)
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                // The query returns exactly one model
                vec![Model {
                    id: 10,
                    name: "Specific Bakery".to_owned(),
                    profit: 500.0,
                }],
            ])
            .into_connection();

        // 3. Execute
        let result = fetch_one::<Entity, Model, TestFilter>(&db, filter).await;

        // 4. Assertions
        let model = result.expect("Should return a model");
        assert_eq!(model.id, 10);
        assert_eq!(model.name, "Specific Bakery");

        // Optional: Verify SQL parameters
        let log = db.into_transaction_log();
        println!("{:?}", log[0]); // Check if "LIMIT 1" is present
    }

    #[tokio::test]
    async fn test_fetch_one_success_mysql() {
        // 1. Setup Filter
        let filter = TestFilter {
            page: 0,
            limit: 10,
            disable_limit: false,
            name_contains: Some("Specific Bakery".to_string()),
        };

        // 2. Setup Mock Database (Return 1 Record)
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![
                // The query returns exactly one model
                vec![Model {
                    id: 10,
                    name: "Specific Bakery".to_owned(),
                    profit: 500.0,
                }],
            ])
            .into_connection();

        // 3. Execute
        let result = fetch_one::<Entity, Model, TestFilter>(&db, filter).await;

        // 4. Assertions
        let model = result.expect("Should return a model");
        assert_eq!(model.id, 10);
        assert_eq!(model.name, "Specific Bakery");

        // Optional: Verify SQL parameters
        let log = db.into_transaction_log();
        println!("{:?}", log[0]); // Check if "LIMIT 1" is present
    }

    #[tokio::test]
    async fn test_fetch_one_not_found_mysql() {
        // 1. Setup Filter
        let filter = TestFilter {
            page: 0,
            limit: 10,
            disable_limit: false,
            name_contains: Some("Ghost Bakery".to_string()),
        };

        // 2. Setup Mock Database (Return Empty Vector)
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![
                // The query returns an empty list (Simulating 'None')
                Vec::<Model>::new(),
            ])
            .into_connection();

        // 3. Execute
        let result = fetch_one::<Entity, Model, TestFilter>(&db, filter).await;

        // 4. Assertions
        match result {
            Err(sea_orm::DbErr::RecordNotFound(msg)) => {
                assert_eq!(msg, "Record not found");
            }
            Err(e) => panic!("Expected RecordNotFound, got {:?}", e),
            Ok(_) => panic!("Expected error, but got a Model!"),
        }
    }

    #[tokio::test]
    async fn test_fetch_one_not_found_postgres() {
        // 1. Setup Filter
        let filter = TestFilter {
            page: 0,
            limit: 10,
            disable_limit: false,
            name_contains: Some("Ghost Bakery".to_string()),
        };

        // 2. Setup Mock Database (Return Empty Vector)
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                // The query returns an empty list (Simulating 'None')
                Vec::<Model>::new(),
            ])
            .into_connection();

        // 3. Execute
        let result = fetch_one::<Entity, Model, TestFilter>(&db, filter).await;

        // 4. Assertions
        match result {
            Err(sea_orm::DbErr::RecordNotFound(msg)) => {
                assert_eq!(msg, "Record not found");
            }
            Err(e) => panic!("Expected RecordNotFound, got {:?}", e),
            Ok(_) => panic!("Expected error, but got a Model!"),
        }
    }

    #[tokio::test]
    async fn test_create_one_mysql() {
        // 1. Setup Data
        let new_bakery = CreateBakeryParam {
            name: "Fresh Bread".to_string(),
            profit: 300.0,
        };

        // 2. Setup Mock Database
        let db = MockDatabase::new(DatabaseBackend::MySql)
            // STEP A: The INSERT command
            // We use 'append_exec_results' because INSERT is an execution, not a query.
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 15, // Simulate DB assigning ID 15
                rows_affected: 1,
            }])
            // STEP B: The SELECT command
            // SeaORM automatically runs this to fetch the data for 'exec_with_returning'
            .append_query_results(vec![vec![Model {
                id: 15, // Must match the last_insert_id above
                name: "Fresh Bread".to_string(),
                profit: 300.0,
            }]])
            .into_connection();

        // 3. Execute
        let result =
            create_one::<Entity, ActiveModel, Model, CreateBakeryParam>(&db, new_bakery).await;

        // 4. Assertions
        let model = result.expect("Failed to create record");

        assert_eq!(model.id, 15);
        assert_eq!(model.name, "Fresh Bread");

        // 5. Verify SQL (Optional, but instructive)
        let log = db.into_transaction_log();

        // Log[0] = INSERT INTO ...
        let insert_sql = format!("{:?}", log[0]);
        assert!(insert_sql.contains("INSERT INTO"));
        assert!(insert_sql.contains("Fresh Bread"));

        // Log[1] = SELECT ... WHERE id = 15
        let select_sql = format!("{:?}", log[1]);
        assert!(select_sql.contains("SELECT"));
        assert!(select_sql.contains("WHERE `bakery`.`id` = ?"));
    }

    #[tokio::test]
    async fn test_create_one_postgres() {
        // 1. Setup Data
        let new_bakery = CreateBakeryParam {
            name: "Fresh Bread".to_string(),
            profit: 300.0,
        };

        // 2. Setup Mock
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![Model {
                id: 15,
                name: "Fresh Bread".to_string(),
                profit: 300.0,
            }]])
            .into_connection();

        // 3. Execute
        let result =
            create_one::<Entity, ActiveModel, Model, CreateBakeryParam>(&db, new_bakery).await;

        // 4. Assertions
        let model = result.expect("Failed to create record");
        assert_eq!(model.id, 15);

        // 5. Verify SQL
        let log = db.into_transaction_log();
        assert_eq!(log.len(), 1); // Postgres MUST do it in 1 query

        let insert_sql = format!("{:?}", log[0]);

        // Debug: Uncomment this to see exactly what SeaORM generated!
        println!("Generated SQL: {}", insert_sql);

        // Assertion A: It is an INSERT
        assert!(insert_sql.contains("INSERT INTO"));

        // Assertion B: It uses RETURNING (This proves it's Postgres optimization)
        assert!(insert_sql.contains("RETURNING"));

        // Assertion C (Relaxed): Check for "id" generally, ignoring complex quoting
        assert!(insert_sql.contains("id"));
    }

    #[tokio::test]
    async fn test_update_one_mysql() {
        use sea_orm::{DatabaseBackend, MockExecResult};

        // 1. Setup Filter (To find the record)
        let filter = TestFilter {
            page: 0,
            limit: 1,
            disable_limit: false,
            name_contains: Some("Old Bakery".to_string()),
        };

        // 2. Setup Data (The change we want to make)
        let update_data = UpdateBakeryParam { new_profit: 900.0 };

        // 3. Define Models
        let original_model = Model {
            id: 1,
            name: "Old Bakery".to_owned(),
            profit: 100.0, // Old profit
        };

        let updated_model = Model {
            id: 1,
            name: "Old Bakery".to_owned(),
            profit: 900.0, // New profit
        };

        // 4. Setup Mock Database (Strict Order!)
        let db = MockDatabase::new(DatabaseBackend::MySql)
            // --- STEP 1: The 'Find' Operation ---
            // Your code calls E::find()...one(db) first.
            .append_query_results(vec![vec![original_model]])
            // --- STEP 2: The 'Update' Execution ---
            // SeaORM executes the UPDATE statement.
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0, // Not an insert, so 0 is fine
                rows_affected: 1,  // We updated 1 row
            }])
            // --- STEP 3: The 'Refresh' Operation ---
            // Because MySQL doesn't support "UPDATE ... RETURNING",
            // SeaORM automatically runs a SELECT to get the fresh data.
            .append_query_results(vec![vec![updated_model]])
            .into_connection();

        // 5. Execute
        let result = update_one::<Entity, ActiveModel, Model, TestFilter, UpdateBakeryParam>(
            &db,
            filter,
            update_data,
        )
        .await;

        // 6. Assertions
        let model = result.expect("Update failed");

        assert_eq!(model.id, 1);
        assert_eq!(model.profit, 900.0); // Should be the NEW profit

        // 7. Verify SQL Order (Optional)
        let log = db.into_transaction_log();

        // Log[0] = SELECT (Find)
        assert!(format!("{:?}", log[0]).contains("SELECT"));

        // Log[1] = UPDATE (Execute)
        assert!(format!("{:?}", log[1]).contains("UPDATE"));

        // Log[2] = SELECT (Refresh)
        assert!(format!("{:?}", log[2]).contains("SELECT"));
    }

    #[tokio::test]
    async fn test_update_one_postgres() {
        // 1. Setup Filter (To find the record)
        let filter = TestFilter {
            page: 0,
            limit: 1,
            disable_limit: false,
            name_contains: Some("Old Bakery".to_string()),
        };

        // 2. Setup Data (The change we want to make)
        let update_data = UpdateBakeryParam { new_profit: 900.0 };

        // 3. Define Models
        let original_model = Model {
            id: 1,
            name: "Old Bakery".to_owned(),
            profit: 100.0, // Old profit
        };

        let updated_model = Model {
            id: 1,
            name: "Old Bakery".to_owned(),
            profit: 900.0, // New profit
        };

        // 4. Setup Mock Database (Strict Order!)
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            // --- STEP 1: The 'Find' Operation ---
            // Your code calls E::find()...one(db) first.
            .append_query_results(vec![vec![original_model]])
            // --- STEP 2: The 'Refresh' Operation ---
            // Because MySQL doesn't support "UPDATE ... RETURNING",
            // SeaORM automatically runs a SELECT to get the fresh data.
            .append_query_results(vec![vec![updated_model]])
            .into_connection();

        // 5. Execute
        let result = update_one::<Entity, ActiveModel, Model, TestFilter, UpdateBakeryParam>(
            &db,
            filter,
            update_data,
        )
        .await;

        // 6. Assertions
        let model = result.expect("Update failed");

        assert_eq!(model.id, 1);
        assert_eq!(model.profit, 900.0); // Should be the NEW profit

        // 7. Verify SQL Order (Optional)
        let log = db.into_transaction_log();

        // Log[0] = SELECT (Find)
        assert!(format!("{:?}", log[0]).contains("SELECT"));

        // Log[1] = UPDATE (Execute)
        assert!(format!("{:?}", log[1]).contains("UPDATE"));
    }

    #[tokio::test]
    async fn test_update_many_success_mysql() {
        // 1. Setup Filter (Target: "Bad Bakery")
        let filter = TestFilter {
            page: 0,
            limit: 10,
            disable_limit: false,
            name_contains: Some("Bad Bakery".to_string()),
        };

        // 2. Setup Data (Action: Reset profit to 0.0)
        let data = UpdateProfitParam { new_profit: 0.0 };

        // 3. Setup Mock Database
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0, // Not relevant for updates
                rows_affected: 5,  // We pretend we updated 5 rows
            }])
            .into_connection();

        // 4. Execute
        let result =
            update_many::<Entity, ActiveModel, TestFilter, UpdateProfitParam>(&db, filter, data)
                .await;

        // 5. Assertions
        let rows_affected = result.expect("Update many failed");
        assert_eq!(rows_affected, 5);

        // 6. Verify SQL
        let log = db.into_transaction_log();
        let sql = format!("{:?}", log[0]);

        // Check for UPDATE structure
        assert!(sql.contains("UPDATE"));

        // Check for the SET clause
        // MySQL/Postgres: SET `profit` = ? or SET "profit" = $1
        assert!(sql.contains("profit"));

        // Check for the WHERE clause
        // It should contain the filter condition ("Bad Bakery")
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("Bad Bakery"));
    }

    #[tokio::test]
    async fn test_update_many_success_postgres() {
        // 1. Setup Filter (Target: "Bad Bakery")
        let filter = TestFilter {
            page: 0,
            limit: 10,
            disable_limit: false,
            name_contains: Some("Bad Bakery".to_string()),
        };

        // 2. Setup Data (Action: Reset profit to 0.0)
        let data = UpdateProfitParam { new_profit: 0.0 };

        // 3. Setup Mock Database
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0, // Not relevant for updates
                rows_affected: 5,  // We pretend we updated 5 rows
            }])
            .into_connection();

        // 4. Execute
        let result =
            update_many::<Entity, ActiveModel, TestFilter, UpdateProfitParam>(&db, filter, data)
                .await;

        // 5. Assertions
        let rows_affected = result.expect("Update many failed");
        assert_eq!(rows_affected, 5);

        // 6. Verify SQL
        let log = db.into_transaction_log();
        let sql = format!("{:?}", log[0]);

        // Check for UPDATE structure
        assert!(sql.contains("UPDATE"));

        // Check for the SET clause
        // MySQL/Postgres: SET `profit` = ? or SET "profit" = $1
        assert!(sql.contains("profit"));

        // Check for the WHERE clause
        // It should contain the filter condition ("Bad Bakery")
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("Bad Bakery"));
    }

    #[tokio::test]
    async fn test_delete_one_success_mysql() {
        use sea_orm::{DatabaseBackend, MockExecResult};

        // 1. Setup Filter
        let filter = TestFilter {
            page: 0,
            limit: 1,
            disable_limit: false,
            name_contains: Some("To Be Deleted".to_string()),
        };

        // 2. Setup the Model that "exists" in the DB
        let model_to_delete = Model {
            id: 99,
            name: "To Be Deleted".to_string(),
            profit: 0.0,
        };

        // 3. Setup Mock Database (Queue of 2)
        let db = MockDatabase::new(DatabaseBackend::MySql)
            // STEP 1: The 'Find' Operation
            // The function calls E::find()...one(db) first.
            .append_query_results(vec![vec![model_to_delete.clone()]])
            // STEP 2: The 'Delete' Execution
            // The function calls E::delete(active_model).exec(db).
            // This returns metadata (rows affected), not data.
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();

        // 4. Execute
        let result = delete_one::<Entity, ActiveModel, Model, TestFilter>(&db, filter).await;

        // 5. Assertions
        let deleted_model = result.expect("Delete failed");

        // Ensure we got back the data of the record we deleted
        assert_eq!(deleted_model.id, 99);
        assert_eq!(deleted_model.name, "To Be Deleted");

        // 6. Verify SQL Sequence
        let log = db.into_transaction_log();

        // Check that SELECT happened first
        let select_sql = format!("{:?}", log[0]);
        assert!(select_sql.contains("SELECT"));
        assert!(select_sql.contains("To Be Deleted")); // Filter applied

        // Check that DELETE happened second
        let delete_sql = format!("{:?}", log[1]);
        assert!(delete_sql.contains("DELETE"));
        // SeaORM deletes by Primary Key automatically when using ActiveModel
        // So it should look like: DELETE FROM `bakery` WHERE `id` = ?
        assert!(delete_sql.contains("WHERE"));

        // Note: Depending on backend, quote styles vary (`id` vs "id"),
        // but the ID 99 should be in the params.
    }

    #[tokio::test]
    async fn test_delete_one_success_postgres() {
        use sea_orm::{DatabaseBackend, MockExecResult};

        // 1. Setup Filter
        let filter = TestFilter {
            page: 0,
            limit: 1,
            disable_limit: false,
            name_contains: Some("To Be Deleted".to_string()),
        };

        // 2. Setup the Model that "exists" in the DB
        let model_to_delete = Model {
            id: 99,
            name: "To Be Deleted".to_string(),
            profit: 0.0,
        };

        // 3. Setup Mock Database (Queue of 2)
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            // STEP 1: The 'Find' Operation
            // The function calls E::find()...one(db) first.
            .append_query_results(vec![vec![model_to_delete.clone()]])
            // STEP 2: The 'Delete' Execution
            // The function calls E::delete(active_model).exec(db).
            // This returns metadata (rows affected), not data.
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();

        // 4. Execute
        let result = delete_one::<Entity, ActiveModel, Model, TestFilter>(&db, filter).await;

        // 5. Assertions
        let deleted_model = result.expect("Delete failed");

        // Ensure we got back the data of the record we deleted
        assert_eq!(deleted_model.id, 99);
        assert_eq!(deleted_model.name, "To Be Deleted");

        // 6. Verify SQL Sequence
        let log = db.into_transaction_log();

        // Check that SELECT happened first
        let select_sql = format!("{:?}", log[0]);
        assert!(select_sql.contains("SELECT"));
        assert!(select_sql.contains("To Be Deleted")); // Filter applied

        // Check that DELETE happened second
        let delete_sql = format!("{:?}", log[1]);
        assert!(delete_sql.contains("DELETE"));
        // SeaORM deletes by Primary Key automatically when using ActiveModel
        // So it should look like: DELETE FROM `bakery` WHERE `id` = ?
        assert!(delete_sql.contains("WHERE"));

        // Note: Depending on backend, quote styles vary (`id` vs "id"),
        // but the ID 99 should be in the params.
    }
}
