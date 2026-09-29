mod cache;
mod config;
mod domain;
mod error;
mod extractors;
mod middleware;
mod openapi;
mod repository;
mod routes;
mod services;
mod state;

use cache::CacheManager;
use config::AppConfig;
use openapi::ApiDoc;
use state::AppState;
use std::net::SocketAddr;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, SetRequestIdLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing & structured logging subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,enterprise_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting Enterprise Backend API Service with L1/L2 Caching...");

    // Load configuration
    let config = AppConfig::from_env().expect("Failed to load application configuration");
    tracing::info!("Configured server address: {}:{}", config.host, config.port);

    // Initialize PostgreSQL connection pool with SQLx
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .connect(&config.database_url)
        .await
        .map_err(|e| {
            tracing::error!("Failed to connect to PostgreSQL database: {:?}", e);
            e
        })?;

    tracing::info!("✅ Database connection pool established successfully");

    // Initialize Redis connection manager for L2 Caching
    let redis_conn = match redis::Client::open(config.redis_url.clone()) {
        Ok(client) => match client.get_connection_manager().await {
            Ok(mgr) => {
                tracing::info!(
                    "✅ L2 Redis cache connection established at {}",
                    config.redis_url
                );
                Some(mgr)
            }
            Err(e) => {
                tracing::warn!("⚠️ Could not connect to Redis L2 cache: {:?}. Fallback to L1 Moka in-memory cache.", e);
                None
            }
        },
        Err(e) => {
            tracing::warn!(
                "⚠️ Invalid Redis URL: {:?}. Fallback to L1 Moka in-memory cache.",
                e
            );
            None
        }
    };

    let cache = CacheManager::new(redis_conn);

    // Routes assume this schema; fail startup if migration fails.
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database schema migrations applied successfully");

    // Build shared application state
    let state = AppState::new(pool, cache, config.clone());

    // Allow the configured frontend origin. Browsers enforce this for web clients.
    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<axum::http::HeaderValue>()?)
        .allow_methods(Any)
        .allow_headers(Any);

    // Assemble Axum application router
    let app = routes::create_router()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(middleware::tracing::create_trace_layer())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(cors)
        .with_state(state);

    // Bind TcpListener and launch Axum web server
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid host and port address format");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("🌟 Server listening on http://{}", addr);
    tracing::info!("📖 Swagger UI live at http://{}/swagger-ui", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
