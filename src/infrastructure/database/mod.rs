use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

use crate::infrastructure::config::Config;

pub mod postgres_account_repository;
pub mod postgres_transaction_repository;
pub mod postgres_api_key_repository;

pub use postgres_account_repository::PostgresAccountRepository;
pub use postgres_transaction_repository::PostgresTransactionRepository;
pub use postgres_api_key_repository::PostgresApiKeyRepository;

/// Create a PostgreSQL connection pool
pub async fn create_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(Duration::from_secs(config.database.acquire_timeout_seconds))
        .connect(&config.database.url)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::config::{Config, DatabaseConfig, ServerConfig, RateLimitConfig, WebhookConfig, LoggingConfig};

    #[tokio::test]
    async fn test_create_pool_with_invalid_url() {
        let config = Config {
            database: DatabaseConfig {
                url: "postgresql://invalid:5432/nonexistent".to_string(),
                max_connections: 5,
                min_connections: 1,
                acquire_timeout_seconds: 5,
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            rate_limiting: RateLimitConfig {
                requests_per_hour: 1000,
            },
            webhook: WebhookConfig {
                timeout_seconds: 30,
                max_retries: 5,
                retry_backoff_seconds: 60,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
        };

        let result = create_pool(&config).await;
        assert!(result.is_err());
    }
}