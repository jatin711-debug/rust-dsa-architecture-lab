use crate::{
    domain::user::{CreateUserDto, Role, User},
    error::AppError,
    services::auth_service::AuthService,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserRepository;

impl UserRepository {
    pub async fn create(pool: &PgPool, dto: CreateUserDto) -> Result<User, AppError> {
        let password_hash = AuthService::hash_password(&dto.password)?;
        // Public registration never accepts a caller-selected privileged role.
        let role = Role::Customer.to_string();

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, full_name, role)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, password_hash, full_name, role, created_at, updated_at
            "#,
        )
        .bind(&dto.email)
        .bind(&password_hash)
        .bind(&dto.full_name)
        .bind(&role)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, full_name, role, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, full_name, role, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }
}
