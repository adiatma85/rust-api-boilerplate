use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};

use crate::business::{
    domain::user::UserDomainTrait,
    entity::{self, auth::Claims, response::AppCode},
};

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
        let user_dom_param = crate::business::entity::user::UserDomParam {
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

#[cfg(test)]
mod tests {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    use sea_orm::prelude::DateTimeUtc;

    use super::*;
    use crate::business::{
        domain::user::MockUserDomainTrait,
        entity::{auth::LoginParams, user},
    };

    fn create_hashed_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    fn create_test_user(password: &str) -> user::Model {
        user::Model {
            id: 1,
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            hashed_password: create_hashed_password(password),
            status: 1,
            created_at: DateTimeUtc::from(Utc::now()),
            updated_at: DateTimeUtc::from(Utc::now()),
        }
    }

    #[tokio::test]
    async fn test_login_success() {
        let password = "password123";
        let user = create_test_user(password);

        let mut mock_user_domain = MockUserDomainTrait::new();
        mock_user_domain
            .expect_get_one()
            .times(1)
            .returning(move |_| Ok(user.clone()));

        let auth_usecase = init(InitParam {
            jwt_secret: "secret".to_string(),
            user_domain: Arc::new(mock_user_domain),
        });

        let result = auth_usecase
            .login(LoginParams {
                email: "test@example.com".to_string(),
                password: password.to_string(),
            })
            .await;

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let mut mock_user_domain = MockUserDomainTrait::new();
        mock_user_domain
            .expect_get_one()
            .times(1)
            .returning(|_| Err(crate::business::entity::error::AppError::NotFound));

        let auth_usecase = init(InitParam {
            jwt_secret: "secret".to_string(),
            user_domain: Arc::new(mock_user_domain),
        });

        let result = auth_usecase
            .login(LoginParams {
                email: "notfound@example.com".to_string(),
                password: "password".to_string(),
            })
            .await;

        match result {
            Err(AppCode::NotFound) => assert!(true),
            _ => panic!("Expected AppCode::NotFound, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let password = "password123";
        let user = create_test_user(password);

        // Mock returning the user, but we'll send a wrong password
        let mut mock_user_domain = MockUserDomainTrait::new();
        mock_user_domain
            .expect_get_one()
            .times(1)
            .returning(move |_| Ok(user.clone()));

        let auth_usecase = init(InitParam {
            jwt_secret: "secret".to_string(),
            user_domain: Arc::new(mock_user_domain),
        });

        let result = auth_usecase
            .login(LoginParams {
                email: "test@example.com".to_string(),
                password: "wrongpassword".to_string(),
            })
            .await;

        match result {
            Err(AppCode::Unauthorized) => assert!(true),
            _ => panic!("Expected AppCode::Unauthorized, got {:?}", result),
        }
    }
}
