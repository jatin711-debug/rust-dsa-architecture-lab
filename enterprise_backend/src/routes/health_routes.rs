use crate::{error::AppError, state::AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "System diagnostic health check & cache metrics")
    )
)]
pub async fn health_check(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let db_status = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let cache_metrics = state.cache.get_metrics();

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "database": db_status,
            "cache": {
                "l1_hits": cache_metrics.l1_hits,
                "l2_hits": cache_metrics.l2_hits,
                "misses": cache_metrics.misses
            },
            "timestamp": chrono::Utc::now()
        })),
    ))
}
