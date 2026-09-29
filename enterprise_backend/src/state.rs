use crate::{cache::CacheManager, config::AppConfig};
use axum::extract::FromRef;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cache: CacheManager,
    pub config: AppConfig,
}

impl AppState {
    pub fn new(db: PgPool, cache: CacheManager, config: AppConfig) -> Self {
        Self { db, cache, config }
    }
}

impl FromRef<AppState> for PgPool {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}

impl FromRef<AppState> for CacheManager {
    fn from_ref(input: &AppState) -> Self {
        input.cache.clone()
    }
}

impl FromRef<AppState> for AppConfig {
    fn from_ref(input: &AppState) -> Self {
        input.config.clone()
    }
}
