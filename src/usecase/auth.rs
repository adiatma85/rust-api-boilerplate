use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::entity::user; // Import your User Entity

// 1. JWT Claims Struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // Subject (Email)
    pub user_id: i32,
    pub exp: usize, // Expiration
    pub iat: usize, // Issued At
}

// 2. The Params Struct
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

// 3. The Usecase Struct
pub struct AuthUsecase {
    db: DatabaseConnection,
    jwt_secret: String,
}

pub struct AuthUseInitParam {
    pub db: DatabaseConnection,
    pub jwt_secret: String,
}

impl AuthUsecase {
    pub fn new(params: AuthUseInitParam) -> Self {
        Self {
            db: params.db,
            jwt_secret: params.jwt_secret,
        }
    }

    // --- Helper: Hash Password (Public so UserUsecase can use it) ---
    // We make this a 'static' associated function because it doesn't need DB or Secret
    pub fn hash_password(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| e.to_string())
            .map(|h| h.to_string())
    }

    // --- Login Logic ---
    pub async fn login(&self, params: LoginParams) -> Result<String, String> {
        // A. Find User
        let user_model = user::Entity::find()
            .filter(user::Column::Email.eq(params.email))
            .one(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Invalid email or password")?;

        // B. Verify Password
        let parsed_hash = PasswordHash::new(&user_model.hashed_password)
            .map_err(|_| "Invalid password hash in DB")?;

        Argon2::default()
            .verify_password(params.password.as_bytes(), &parsed_hash)
            .map_err(|_| "Invalid email or password")?;

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
        .map_err(|e| e.to_string())?;

        Ok(token)
    }
}
