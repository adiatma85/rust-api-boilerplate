# Project Instructions: Rust API Boilerplate

This project is a REST API service built with **Rust**, **Axum**, and **SeaORM**. It follows a **Clean Architecture** pattern and uses **SOPS** for secret management and **Utoipa** for OpenAPI documentation.

## Architecture Overview

The project follows a layered architecture located primarily in `src/business/`:

1.  **Handler (`src/business/handler/`)**: Axum HTTP handlers, route definitions, and middlewares.
2.  **Usecase (`src/business/usecase/`)**: Core business logic. Depends on Domain traits.
3.  **Domain (`src/business/domain/`)**: Data access layer (repositories). Defines traits and implements them (usually with SeaORM).
4.  **Entity (`src/business/entity/`)**: Data models, request/response structs, and shared types (e.g., `AppError`).

### Supporting Modules
-   **`src/config/`**: Configuration loading and management (via `config` crate and SOPS).
-   **`src/state/`**: Application state (`AppState`) shared across handlers.
-   **`src/helper/`**: Shared utilities like logging and password hashing.
-   **`etc/cfg/`**: Configuration files. `.enc.json` files are encrypted with SOPS.

## Development Workflow

### Building and Running
-   **Initialize Project**: `make init` (Installs tools like SOPS, Ratchet, and sets up git hooks).
-   **Run Application**: `make run` (Starts the server, typically on `http://127.0.0.1:8080`).
-   **Test**: `make test` (Runs cargo tests).
-   **Linting**: `make lint` (Runs Clippy).
-   **Formatting**: `make format` (Runs nightly rustfmt).

### Configuration & Secrets
This project uses **SOPS** with **age** for secret management.
-   **Decrypt local config**: `make decrypt-local` (Creates `etc/cfg/conf.json`).
-   **Edit encrypted config**: `make conf-local` (Opens the encrypted file in your editor and re-encrypts on save).
-   **Key Location**: SOPS expects the age key at `.secrets/keys.txt` (or via `SOPS_AGE_KEY` env var).

### API Documentation
-   **Swagger UI**: Available at `/swagger-ui` when the application is running.
-   **Spec**: The OpenAPI spec is generated using `utoipa` macros on handlers and models.

## Development Conventions

-   **Clean Architecture**: Always follow the layer boundaries. Use traits for dependencies between layers to facilitate testing/mocking.
-   **Async Traits**: Use `#[async_trait]` for all traits defining async methods (from the `async-trait` crate).
-   **Error Handling**: Use the `AppError` enum in `src/business/entity/error.rs` for domain-specific errors. Map database errors (SeaORM) to `AppError`.
-   **Entity Mapping**: Use `From` or `Into` traits for converting between database entities and API response models.
-   **State Management**: Access application state in handlers via `axum::extract::State<AppState>`.
-   **Git Hooks**: The project uses a pre-commit hook (configured via `make init`) that checks formatting and lints.

## Key Files
-   `src/main.rs`: Entry point, initializes layers and starts the server.
-   `src/business/handler/http/rest/mod.rs`: Central route registration.
-   `Cargo.toml`: Dependency management (Rust 2024 edition).
-   `makefile`: Primary interface for development tasks.
