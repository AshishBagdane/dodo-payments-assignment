mod application;
mod domain;
mod infrastructure;
mod presentation;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::application::services::{AccountService, AuthService, TransactionService};
use crate::application::AppState;
use crate::infrastructure::config::Config;
use crate::infrastructure::database::{
    self, PostgresAccountRepository, PostgresApiKeyRepository, PostgresTransactionRepository,
};
use crate::presentation::api::{
    account::{create_account, get_account, list_accounts},
    health::health_check,
    transaction::{deposit, get_history, transfer, withdraw},
};
use crate::presentation::middleware::auth::require_auth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_env()?;

    println!("Starting Dodo Payments Service...");
    println!("Server: {}", config.server_address());
    println!("Database: {}", config.database_url());

    // Create database connection pool
    let pool = database::create_pool(&config).await?;
    println!(
        "✓ Database connection pool created (max: {} connections)",
        config.database.max_connections
    );

    // Initialize Repositories
    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let transaction_repo = Arc::new(PostgresTransactionRepository::new(pool.clone()));
    let api_key_repo = Arc::new(PostgresApiKeyRepository::new(pool.clone()));

    // Initialize Services
    let account_service = Arc::new(AccountService::new(account_repo));
    let transaction_service = Arc::new(TransactionService::new(transaction_repo));
    let auth_service = Arc::new(AuthService::new(api_key_repo));

    // Create Application State
    let state = AppState {
        account_service,
        transaction_service,
        auth_service,
    };

    // Public Routes
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/accounts", axum::routing::post(create_account)); // Creation is public

    // Protected Routes
    let protected_routes = Router::new()
        .route("/accounts", get(list_accounts))
        .route("/accounts/{id}", get(get_account))
        .route("/transactions/deposit", axum::routing::post(deposit))
        .route("/transactions/withdraw", axum::routing::post(withdraw))
        .route("/transactions/transfer", axum::routing::post(transfer))
        .route("/transactions/history", get(get_history))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ));

    // Combine Routers
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state);

    // Start Server
    let addr: SocketAddr = config.server_address().parse()?;
    println!("✓ Service ready on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}