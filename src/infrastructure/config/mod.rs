// Configuration management will be defined here
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub rate_limiting: RateLimitConfig,
    pub webhook: WebhookConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_hour: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_backoff_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let database = DatabaseConfig {
            url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?,
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DATABASE_MAX_CONNECTIONS"))?,
            min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DATABASE_MIN_CONNECTIONS"))?,
            acquire_timeout_seconds: env::var("DATABASE_ACQUIRE_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DATABASE_ACQUIRE_TIMEOUT_SECONDS"))?,
        };

        let server = ServerConfig {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("SERVER_PORT"))?,
        };

        let rate_limiting = RateLimitConfig {
            requests_per_hour: env::var("RATE_LIMIT_PER_HOUR")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("RATE_LIMIT_PER_HOUR"))?,
        };

        let webhook = WebhookConfig {
            timeout_seconds: env::var("WEBHOOK_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("WEBHOOK_TIMEOUT_SECONDS"))?,
            max_retries: env::var("WEBHOOK_MAX_RETRIES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("WEBHOOK_MAX_RETRIES"))?,
            retry_backoff_seconds: env::var("WEBHOOK_RETRY_BACKOFF_SECONDS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("WEBHOOK_RETRY_BACKOFF_SECONDS"))?,
        };

        let logging = LoggingConfig {
            level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        };

        Ok(Config {
            database,
            server,
            rate_limiting,
            webhook,
            logging,
        })
    }

    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(&'static str),

    #[error("Invalid value for environment variable: {0}")]
    InvalidValue(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env_with_defaults() {
        env::set_var("DATABASE_URL", "postgresql://localhost/test");

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(config.server.port, 8080);
        assert_eq!(config.rate_limiting.requests_per_hour, 1000);
        assert_eq!(config.webhook.max_retries, 5);

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_requires_database_url() {
        env::remove_var("DATABASE_URL");

        let result = Config::from_env();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::MissingEnvVar("DATABASE_URL")));
    }
}