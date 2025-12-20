use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Notify;

use dodo_payments_assignment::application::dto::DepositRequest;
use dodo_payments_assignment::application::services::{TransactionService, WebhookService};
use dodo_payments_assignment::domain::entities::{Account, Webhook};
use dodo_payments_assignment::domain::repositories::{AccountRepository, WebhookRepository};
use dodo_payments_assignment::domain::services::WebhookDispatcher;
use dodo_payments_assignment::domain::value_objects::{Money, WebhookEvent};
use dodo_payments_assignment::infrastructure::config::Config;
use dodo_payments_assignment::infrastructure::database::{
    create_pool, PostgresAccountRepository, PostgresTransactionRepository, PostgresWebhookRepository,
};
use rust_decimal_macros::dec;

// Mock Dispatcher to capture calls
#[derive(Clone)]
struct MockWebhookDispatcher {
    calls: Arc<Mutex<Vec<(String, Value, String)>>>,
    notify: Arc<Notify>,
}

impl MockWebhookDispatcher {
    fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            notify: Arc::new(Notify::new()),
        }
    }
}

#[async_trait]
impl WebhookDispatcher for MockWebhookDispatcher {
    async fn dispatch(&self, url: &str, payload: &Value, secret: &str) -> Result<(), String> {
        self.calls.lock().unwrap().push((url.to_string(), payload.clone(), secret.to_string()));
        self.notify.notify_one();
        Ok(())
    }
}

#[tokio::test]
async fn test_webhook_dispatch_on_deposit() {
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.unwrap();

    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let transaction_repo = Arc::new(PostgresTransactionRepository::new(pool.clone()));
    let webhook_repo = Arc::new(PostgresWebhookRepository::new(pool.clone()));

    let mock_dispatcher = Arc::new(MockWebhookDispatcher::new());
    let webhook_service = Arc::new(WebhookService::new(
        webhook_repo.clone(),
        account_repo.clone(),
        mock_dispatcher.clone(),
    ));

    let transaction_service = TransactionService::new(
        transaction_repo,
        Some(webhook_service),
    );

    // 1. Create Account
    let account = Account::new("Webhook Dispatch Test".to_string(), Money::new(dec!(0.0)).unwrap()).unwrap();
    account_repo.create(&account).await.unwrap();

    // 2. Register Webhook
    let webhook = Webhook::new(
        account.id,
        "https://example.com/callback".to_string(),
        WebhookEvent::TransactionCompleted,
    ).unwrap();
    webhook_repo.create(webhook).await.unwrap();

    // 3. Perform Deposit
    let deposit = DepositRequest {
        account_id: account.id,
        amount: dec!(100.0),
        idempotency_key: None,
    };
    
    let _ = transaction_service.deposit(deposit).await.unwrap();

    // 4. Wait for background task
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(2), mock_dispatcher.notify.notified());
    assert!(timeout.await.is_ok(), "Timed out waiting for webhook dispatch");

    // 5. Verify Dispatch
    let calls = mock_dispatcher.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "https://example.com/callback");
    
    // Verify secret was passed correctly
    assert_eq!(calls[0].2, account.webhook_secret);
    
    // Check payload
    let payload = &calls[0].1;
    // TransactionResponse has to_account_id, transaction_type, amount
    assert_eq!(payload["to_account_id"].as_str().unwrap(), account.id.to_string());
    if let Some(amt) = payload["amount"].as_f64() {
        assert_eq!(amt, 100.0);
    } else {
        assert_eq!(payload["amount"].as_str().unwrap(), "100.00");
    }
    
    // transaction_type instead of type
    assert_eq!(payload["transaction_type"].as_str().unwrap(), "credit");
}
