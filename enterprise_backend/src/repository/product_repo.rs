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
        if dto.name.as_ref().is_some_and(|name| name.trim().is_empty()) {
            return Err(AppError::BadRequest("Product name is required".to_string()));
        }
        if dto.price_cents.is_some_and(|price| price < 0) {
            return Err(AppError::BadRequest(
                "Price must be non-negative".to_string(),
            ));
        }
        if dto.stock_quantity.is_some_and(|stock| stock < 0) {
            return Err(AppError::BadRequest(
                "Stock quantity must be non-negative".to_string(),
            ));
        }

        // One statement updates only supplied fields. A separate read followed
        // by a full-row write could overwrite another request's disjoint edit.
        let updated = sqlx::query_as::<_, Product>(
            r#"
            UPDATE products
            SET name = COALESCE($1, name),
                description = COALESCE($2, description),
                price_cents = COALESCE($3, price_cents),
                stock_quantity = COALESCE($4, stock_quantity),
                updated_at = NOW()
            WHERE id = $5
            RETURNING id, name, description, price_cents, stock_quantity, created_at, updated_at
            "#,
        )
        .bind(dto.name)
        .bind(dto.description)
        .bind(dto.price_cents)
        .bind(dto.stock_quantity)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product with id {id} not found")))?;

        Ok(updated)
    }
}
