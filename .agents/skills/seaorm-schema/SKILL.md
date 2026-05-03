---
name: seaorm-schema
description: Guidelines for database schema management and SeaORM entity synchronization. Use when adding new tables, modifying columns, or updating database relationships.
---

# SeaORM and Schema Management

This skill provides a workflow for managing the database schema and synchronizing it with Rust entities.

## Database Migrations

Migrations in this project are managed using raw SQL scripts.
- Location: `docs/sql/` (organized by year, e.g., `docs/sql/2025/`).
- Initialization script: `docs/sql/init.sh`.

### Adding a New Migration
1. Create a new `.sql` file in the appropriate directory within `docs/sql/`.
2. Write the standard SQL (DDL) for creating or altering tables.
3. Update `docs/sql/init.sh` if it needs to include the new script.

## SeaORM Entities

Rust entities are located in `src/business/entity/` and use the `sea_orm` crate.

### Updating Entities
When the database schema changes, you must manually update the corresponding Rust entity:
- Add or remove fields in the `Model` struct.
- Update the `Column` enum.
- Update `Relation` and `ActiveModelBehavior` if necessary.
- Ensure proper mapping for types like `Uuid` and `DateTime<FixedOffset>` (Chrono).

Example:
```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub username: String,
    // ...
}
```

## Rules
- **MySQL**: The project uses MySQL. Ensure SQL syntax is compatible.
- **Manual Sync**: This project does not use SeaORM CLI for migrations; entities must be kept in sync manually with the SQL scripts in `docs/sql/`.
- **Testing**: After updating a schema, run `cargo check` to ensure entities still compile correctly.
