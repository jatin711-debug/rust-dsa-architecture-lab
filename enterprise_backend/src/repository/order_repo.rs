use crate::{
    domain::order::{
        CreateOrderDto, Order, OrderItem, OrderItemResponse, OrderResponse, OrderStatus,
    },
    error::AppError,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct OrderRepository;

#[derive(sqlx::FromRow)]
struct ProductLockRow {
    name: String,
    price_cents: i64,
    stock_quantity: i32,
}

impl OrderRepository {
    pub async fn create_order_transaction(
        pool: &PgPool,
        user_id: Uuid,
        dto: CreateOrderDto,
    ) -> Result<OrderResponse, AppError> {
        if dto.items.is_empty() {
            return Err(AppError::BadRequest(
                "Cannot create an order with zero items".to_string(),
            ));
        }

        // Begin SQL Transaction
        let mut tx = pool.begin().await?;

        let mut total_amount_cents: i64 = 0;
        let mut prepared_items: Vec<(Uuid, i64, i32)> = Vec::new();

        for item in &dto.items {
            if item.quantity <= 0 {
                return Err(AppError::BadRequest(
                    "Item quantity must be greater than zero".to_string(),
                ));
            }

            // Lock row for update to prevent concurrent race conditions
            let product = sqlx::query_as::<_, ProductLockRow>(
                r#"
                SELECT name, price_cents, stock_quantity
                FROM products
                WHERE id = $1
                FOR UPDATE
                "#,
            )
            .bind(item.product_id)
            .fetch_optional(&mut *tx)
            .await?;

            let product = product.ok_or_else(|| {
                AppError::NotFound(format!("Product with id {} not found", item.product_id))
            })?;

            if product.stock_quantity < item.quantity {
                return Err(AppError::BadRequest(format!(
                    "Insufficient stock for product '{}'. Requested: {}, Available: {}",
                    product.name, item.quantity, product.stock_quantity
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
            .bind(item.quantity)
            .bind(item.product_id)
            .execute(&mut *tx)
            .await?;

            let item_total = product.price_cents * (item.quantity as i64);
            total_amount_cents += item_total;

            prepared_items.push((item.product_id, product.price_cents, item.quantity));
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

        let mut responses = Vec::new();

        for order in orders {
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

            responses.push(OrderResponse {
                id: order.id,
                user_id: order.user_id,
                status: order.status,
                total_amount_cents: order.total_amount_cents,
                items: item_responses,
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
