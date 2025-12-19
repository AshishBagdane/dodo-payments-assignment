use dodo_payments_assignment::infrastructure::{config::{Config, DatabaseConfig, ServerConfig, RateLimitConfig, WebhookConfig, LoggingConfig}, database};

/// Helper to create test config with localhost database
fn create_test_config() -> Config {
    Config {
        database: DatabaseConfig {
            url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/dodo-payments".to_string()),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout_seconds: 30,
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
            initial_backoff_ms: 1000,
        },
        logging: LoggingConfig {
            level: "info".to_string(),
        },
    }
}

#[tokio::test]
async fn test_database_connection_pool() {
    // Use test config with localhost
    let config = create_test_config();

    // Create pool
    let pool = database::create_pool(&config)
        .await
        .expect("Failed to create database pool");

    // Test query
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to execute test query");

    assert_eq!(result.0, 1);
}

#[tokio::test]
async fn test_pool_can_handle_multiple_connections() {
    let config = create_test_config();
    let pool = database::create_pool(&config)
        .await
        .expect("Failed to create database pool");

    // Execute multiple queries concurrently
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let result: (i32,) = sqlx::query_as(&format!("SELECT {}", i))
                    .fetch_one(&pool)
                    .await
                    .expect("Query failed");
                result.0
            })
        })
        .collect();

    // Wait for all to complete
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.expect("Task failed");
        assert_eq!(result, i as i32);
    }
}