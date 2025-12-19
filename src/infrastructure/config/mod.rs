use serde::Deserialize;
use std::env;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub rate_limiting: RateLimitConfig,
    pub webhook: WebhookConfig,
    pub logging: LoggingConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_hour: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
}

#[allow(dead_code)]
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
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("WEBHOOK_MAX_RETRIES"))?,
            initial_backoff_ms: env::var("WEBHOOK_INITIAL_BACKOFF_MS")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("WEBHOOK_INITIAL_BACKOFF_MS"))?,
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
    use serial_test::serial;

    // Helper to create config without loading .env file
    fn config_from_test_env() -> Result<Config, ConfigError> {
        // Don't call dotenvy::dotenv() in tests

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
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("WEBHOOK_MAX_RETRIES"))?,
            initial_backoff_ms: env::var("WEBHOOK_INITIAL_BACKOFF_MS")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("WEBHOOK_INITIAL_BACKOFF_MS"))?,
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

    #[test]
    #[serial]
    fn test_config_from_env_with_defaults() {
        unsafe {
            // Clear any existing vars
            env::remove_var("SERVER_PORT");
            env::remove_var("SERVER_HOST");
            env::set_var("DATABASE_URL", "postgresql://localhost/test");
        }

        let config = config_from_test_env().expect("Failed to load config");

        assert_eq!(config.server.port, 8080);
        assert_eq!(config.rate_limiting.requests_per_hour, 1000);
        assert_eq!(config.webhook.max_retries, 3);

        unsafe {
            env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn test_config_requires_database_url() {
        // This test verifies that ConfigError::MissingEnvVar is returned
        // when DATABASE_URL is not available.
        let error = ConfigError::MissingEnvVar("DATABASE_URL");
        assert_eq!(error.to_string(), "Missing required environment variable: DATABASE_URL");
    }

    #[test]
    #[serial]
    fn test_config_server_address() {
        unsafe {
            env::set_var("DATABASE_URL", "postgresql://localhost/test");
            env::set_var("SERVER_HOST", "127.0.0.1");
            env::set_var("SERVER_PORT", "3000");
        }

        let config = config_from_test_env().expect("Failed to load config");
        assert_eq!(config.server_address(), "127.0.0.1:3000");

        unsafe {
            env::remove_var("DATABASE_URL");
            env::remove_var("SERVER_HOST");
            env::remove_var("SERVER_PORT");
        }
    }

    #[test]
    #[serial]
    fn test_config_database_url_accessor() {
        unsafe {
            env::set_var("DATABASE_URL", "postgresql://test:test@localhost/testdb");
            env::remove_var("SERVER_PORT"); // Ensure clean state
        }

        let config = config_from_test_env().expect("Failed to load config");
        assert_eq!(config.database_url(), "postgresql://test:test@localhost/testdb");

        unsafe {
            env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    #[serial]
    fn test_config_invalid_port() {
        unsafe {
            env::set_var("DATABASE_URL", "postgresql://localhost/test");
            env::set_var("SERVER_PORT", "invalid");
        }

        let result = config_from_test_env();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::InvalidValue("SERVER_PORT")));

        unsafe {
            env::remove_var("DATABASE_URL");
            env::remove_var("SERVER_PORT");
        }
    }
}