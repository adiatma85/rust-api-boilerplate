# AGENTS.md

## Commands
- **Build**: `cargo build` or `cargo check`
- **Lint**: `make lint` (runs `cargo clippy -- -D warnings`)
- **Format**: `make format` (uses nightly: `cargo +nightly fmt`)
- **Test all**: `cargo test`
- **Test single**: `cargo test <test_name>` or `cargo test <module>::<test_name>`
- **Run**: `cargo run` (requires decrypted config at `etc/cfg/conf.json`)

## Architecture
Layered Rust/Axum REST API with SeaORM (MySQL). Structure:
- `src/config/` - App settings, loaded from encrypted JSON via SOPS
- `src/domain/` - Data access layer (repositories)
- `src/entity/` - SeaORM entity definitions
- `src/handler/http/` - Axum HTTP handlers/routes
- `src/usecase/` - Business logic layer
- `src/state/` - App state (DB, JWT secret)
- `src/helper/` - Shared utilities

## Code Style
- Rust 2024 edition, format with nightly rustfmt
- Imports: group by std → external → crate (see `rustfmt.toml`)
- Use `thiserror` for error types, `async-trait` for async traits
- API docs via `utoipa` + Swagger UI
