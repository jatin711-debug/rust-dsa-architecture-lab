use crate::{
    domain::order::{CreateOrderDto, OrderResponse},
    error::AppError,
    extractors::auth::AuthenticatedUser,
    repository::order_repo::OrderRepository,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/v1/orders",
    request_body = CreateOrderDto,
    responses(
        (status = 201, description = "Order created successfully", body = OrderResponse),
        (status = 400, description = "Insufficient stock or invalid request"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_order(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(dto): Json<CreateOrderDto>,
) -> Result<impl IntoResponse, AppError> {
    let order_response =
        OrderRepository::create_order_transaction(&state.db, auth_user.id, dto).await?;

    // Invalidate product catalog list cache since stock was decremented
    state.cache.invalidate("products_list_all").await;

    Ok((StatusCode::CREATED, Json(order_response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/orders",
    responses(
        (status = 200, description = "List authenticated user's orders", body = Vec<OrderResponse>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_user_orders(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let orders = OrderRepository::list_by_user(&state.db, auth_user.id).await?;
    Ok((StatusCode::OK, Json(orders)))
}

#[utoipa::path(
    get,
    path = "/api/v1/orders/{id}",
    responses(
        (status = 200, description = "Get order details by ID", body = OrderResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Order not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_order(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let order = OrderRepository::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Order with id {} not found", id)))?;

    // Authorization check: ensure user owns the order or is an admin
    if order.user_id != auth_user.id && auth_user.role != "StoreAdmin" {
        return Err(AppError::Forbidden);
    }

    Ok((StatusCode::OK, Json(order)))
}
