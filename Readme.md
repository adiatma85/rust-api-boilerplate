# Gemini Assisted Axum API

This repository contains a REST API service built with **Rust**, **Axum**, and **SeaORM**. It follows a Clean Architecture pattern and features interactive API documentation via **Swagger UI** (Utoipa).

## 🏗️ Architecture

The project is structured using a layered architecture:
- **`src/config/`**: Application configuration (loaded via SOPS).
- **`src/domain/`**: Data access layer (repositories).
- **`src/entity/`**: Database entities (SeaORM).
- **`src/handler/`**: HTTP handlers and routes (Axum).
- **`src/usecase/`**: core business logic.
- **`src/state/`**: Application state management.
- **`src/helper/`**: Shared utilities.

## 📋 Pre-requisites

Before starting, ensure you have the following installed:
- **[Docker](https://www.docker.com/)**: Required to run the MySQL database.
- **[Rust](https://www.rust-lang.org/)**: Minimal version supporting Edition 2024 (Nightly is recommended for formatting).

## 🚀 Getting Started

Follow these steps to set up the project locally.

### 1. Initialize the Project
Run the following command to install necessary tools (like SOPS, Ratchet) and setup git hooks:
```bash
make init
```

### 2. Configuration
You need to configure the application before running it.
- Create your own configuration file at `etc/cfg/conf.json`.
- **Note**: If you have access to the encrypted secrets, you can decrypt the local configuration using:
  ```bash
  make decrypt-local
  ```

### 3. Database Migration
Ensure your MySQL database is running and apply the migrations located in:
```text
docs/sql/init.sh
```

### 4. Run the Application
Once configured, you can start the application with:
```bash
make run
```

## 🛠️ Essential Make Commands

The `makefile` provides several commands to streamline development:

### Development
- **`make init`**: Initialize the project (install tools, setup hooks).
- **`make run`**: Run the application (`cargo run`).
- **`make check`**: check the code and workflow pins.
- **`make test`**: Run all unit tests.
- **`make clean`**: Clean up build artifacts and tools.

### Code Quality
- **`make lint`**: Run clippy lints (`cargo clippy -- -D warnings`).
- **`make format`**: Format code using nightly rustfmt (`cargo +nightly fmt`).
- **`make format-check`**: Check code formatting without modifying files.
- **`make sort`**: Sort dependencies in `Cargo.toml`.

### Configuration (SOPS)
- **`make decrypt-local`**: Decrypt `conf.local.enc.json` to `conf.json` for local use.
- **`make encrypt-local`**: Encrypt `conf.json` to `conf.local.enc.json`.
- **`make conf-local`**: Safely edit the local encrypted configuration.
*(Replace `local` with `staging` or `prod` for other environments)*

### Tools
- **`make install-tools`**: Install required binaries (age, sops, ratchet) to `./bin`.
- **`make ratchet-check`**: Check if GitHub Actions workflows are pinned.
- **`make ratchet-update`**: Pin GitHub Actions workflows to the latest SHAs.
