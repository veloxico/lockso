use anyhow::Result;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use std::path::Path;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub storage: lockso_db::storage::FileStorage,
    pub config: AppConfig,
    /// Server-side encryption key (AES-256-GCM, 32 bytes).
    pub encryption_key: Vec<u8>,
}

/// Runtime configuration loaded from environment.
#[derive(Clone)]
pub struct AppConfig {
    pub app_url: String,
    pub env: AppEnv,
    /// Trust X-Forwarded-For header (set to true only when behind a trusted reverse proxy).
    pub trust_proxy: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

impl AppEnv {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            _ => Self::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        *self == Self::Production
    }
}

impl AppState {
    pub async fn init() -> Result<Self> {
        let env = AppEnv::from_str(
            &std::env::var("LOCKSO_ENV").unwrap_or_else(|_| "development".to_string()),
        );
        let app_url =
            std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        // S3 config — require explicit credentials in production
        let s3_endpoint =
            std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:9000".to_string());
        let s3_bucket =
            std::env::var("S3_BUCKET").unwrap_or_else(|_| "lockso-files".to_string());
        let s3_region =
            std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        let s3_access_key = if env.is_production() {
            std::env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY must be set in production")
        } else {
            std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string())
        };
        let s3_secret_key = if env.is_production() {
            std::env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY must be set in production")
        } else {
            std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string())
        };

        // PostgreSQL with explicit pool limits
        let max_connections: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);

        let db = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(600))
            .connect(&database_url)
            .await?;

        // Run migrations at runtime (no compile-time DB required)
        let migrations_path = std::env::var("LOCKSO_MIGRATIONS_PATH")
            .unwrap_or_else(|_| "/migrations/schema".to_string());
        let migrator = Migrator::new(Path::new(&migrations_path)).await?;
        migrator.run(&db).await?;
        tracing::info!("Database connected and migrations applied");

        // Redis
        let redis_client = redis::Client::open(redis_url.as_str())?;
        let redis = redis::aio::ConnectionManager::new(redis_client).await?;
        tracing::info!("Redis connected");

        // S3 / MinIO
        let storage = lockso_db::storage::FileStorage::new(
            &s3_endpoint,
            &s3_region,
            &s3_access_key,
            &s3_secret_key,
            &s3_bucket,
        )
        .await?;
        tracing::info!("S3 storage initialized");

        let trust_proxy = std::env::var("LOCKSO_TRUST_PROXY")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        if trust_proxy {
            tracing::info!("Trusting X-Forwarded-For header from reverse proxy");
        }

        let config = AppConfig { app_url, env: env.clone(), trust_proxy };

        // Encryption key — required in production, generated for development
        let encryption_key = if env.is_production() {
            lockso_core::encryption::load_encryption_key()
                .expect("LOCKSO_ENCRYPTION_KEY must be set in production")
        } else {
            match lockso_core::encryption::load_encryption_key() {
                Ok(key) => key,
                Err(_) => {
                    tracing::warn!(
                        "LOCKSO_ENCRYPTION_KEY not set, using deterministic dev key. DO NOT use in production!"
                    );
                    // Deterministic dev key (sha256 of "lockso-dev-key")
                    hex::decode(
                        "a3c2f8e1d4b7c6a5e8f1d2c3b4a5d6e7f8a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5"
                    ).expect("hardcoded hex is valid")
                }
            }
        };
        lockso_crypto::aes_gcm::validate_key(&encryption_key)
            .expect("Encryption key validation failed");
        tracing::info!("Encryption key loaded and validated");

        Ok(Self { db, redis, storage, config, encryption_key })
    }
}
