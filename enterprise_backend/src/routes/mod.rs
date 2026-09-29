pub mod auth_routes;
pub mod health_routes;
pub mod order_routes;
pub mod product_routes;

use crate::state::AppState;
use axum::{
    routing::{get, post, put},
    Router,
};

pub fn create_router() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(health_routes::health_check))
        // Auth routes
        .route("/api/v1/auth/register", post(auth_routes::register))
        .route("/api/v1/auth/login", post(auth_routes::login))
        .route("/api/v1/auth/me", get(auth_routes::me))
        // Product routes
        .route("/api/v1/products", get(product_routes::list_products))
        .route("/api/v1/products", post(product_routes::create_product))
        .route("/api/v1/products/{id}", get(product_routes::get_product))
        .route("/api/v1/products/{id}", put(product_routes::update_product))
        // Order routes
        .route("/api/v1/orders", post(order_routes::create_order))
        .route("/api/v1/orders", get(order_routes::list_user_orders))
        .route("/api/v1/orders/{id}", get(order_routes::get_order))
}
