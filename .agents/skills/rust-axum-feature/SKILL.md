---
name: rust-axum-feature
description: Guide for scaffolding new features in a Rust Axum API using Clean Architecture. Use when adding new domain entities, business logic, or HTTP endpoints.
---

# Rust Axum Feature Scaffolding

This skill guides the creation of new features following the project's layered architecture.

## Workflow

When adding a new feature (e.g., "Product"), follow these steps:

### 1. Define the Entity
Create or update files in `src/business/entity/`.
- Define request/response structs.
- Implement common traits (e.g., `From<Row>`).
- Use `serde` for serialization.

### 2. Define the Domain (Data Access)
Create or update files in `src/business/domain/`.
- Define a trait for the repository/data access.
- Use `#[async_trait]` from the `async-trait` crate.

### 3. Implement the Usecase (Business Logic)
Create or update files in `src/business/usecase/`.
- Define the business logic struct.
- Inject the Domain trait as a dependency (usually via a generic or `Box<dyn Trait>`).
- Implement the business methods using `#[async_trait]`.

### 4. Create the HTTP Handler
Create or update files in `src/business/handler/http/rest/`.
- Define Axum handler functions.
- Use `axum::extract::State` to access the application state.
- Extract request bodies or path parameters.
- Call the Usecase methods and return appropriate `axum::response::Response`.

### 5. Wire the Routes
Update `src/business/handler/http/mod.rs` (or the relevant router file) to register the new routes and inject the necessary dependencies into the state.

## Rules
- **Consistency**: Always use `async-trait` for async methods in traits.
- **Error Handling**: Use the project's custom error types (defined in `src/business/entity/error.rs`).
- **Typing**: Prefer explicit types and avoid `unwrap()` in production code. Use `?` for error propagation.
