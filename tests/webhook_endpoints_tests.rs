use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt; 
use sha2::Digest; 

use dodo_payments_assignment::application::dto::WebhookResponse;
use dodo_payments_assignment::domain::entities::ApiKey;
use dodo_payments_assignment::domain::value_objects::{Money, WebhookEvent};
use rust_decimal_macros::dec; 
use dodo_payments_assignment::infrastructure::config::Config;
use dodo_payments_assignment::infrastructure::database::{
    create_pool, PostgresAccountRepository, PostgresApiKeyRepository, PostgresTransactionRepository,
    PostgresWebhookRepository,
};
use dodo_payments_assignment::domain::repositories::{AccountRepository, ApiKeyRepository};
use dodo_payments_assignment::application::services::{AccountService, AuthService, TransactionService};
use dodo_payments_assignment::application::AppState;
use dodo_payments_assignment::presentation::api::{
    webhook::{create_webhook, delete_webhook, list_webhooks},
};
use dodo_payments_assignment::presentation::middleware::auth::require_auth;
use axum::Router;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_app_and_key() -> (Router, String, Uuid) {
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.unwrap();

    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let transaction_repo = Arc::new(PostgresTransactionRepository::new(pool.clone()));
    let api_key_repo = Arc::new(PostgresApiKeyRepository::new(pool.clone()));
    let webhook_repo = Arc::new(PostgresWebhookRepository::new(pool.clone()));

    let account_service = Arc::new(AccountService::new(account_repo.clone()));
    let transaction_service = Arc::new(TransactionService::new(transaction_repo, None));
    let auth_service = Arc::new(AuthService::new(api_key_repo.clone()));

    let state = AppState {
        account_service,
        transaction_service,
        auth_service,
        webhook_repository: webhook_repo,
    };

    let account = dodo_payments_assignment::domain::entities::Account::new(
        "Webhook Test User".to_string(), 
        Money::new(dec!(0.0)).unwrap()
    ).unwrap();
    account_repo.create(&account.clone()).await.unwrap();
    
    let raw_key = format!("test_key_{}", Uuid::new_v4());
    let hashed_key = hex::encode(sha2::Sha256::digest(raw_key.as_bytes()));
    let api_key = ApiKey::new(account.id, hashed_key);
    
    api_key_repo.create(&api_key).await.unwrap();

    let protected_routes = Router::new()
        .route("/webhooks", axum::routing::post(create_webhook).get(list_webhooks))
        .route("/webhooks/{id}", axum::routing::delete(delete_webhook))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ));

    let app = Router::new()
        .merge(protected_routes)
        .with_state(state);
        
    (app, raw_key, account.id)
}

#[tokio::test]
async fn test_create_webhook() {
    let (app, api_key, _account_id) = setup_app_and_key().await;

    let payload = json!({
        "url": "https://example.com/webhook",
        "event": "transaction.completed"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/webhooks")
                .method("POST")
                .header("content-type", "application/json")
                .header("x-api-key", api_key)
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_dto: WebhookResponse = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(response_dto.url, "https://example.com/webhook");
    assert_eq!(response_dto.event, WebhookEvent::TransactionCompleted);
}

#[tokio::test]
async fn test_list_webhooks() {
    let (app, api_key, _account_id) = setup_app_and_key().await;

    // Create one first
    let payload = json!({
        "url": "https://example.com/webhook1",
        "event": "account.created"
    });

    app.clone()
        .oneshot(
             Request::builder()
                .uri("/webhooks")
                .method("POST")
                .header("content-type", "application/json")
                .header("x-api-key", api_key.clone())
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // List
    let response = app
        .oneshot(
            Request::builder()
                .uri("/webhooks")
                .method("GET")
                .header("x-api-key", api_key)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let webhooks: Vec<WebhookResponse> = serde_json::from_slice(&body_bytes).unwrap();
    
    assert!(!webhooks.is_empty());
    assert_eq!(webhooks[0].url, "https://example.com/webhook1");
}

#[tokio::test]
async fn test_delete_webhook() {
    let (app, api_key, _account_id) = setup_app_and_key().await;

    // Create
    let payload = json!({
        "url": "https://example.com/webhook-delete",
        "event": "transaction.completed"
    });

    let create_response = app.clone()
        .oneshot(
             Request::builder()
                .uri("/webhooks")
                .method("POST")
                .header("content-type", "application/json")
                .header("x-api-key", api_key.clone())
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
        
    let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let created: WebhookResponse = serde_json::from_slice(&body_bytes).unwrap();
    
    // Delete
     let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/webhooks/{}", created.id))
                .method("DELETE")
                .header("x-api-key", api_key)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
