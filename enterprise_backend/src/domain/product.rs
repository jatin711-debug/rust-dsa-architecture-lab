use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub stock_quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProductDto {
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i64,
    pub stock_quantity: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProductDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price_cents: Option<i64>,
    pub stock_quantity: Option<i32>,
}
