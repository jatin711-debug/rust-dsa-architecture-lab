use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "PascalCase")]
pub enum Role {
    #[default]
    Customer,
    StoreAdmin,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Customer => write!(f, "Customer"),
            Role::StoreAdmin => write!(f, "StoreAdmin"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Customer" => Ok(Role::Customer),
            "StoreAdmin" => Ok(Role::StoreAdmin),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateUserDto {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginUserDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            role: user.role,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthTokenResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,     // User ID
    pub email: String, // Email
    pub role: String,  // Role string
    pub exp: usize,    // Expiration time (unix timestamp)
}
