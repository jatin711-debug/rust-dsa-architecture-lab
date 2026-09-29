use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum OrderStatus {
    Pending,
    Paid,
    Shipped,
    Completed,
    Cancelled,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "Pending"),
            OrderStatus::Paid => write!(f, "Paid"),
            OrderStatus::Shipped => write!(f, "Shipped"),
            OrderStatus::Completed => write!(f, "Completed"),
            OrderStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::str::FromStr for OrderStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(OrderStatus::Pending),
            "Paid" => Ok(OrderStatus::Paid),
            "Shipped" => Ok(OrderStatus::Shipped),
            "Completed" => Ok(OrderStatus::Completed),
            "Cancelled" => Ok(OrderStatus::Cancelled),
            _ => Err(format!("Unknown order status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub total_amount_cents: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub unit_price_cents: i64,
    pub quantity: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OrderItemInputDto {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrderDto {
    pub items: Vec<OrderItemInputDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderItemResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub unit_price_cents: i64,
    pub quantity: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub total_amount_cents: i64,
    pub items: Vec<OrderItemResponse>,
    pub created_at: DateTime<Utc>,
}
