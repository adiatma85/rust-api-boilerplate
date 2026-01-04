use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};

use crate::{
    domain::user::UserDomainTrait,
    entity::{self, auth::Claims, response::AppCode},
}; // Import your User Entity

// 3. The Usecase Struct

#[async_trait]
pub trait AuthUsecaseTrait: Send + Sync {
    async fn login(&self, params: entity::auth::LoginParams) -> Result<String, AppCode>;
}

pub struct AuthUsecase {
    jwt_secret: String,
    user_domain: Arc<dyn UserDomainTrait>,
}

pub struct InitParam {
    pub jwt_secret: String,
    pub user_domain: Arc<dyn UserDomainTrait>,
}

pub fn init(param: InitParam) -> impl AuthUsecaseTrait {
    AuthUsecase {
        jwt_secret: param.jwt_secret,
        user_domain: param.user_domain,
    }
}

#[async_trait]
impl AuthUsecaseTrait for AuthUsecase {
    async fn login(&self, params: entity::auth::LoginParams) -> Result<String, AppCode> {
        let user_dom_param = crate::entity::user::UserDomParam {
            email_eq: Some(params.email),
            ..Default::default()
        };

        let user_model = self
            .user_domain
            .get_one(user_dom_param)
            .await
            .map_err(AppCode::from)?;

        let parsed_hash =
            PasswordHash::new(&user_model.hashed_password).map_err(|_| AppCode::Unauthorized)?;

        // This could be written in the helper instead in here
        Argon2::default()
            .verify_password(params.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppCode::Unauthorized)?;

        // C. Generate JWT
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_model.email,
            user_id: user_model.id,
            exp: expiration as usize,
            iat: Utc::now().timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AppCode::Unauthorized)?;

        Ok(token)
    }
}
