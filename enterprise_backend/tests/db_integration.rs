#![cfg(feature = "db-integration")]

//! SQLx creates a fresh database per test and applies migrations. Run only
//! against a disposable PostgreSQL server with a role allowed to create DBs.

use enterprise_backend::domain::order::{CreateOrderDto, OrderItemInputDto};
use enterprise_backend::domain::product::UpdateProductDto;
use enterprise_backend::repository::order_repo::OrderRepository;
use enterprise_backend::repository::product_repo::ProductRepository;
use sqlx::PgPool;
use uuid::Uuid;

async fn user(pool: &PgPool, email: &str) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO users (email, password_hash, full_name) VALUES ($1, 'test-hash', 'Test') RETURNING id",
    )
    .bind(email)
    .fetch_one(pool)
    .await
}

async fn product(pool: &PgPool, stock: i32, price: i64) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO products (name, price_cents, stock_quantity) VALUES ('Widget', $1, $2) RETURNING id",
    )
    .bind(price)
    .bind(stock)
    .fetch_one(pool)
    .await
}

fn order(product_id: Uuid, quantity: i32) -> CreateOrderDto {
    CreateOrderDto {
        items: vec![OrderItemInputDto {
            product_id,
            quantity,
        }],
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn competing_orders_cannot_oversell(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let product_id = product(&pool, 1, 250).await?;
    let alice = user(&pool, "alice@example.com").await?;
    let bob = user(&pool, "bob@example.com").await?;

    let (first, second) = tokio::join!(
        OrderRepository::create_order_transaction(&pool, alice, order(product_id, 1)),
        OrderRepository::create_order_transaction(&pool, bob, order(product_id, 1)),
    );
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    let stock: i32 = sqlx::query_scalar("SELECT stock_quantity FROM products WHERE id = $1")
        .bind(product_id)
        .fetch_one(&pool)
        .await?;
    let orders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders")
        .fetch_one(&pool)
        .await?;
    assert_eq!(stock, 0);
    assert_eq!(orders, 1);
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn duplicate_lines_are_combined_and_listed(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let product_id = product(&pool, 5, 120).await?;
    let user_id = user(&pool, "reader@example.com").await?;
    let dto = CreateOrderDto {
        items: vec![
            OrderItemInputDto {
                product_id,
                quantity: 1,
            },
            OrderItemInputDto {
                product_id,
                quantity: 2,
            },
        ],
    };
    let created = OrderRepository::create_order_transaction(&pool, user_id, dto).await?;
    assert_eq!(created.total_amount_cents, 360);
    assert_eq!(created.items.len(), 1);
    assert_eq!(created.items[0].quantity, 3);
    let listed = OrderRepository::list_by_user(&pool, user_id).await?;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].items.len(), 1);
    assert_eq!(listed[0].items[0].quantity, 3);
    let stock: i32 = sqlx::query_scalar("SELECT stock_quantity FROM products WHERE id = $1")
        .bind(product_id)
        .fetch_one(&pool)
        .await?;
    assert_eq!(stock, 2);
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn disjoint_product_edits_do_not_overwrite_each_other(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let id = product(&pool, 5, 120).await?;
    let (name, price) = tokio::join!(
        ProductRepository::update(
            &pool,
            id,
            UpdateProductDto {
                name: Some("Renamed".to_string()),
                description: None,
                price_cents: None,
                stock_quantity: None,
            }
        ),
        ProductRepository::update(
            &pool,
            id,
            UpdateProductDto {
                name: None,
                description: None,
                price_cents: Some(250),
                stock_quantity: None,
            }
        ),
    );
    name?;
    price?;
    let product = ProductRepository::find_by_id(&pool, id).await?.unwrap();
    assert_eq!(product.name, "Renamed");
    assert_eq!(product.price_cents, 250);
    assert_eq!(product.stock_quantity, 5);
    Ok(())
}
