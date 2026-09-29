use crate::{
    domain::user::Claims, error::AppError, services::auth_service::AuthService, state::AppState,
};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: String,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized);
        }

        let token = &auth_header[7..];

        let claims: Claims = AuthService::verify_jwt(token, &app_state.config.jwt_secret)?;

        Ok(AuthenticatedUser {
            id: claims.sub,
            role: claims.role,
        })
    }
}
