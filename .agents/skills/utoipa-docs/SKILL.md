---
name: utoipa-docs
description: Procedures for maintaining OpenAPI/Swagger documentation using Utoipa macros. Use when adding or modifying API endpoints, request/response models.
---

# Utoipa API Documentation

This skill guides the maintenance of interactive API documentation (Swagger UI) using the `utoipa` crate.

## Workflow

### 1. Annotate Handlers
Every Axum handler exposed in the API should have a `#[utoipa::path(...)]` attribute.
```rust
#[utoipa::path(
    get,
    path = "/api/v1/resource",
    responses(
        (status = 200, description = "Success", body = ResourceResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_resource(...) -> ... {}
```

### 2. Derive ToSchema
All request and response structs must derive `utoipa::ToSchema`.
```rust
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ResourceResponse {
    pub id: uuid::Uuid,
    pub name: String,
}
```

### 3. Register in ApiDoc
New paths and schemas must be registered in the central `ApiDoc` struct, typically found in `src/business/handler/http/doc/mod.rs`.

```rust
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        crate::business::handler::http::rest::resource::get_resource,
        // Add new paths here
    ),
    components(
        schemas(
            crate::business::entity::resource::ResourceResponse,
            // Add new schemas here
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
```

## Rules
- **Descriptions**: Always provide a clear `description` for status codes.
- **Security**: Include `security(("bearer_auth" = []))` for endpoints requiring authentication.
- **Sync**: Ensure that any change to an endpoint's path or body is reflected in the `#[utoipa::path]` macro.
