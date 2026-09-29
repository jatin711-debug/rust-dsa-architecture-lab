use crate::{
    domain::order::{
        CreateOrderDto, Order, OrderItem, OrderItemResponse, OrderResponse, OrderStatus,
    },
    error::AppError,
};
use sqlx::PgPool;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

pub struct OrderRepository;

#[derive(sqlx::FromRow)]
struct ProductLockRow {
    name: String,
    price_cents: i64,
    stock_quantity: i32,
}

/// Validate and combine repeated products before starting a transaction.
/// BTreeMap gives every order the same row-lock order, avoiding a common
/// deadlock pattern when two customers buy overlapping products.
fn prepare_order_items(dto: &CreateOrderDto) -> Result<Vec<(Uuid, i32)>, AppError> {
    if dto.items.is_empty() {
        return Err(AppError::BadRequest(
            "Cannot create an order with zero items".to_string(),
        ));
    }
    let mut quantities = BTreeMap::<Uuid, i32>::new();
    for item in &dto.items {
        if item.quantity <= 0 {
            return Err(AppError::BadRequest(
                "Item quantity must be greater than zero".to_string(),
            ));
        }
        let quantity = quantities.entry(item.product_id).or_default();
        *quantity = quantity.checked_add(item.quantity).ok_or_else(|| {
            AppError::BadRequest("Combined item quantity is too large".to_string())
        })?;
    }
    Ok(quantities.into_iter().collect())
}

impl OrderRepository {
    pub async fn create_order_transaction(
        pool: &PgPool,
        user_id: Uuid,
        dto: CreateOrderDto,
    ) -> Result<OrderResponse, AppError> {
        let items = prepare_order_items(&dto)?;

        // Begin SQL Transaction
        let mut tx = pool.begin().await?;

        let mut total_amount_cents: i64 = 0;
        let mut prepared_items: Vec<(Uuid, i64, i32)> = Vec::new();

        for (product_id, quantity) in items {
            // Lock row for update to prevent concurrent race conditions
            let product = sqlx::query_as::<_, ProductLockRow>(
                r#"
                SELECT name, price_cents, stock_quantity
                FROM products
                WHERE id = $1
                FOR UPDATE
                "#,
            )
            .bind(product_id)
            .fetch_optional(&mut *tx)
            .await?;

            let product = product.ok_or_else(|| {
                AppError::NotFound(format!("Product with id {product_id} not found"))
            })?;

            if product.stock_quantity < quantity {
                return Err(AppError::BadRequest(format!(
                    "Insufficient stock for product '{}'. Requested: {}, Available: {}",
                    product.name, quantity, product.stock_quantity
                )));
            }

            // Decrement stock
            sqlx::query(
                r#"
                UPDATE products
                SET stock_quantity = stock_quantity - $1, updated_at = NOW()
                WHERE id = $2
                "#,
            )
            .bind(quantity)
            .bind(product_id)
            .execute(&mut *tx)
            .await?;

            let item_total = product
                .price_cents
                .checked_mul(i64::from(quantity))
                .ok_or_else(|| AppError::BadRequest("Order total is too large".to_string()))?;
            total_amount_cents = total_amount_cents
                .checked_add(item_total)
                .ok_or_else(|| AppError::BadRequest("Order total is too large".to_string()))?;

            prepared_items.push((product_id, product.price_cents, quantity));
        }

        // Insert Order record
        let order = sqlx::query_as::<_, Order>(
            r#"
            INSERT INTO orders (user_id, status, total_amount_cents)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, status, total_amount_cents, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(OrderStatus::Pending.to_string())
        .bind(total_amount_cents)
        .fetch_one(&mut *tx)
        .await?;

        let mut item_responses = Vec::new();

        // Insert Order Items
        for (product_id, unit_price_cents, quantity) in prepared_items {
            let item = sqlx::query_as::<_, OrderItem>(
                r#"
                INSERT INTO order_items (order_id, product_id, unit_price_cents, quantity)
                VALUES ($1, $2, $3, $4)
                RETURNING id, order_id, product_id, unit_price_cents, quantity, created_at
                "#,
            )
            .bind(order.id)
            .bind(product_id)
            .bind(unit_price_cents)
            .bind(quantity)
            .fetch_one(&mut *tx)
            .await?;

            item_responses.push(OrderItemResponse {
                id: item.id,
                product_id: item.product_id,
                unit_price_cents: item.unit_price_cents,
                quantity: item.quantity,
            });
        }

        // Commit transaction
        tx.commit().await?;

        Ok(OrderResponse {
            id: order.id,
            user_id: order.user_id,
            status: order.status,
            total_amount_cents: order.total_amount_cents,
            items: item_responses,
            created_at: order.created_at,
        })
    }

    pub async fn list_by_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<OrderResponse>, AppError> {
        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT id, user_id, status, total_amount_cents, created_at, updated_at
            FROM orders
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        if orders.is_empty() {
            return Ok(Vec::new());
        }
        let order_ids: Vec<Uuid> = orders.iter().map(|order| order.id).collect();
        // One query for all children avoids one extra round trip per order.
        let items = sqlx::query_as::<_, OrderItem>(
            r#"
            SELECT id, order_id, product_id, unit_price_cents, quantity, created_at
            FROM order_items
            WHERE order_id = ANY($1)
            ORDER BY created_at, id
            "#,
        )
        .bind(&order_ids)
        .fetch_all(pool)
        .await?;
        let mut items_by_order: HashMap<Uuid, Vec<OrderItemResponse>> = HashMap::new();
        for item in items {
            items_by_order
                .entry(item.order_id)
                .or_default()
                .push(OrderItemResponse {
                    id: item.id,
                    product_id: item.product_id,
                    unit_price_cents: item.unit_price_cents,
                    quantity: item.quantity,
                });
        }

        let mut responses = Vec::with_capacity(orders.len());
        for order in orders {
            responses.push(OrderResponse {
                id: order.id,
                user_id: order.user_id,
                status: order.status,
                total_amount_cents: order.total_amount_cents,
                items: items_by_order.remove(&order.id).unwrap_or_default(),
                created_at: order.created_at,
            });
        }

        Ok(responses)
    }

    pub async fn find_by_id(
        pool: &PgPool,
        order_id: Uuid,
    ) -> Result<Option<OrderResponse>, AppError> {
        let order = sqlx::query_as::<_, Order>(
            r#"
            SELECT id, user_id, status, total_amount_cents, created_at, updated_at
            FROM orders
            WHERE id = $1
            "#,
        )
        .bind(order_id)
        .fetch_optional(pool)
        .await?;

        let order = match order {
            Some(o) => o,
            None => return Ok(None),
        };

        let items = sqlx::query_as::<_, OrderItem>(
            r#"
            SELECT id, order_id, product_id, unit_price_cents, quantity, created_at
            FROM order_items
            WHERE order_id = $1
            "#,
        )
        .bind(order.id)
        .fetch_all(pool)
        .await?;

        let item_responses = items
            .into_iter()
            .map(|item| OrderItemResponse {
                id: item.id,
                product_id: item.product_id,
                unit_price_cents: item.unit_price_cents,
                quantity: item.quantity,
            })
            .collect();

        Ok(Some(OrderResponse {
            id: order.id,
            user_id: order.user_id,
            status: order.status,
            total_amount_cents: order.total_amount_cents,
            items: item_responses,
            created_at: order.created_at,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::order::OrderItemInputDto;

    #[test]
    fn prepares_sorted_consolidated_items() {
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);
        let dto = CreateOrderDto {
            items: vec![
                OrderItemInputDto {
                    product_id: b,
                    quantity: 1,
                },
                OrderItemInputDto {
                    product_id: a,
                    quantity: 2,
                },
                OrderItemInputDto {
                    product_id: b,
                    quantity: 3,
                },
            ],
        };
        assert_eq!(prepare_order_items(&dto).unwrap(), vec![(a, 2), (b, 4)]);
    }

    #[test]
    fn rejects_invalid_or_overflowing_quantity() {
        let id = Uuid::new_v4();
        for quantities in [vec![], vec![0], vec![-1], vec![i32::MAX, 1]] {
            let dto = CreateOrderDto {
                items: quantities
                    .into_iter()
                    .map(|quantity| OrderItemInputDto {
                        product_id: id,
                        quantity,
                    })
                    .collect(),
            };
            assert!(prepare_order_items(&dto).is_err());
        }
    }
}
