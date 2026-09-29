use moka::future::Cache as MokaCache;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CacheSource {
    L1Hit,
    L2Hit,
    Miss,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheMetrics {
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub misses: u64,
}

#[derive(Clone)]
pub struct CacheManager {
    l1: MokaCache<String, String>,
    l2: Option<ConnectionManager>,
    l1_hits: Arc<AtomicU64>,
    l2_hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

impl CacheManager {
    pub fn new(redis_conn: Option<ConnectionManager>) -> Self {
        let l1 = MokaCache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300)) // 5 mins default TTL
            .build();

        Self {
            l1,
            l2: redis_conn,
            l1_hits: Arc::new(AtomicU64::new(0)),
            l2_hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> (Option<T>, CacheSource) {
        // 1. Check L1 Cache (Moka)
        if let Some(val_str) = self.l1.get(key).await {
            if let Ok(val) = serde_json::from_str::<T>(&val_str) {
                self.l1_hits.fetch_add(1, Ordering::Relaxed);
                tracing::debug!("⚡ L1 Cache HIT for key: {}", key);
                return (Some(val), CacheSource::L1Hit);
            }
        }

        // 2. Check L2 Cache (Redis)
        if let Some(mut conn) = self.l2.clone() {
            let redis_res: Result<Option<String>, redis::RedisError> = conn.get(key).await;
            if let Ok(Some(val_str)) = redis_res {
                if let Ok(val) = serde_json::from_str::<T>(&val_str) {
                    self.l2_hits.fetch_add(1, Ordering::Relaxed);
                    tracing::debug!("🚀 L2 Cache HIT (Redis) for key: {}", key);
                    // Populate L1 cache
                    self.l1.insert(key.to_string(), val_str).await;
                    return (Some(val), CacheSource::L2Hit);
                }
            }
        }

        // 3. Cache Miss
        self.misses.fetch_add(1, Ordering::Relaxed);
        tracing::debug!("🔍 Cache MISS for key: {}", key);
        (None, CacheSource::Miss)
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_secs: u64) {
        if let Ok(val_str) = serde_json::to_string(value) {
            // Write to L1 Cache
            self.l1.insert(key.to_string(), val_str.clone()).await;

            // Write to L2 Cache (Redis)
            if let Some(mut conn) = self.l2.clone() {
                let _: Result<(), redis::RedisError> = conn.set_ex(key, val_str, ttl_secs).await;
            }
        }
    }

    pub async fn invalidate(&self, key: &str) {
        self.l1.invalidate(key).await;
        if let Some(mut conn) = self.l2.clone() {
            let _: Result<(), redis::RedisError> = conn.del(key).await;
        }
        tracing::info!("🧹 Invalidated cache key: {}", key);
    }

    pub fn get_metrics(&self) -> CacheMetrics {
        CacheMetrics {
            l1_hits: self.l1_hits.load(Ordering::Relaxed),
            l2_hits: self.l2_hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
        }
    }
}
