use crate::{
    domain::user::Role, error::AppError, extractors::auth::AuthenticatedUser, state::AppState,
};
use axum::{extract::FromRequestParts, http::request::Parts};

#[derive(Debug, Clone)]
pub struct RequireRole;

fn is_store_admin(role: &str) -> bool {
    role == Role::StoreAdmin.to_string()
}

impl<S> FromRequestParts<S> for RequireRole
where
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_user = AuthenticatedUser::from_request_parts(parts, state).await?;

        if !is_store_admin(&auth_user.role) {
            return Err(AppError::Forbidden);
        }

        Ok(RequireRole)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_write_role_requires_admin() {
        assert!(is_store_admin(&Role::StoreAdmin.to_string()));
        assert!(!is_store_admin(&Role::Customer.to_string()));
        assert!(!is_store_admin("unknown"));
    }
}
