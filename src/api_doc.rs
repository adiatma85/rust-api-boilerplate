use utoipa::OpenApi;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};

use crate::handler::card::{CreateCardRequest, UpdateCardStatusRequest};
use crate::handler::user::{LoginRequest, RegisterRequest};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Util
        crate::usecase::util::health_check_handler,

        // Auth Handler
        crate::handler::user::create_user_handler,
        crate::handler::user::login_handler,

        // Card Handler
        crate::handler::card::create_card_handler,
        crate::handler::card::update_card_status_handler,
        crate::handler::card::delete_card_handler
    ),
    components(
        schemas(CreateCardRequest, UpdateCardStatusRequest, RegisterRequest, LoginRequest)
    ),
    // 1. ADD MODIFIERS HERE
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Cards", description = "Card management endpoints"),
    )
)]
pub struct ApiDoc;

// 2. DEFINE THE SECURITY ADDON
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // specific to utoipa version, might need check
        components.add_security_scheme(
            "bearer_auth", // This name matches the one in the handler annotations
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}
