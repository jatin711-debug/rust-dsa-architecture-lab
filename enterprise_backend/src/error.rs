use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Invalid or expired token")]
    Unauthorized,

    #[error("Forbidden access: insufficient permissions")]
    Forbidden,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    BadRequest(String),

    #[error("Conflict error: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    status: &'static str,
    error_code: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::Database(err) => {
                tracing::error!("Database query failed: {:?}", err);
                if let sqlx::Error::Database(db_err) = err {
                    if db_err.is_unique_violation() {
                        (
                            StatusCode::CONFLICT,
                            "UNIQUE_VIOLATION".to_string(),
                            "Resource with specified identifier already exists".to_string(),
                        )
                    } else {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "DATABASE_ERROR".to_string(),
                            "An unexpected database error occurred".to_string(),
                        )
                    }
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "DATABASE_ERROR".to_string(),
                        "An unexpected database error occurred".to_string(),
                    )
                }
            }
            AppError::Auth(msg) => (
                StatusCode::UNAUTHORIZED,
                "AUTH_FAILED".to_string(),
                msg.clone(),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED".to_string(),
                "Access token is missing, invalid, or expired".to_string(),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN".to_string(),
                "You do not have permission to access this resource".to_string(),
            ),
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND".to_string(), msg.clone())
            }
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST".to_string(),
                msg.clone(),
            ),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT".to_string(), msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR".to_string(),
                    "An internal server error occurred".to_string(),
                )
            }
        };

        let body = Json(ErrorResponse {
            status: "error",
            error_code,
            message,
        });

        (status, body).into_response()
    }
}
