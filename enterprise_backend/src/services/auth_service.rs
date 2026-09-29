use crate::{domain::user::Claims, error::AppError};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

pub struct AuthService;

impl AuthService {
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AppError::Internal(format!("Password hashing error: {}", e)))
    }

    pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(password_hash).map_err(|e| {
            AppError::Internal(format!("Invalid stored password hash format: {}", e))
        })?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn generate_jwt(
        user_id: Uuid,
        email: &str,
        role: &str,
        jwt_secret: &str,
        expiration_hours: i64,
    ) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = (now + chrono::Duration::hours(expiration_hours)).timestamp() as usize;

        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            role: role.to_string(),
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("JWT generation error: {}", e)))
    }

    pub fn verify_jwt(token: &str, jwt_secret: &str) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        Ok(token_data.claims)
    }
}
