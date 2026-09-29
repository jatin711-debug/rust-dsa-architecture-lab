use crate::{
    cache::CacheSource,
    domain::product::{CreateProductDto, Product, UpdateProductDto},
    error::AppError,
    extractors::roles::RequireRole,
    repository::product_repo::ProductRepository,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

const PRODUCTS_LIST_KEY: &str = "products_list_all";

#[utoipa::path(
    get,
    path = "/api/v1/products",
    responses(
        (status = 200, description = "List all products with L1/L2 caching", body = Vec<Product>)
    )
)]
pub async fn list_products(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let (cached_products, source): (Option<Vec<Product>>, CacheSource) =
        state.cache.get(PRODUCTS_LIST_KEY).await;

    let (products, cache_header_val) = match (cached_products, source) {
        (Some(prods), CacheSource::L1Hit) => (prods, "L1_HIT"),
        (Some(prods), CacheSource::L2Hit) => (prods, "L2_HIT"),
        _ => {
            let prods = ProductRepository::list_all(&state.db).await?;
            state.cache.set(PRODUCTS_LIST_KEY, &prods, 300).await;
            (prods, "DB_QUERY")
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert("X-Cache-Status", HeaderValue::from_static(cache_header_val));

    Ok((StatusCode::OK, headers, Json(products)))
}

#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    responses(
        (status = 200, description = "Get product by ID", body = Product),
        (status = 404, description = "Product not found")
    )
)]
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = format!("product_{}", id);
    let (cached_product, source): (Option<Product>, CacheSource) =
        state.cache.get(&cache_key).await;

    let (product, cache_header_val) = match (cached_product, source) {
        (Some(prod), CacheSource::L1Hit) => (prod, "L1_HIT"),
        (Some(prod), CacheSource::L2Hit) => (prod, "L2_HIT"),
        _ => {
            let prod = ProductRepository::find_by_id(&state.db, id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Product with id {} not found", id)))?;
            state.cache.set(&cache_key, &prod, 300).await;
            (prod, "DB_QUERY")
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert("X-Cache-Status", HeaderValue::from_static(cache_header_val));

    Ok((StatusCode::OK, headers, Json(product)))
}

#[utoipa::path(
    post,
    path = "/api/v1/products",
    request_body = CreateProductDto,
    responses(
        (status = 201, description = "Product created successfully", body = Product),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin required")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_product(
    State(state): State<AppState>,
    _role: RequireRole,
    Json(dto): Json<CreateProductDto>,
) -> Result<impl IntoResponse, AppError> {
    if dto.name.trim().is_empty() {
        return Err(AppError::BadRequest("Product name is required".to_string()));
    }
    if dto.price_cents < 0 {
        return Err(AppError::BadRequest(
            "Price must be non-negative".to_string(),
        ));
    }
    if dto.stock_quantity < 0 {
        return Err(AppError::BadRequest(
            "Stock quantity must be non-negative".to_string(),
        ));
    }

    let product = ProductRepository::create(&state.db, dto).await?;

    // Invalidate product catalog list cache
    state.cache.invalidate(PRODUCTS_LIST_KEY).await;

    Ok((StatusCode::CREATED, Json(product)))
}

#[utoipa::path(
    put,
    path = "/api/v1/products/{id}",
    request_body = UpdateProductDto,
    responses(
        (status = 200, description = "Product updated successfully", body = Product),
        (status = 404, description = "Product not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_product(
    State(state): State<AppState>,
    _role: RequireRole,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateProductDto>,
) -> Result<impl IntoResponse, AppError> {
    let product = ProductRepository::update(&state.db, id, dto).await?;

    // Invalidate caches
    state.cache.invalidate(PRODUCTS_LIST_KEY).await;
    state.cache.invalidate(&format!("product_{}", id)).await;

    Ok((StatusCode::OK, Json(product)))
}
