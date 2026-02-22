use utoipa::{
    OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

use crate::business::entity::{
    card::{CreateCardRequest, UpdateCardStatusRequest},
    user::{LoginRequest, RegisterRequest, UserUseResponse},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Util
        crate::business::handler::http::rest::util::ping_handler,

        // Auth Handler
        crate::business::handler::http::rest::user::create_user_handler,
        crate::business::handler::http::rest::user::login_handler,

        // User Handler
        crate::business::handler::http::rest::user::get_user_list_handler,

        // Card Handler
        crate::business::handler::http::rest::card::create_card_handler,
        crate::business::handler::http::rest::card::get_card_handler,
        crate::business::handler::http::rest::card::get_card_list_handler,
        crate::business::handler::http::rest::card::update_card_status_handler,
        crate::business::handler::http::rest::card::delete_card_handler,
    ),
    components(
        schemas(CreateCardRequest, UpdateCardStatusRequest, RegisterRequest, LoginRequest, UserUseResponse)
    ),
    // 1. ADD MODIFIERS HERE
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Cards", description = "Card management endpoints"),
        (name = "Users", description = "User management endpoints"),
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
