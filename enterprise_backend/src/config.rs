use std::env;

// Avoid deriving Debug: it would print the JWT signing secret in logs.
#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub cors_origin: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let host = env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("APP_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|e| format!("Invalid APP_PORT: {}", e))?;

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable must be set".to_string())?;

        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET environment variable must be set".to_string())?;
        if jwt_secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 bytes".to_string());
        }

        let cors_origin =
            env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".to_string());

        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<i64>()
            .map_err(|e| format!("Invalid JWT_EXPIRATION_HOURS: {}", e))?;

        Ok(Self {
            host,
            port,
            database_url,
            redis_url,
            jwt_secret,
            jwt_expiration_hours,
            cors_origin,
        })
    }
}
