use crate::{
    domain::product::{CreateProductDto, Product, UpdateProductDto},
    error::AppError,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct ProductRepository;

impl ProductRepository {
    pub async fn create(pool: &PgPool, dto: CreateProductDto) -> Result<Product, AppError> {
        let description = dto.description.unwrap_or_default();

        let product = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (name, description, price_cents, stock_quantity)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, description, price_cents, stock_quantity, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(&description)
        .bind(dto.price_cents)
        .bind(dto.stock_quantity)
        .fetch_one(pool)
        .await?;

        Ok(product)
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<Product>, AppError> {
        let products = sqlx::query_as::<_, Product>(
            r#"
            SELECT id, name, description, price_cents, stock_quantity, created_at, updated_at
            FROM products
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(products)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Product>, AppError> {
        let product = sqlx::query_as::<_, Product>(
            r#"
            SELECT id, name, description, price_cents, stock_quantity, created_at, updated_at
            FROM products
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(product)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        dto: UpdateProductDto,
    ) -> Result<Product, AppError> {
        let existing = Self::find_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product with id {} not found", id)))?;

        let name = dto.name.unwrap_or(existing.name);
        let description = dto.description.unwrap_or(existing.description);
        let price_cents = dto.price_cents.unwrap_or(existing.price_cents);
        let stock_quantity = dto.stock_quantity.unwrap_or(existing.stock_quantity);

        let updated = sqlx::query_as::<_, Product>(
            r#"
            UPDATE products
            SET name = $1, description = $2, price_cents = $3, stock_quantity = $4, updated_at = NOW()
            WHERE id = $5
            RETURNING id, name, description, price_cents, stock_quantity, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(&description)
        .bind(price_cents)
        .bind(stock_quantity)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }
}
