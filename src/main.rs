mod application;
mod domain;
mod infrastructure;
mod presentation;

use axum::{routing::get, Router};
use utoipa::OpenApi;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::application::services::{AccountService, AuthService, TransactionService, WebhookService};
use crate::application::AppState;
use crate::infrastructure::config::Config;
use crate::infrastructure::database::{
    self, PostgresAccountRepository, PostgresApiKeyRepository, PostgresTransactionRepository,
    PostgresWebhookRepository,
};
use crate::infrastructure::http_client::ReqwestWebhookDispatcher;
// Imports cleaned up
// Actually, let's clean up unused imports too. `require_auth` is not used anymore.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    tracing::info!("Starting Dodo Payments Service...");
    tracing::info!("Server: {}", config.server_address());
    tracing::info!("Database: {}", config.database_url());

    // Create database connection pool
    let pool = database::create_pool(&config).await?;
    tracing::info!(
        "✓ Database connection pool created (max: {} connections)",
        config.database.max_connections
    );

    // Initialize Repositories
    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let transaction_repo = Arc::new(PostgresTransactionRepository::new(pool.clone()));
    let api_key_repo = Arc::new(PostgresApiKeyRepository::new(pool.clone()));
    let webhook_repo = Arc::new(PostgresWebhookRepository::new(pool.clone()));

    // Initialize Webhook Components
    let webhook_dispatcher = Arc::new(ReqwestWebhookDispatcher::new(
        config.webhook.max_retries,
        config.webhook.initial_backoff_ms,
    ));
    let webhook_service = Arc::new(WebhookService::new(
        webhook_repo.clone(),
        account_repo.clone(),
        webhook_dispatcher,
    ));

    // Initialize Services
    let account_service = Arc::new(AccountService::new(account_repo));
    let transaction_service = Arc::new(TransactionService::new(
        transaction_repo,
        Some(webhook_service),
    ));
    let auth_service = Arc::new(AuthService::new(api_key_repo));

    // Create Application State
    let app_state = AppState {
        account_service,
        transaction_service,
        auth_service,
        webhook_repository: webhook_repo,
    };

    // Initialize Rate Limit Layer
    let rate_limit_layer = crate::presentation::middleware::rate_limit::RateLimitLayer::new(
        config.rate_limiting.requests_per_hour,
    );

    // Build API Router
    let protected_routes = Router::new()
        .nest("/transactions", Router::new()
            .route("/deposit", axum::routing::post(presentation::api::transaction::deposit))
            .route("/withdraw", axum::routing::post(presentation::api::transaction::withdraw))
            .route("/transfer", axum::routing::post(presentation::api::transaction::transfer))
            .route("/history", get(presentation::api::transaction::get_history))
        )
        .route("/accounts/:id", get(presentation::api::account::get_account))
        .route("/accounts", get(presentation::api::account::list_accounts))
        .route("/webhooks", axum::routing::get(presentation::api::webhook::list_webhooks))
        .route("/webhooks/:id", axum::routing::delete(presentation::api::webhook::delete_webhook))
        .route("/webhooks", axum::routing::post(presentation::api::webhook::create_webhook))
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), crate::presentation::middleware::auth::require_auth));


    // Create OpenAPI Spec
    let openapi = crate::presentation::api::openapi::ApiDoc::openapi();

    let api_router: Router = Router::new()
        // Public Endpoints
        .route("/health", get(presentation::api::health::health_check))
        .route("/accounts", axum::routing::post(presentation::api::account::create_account))
        // Protected Endpoints
        .merge(protected_routes)
        // Apply Global Middleware
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn_with_state(rate_limit_layer, crate::presentation::middleware::rate_limit::RateLimitLayer::handle))
        .with_state(app_state);

    let app = Router::new()
        .merge(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .merge(api_router)
        .into_make_service_with_connect_info::<SocketAddr>(); // Important for rate limiting

    // Start Server
    let addr: SocketAddr = config.server_address().parse()?;
    tracing::info!("✓ Service ready on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}