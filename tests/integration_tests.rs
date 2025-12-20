use dodo_payments_assignment::application::dto::account_dto::CreateAccountRequest;
use dodo_payments_assignment::application::dto::transaction_dto::{
    DepositRequest, TransferRequest, WithdrawRequest,
};
use dodo_payments_assignment::application::services::account_service::AccountService;
use dodo_payments_assignment::application::services::transaction_service::TransactionService;
use dodo_payments_assignment::application::services::webhook_service::WebhookService;
use dodo_payments_assignment::domain::services::WebhookDispatcher;
use dodo_payments_assignment::infrastructure::config::Config;
use dodo_payments_assignment::infrastructure::database::postgres_account_repository::PostgresAccountRepository;
use dodo_payments_assignment::infrastructure::database::postgres_api_key_repository::PostgresApiKeyRepository;
use dodo_payments_assignment::infrastructure::database::postgres_transaction_repository::PostgresTransactionRepository;
use dodo_payments_assignment::infrastructure::database::postgres_webhook_repository::PostgresWebhookRepository;
use rust_decimal_macros::dec;
use serde_json::Value; // Import Value
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::Mutex; // Import Mutex
use uuid::Uuid;

// Mock Webhook Dispatcher to avoid needing a real HTTP server
struct MockWebhookDispatcher {
    dispatched: Arc<Mutex<Vec<(String, Value)>>>,
}

#[async_trait::async_trait]
impl WebhookDispatcher for MockWebhookDispatcher {
    async fn dispatch(
        &self,
        url: &str,
        payload: &Value,
        _secret: &str,
    ) -> Result<(), String> {
        let mut dispatched = self.dispatched.lock().await;
        dispatched.push((url.to_string(), payload.clone()));
        Ok(())
    }
}

async fn setup_test_context() -> (
    Arc<AccountService>,
    Arc<TransactionService>,
    Arc<Mutex<Vec<(String, Value)>>>,
) {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("Failed to load config");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url)
        .await
        .expect("Failed to connect to DB");

    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let transaction_repo = Arc::new(PostgresTransactionRepository::new(pool.clone()));
    let webhook_repo = Arc::new(PostgresWebhookRepository::new(pool.clone()));
    // We don't strictly need ApiKeyRepo for service tests unless we test Auth service
    let _api_key_repo = Arc::new(PostgresApiKeyRepository::new(pool.clone()));

    let dispatched = Arc::new(Mutex::new(Vec::new()));
    let dispatcher = Arc::new(MockWebhookDispatcher {
        dispatched: dispatched.clone(),
    });

    let webhook_service = Arc::new(WebhookService::new(
        webhook_repo.clone(),
        account_repo.clone(),
        dispatcher,
    ));

    let account_service = Arc::new(AccountService::new(account_repo.clone()));
    let transaction_service = Arc::new(TransactionService::new(
        transaction_repo.clone(),
        Some(webhook_service.clone()),
    ));

    (account_service, transaction_service, dispatched)
}

#[tokio::test]
async fn test_full_lifecycle() {
    let (account_service, transaction_service, _) = setup_test_context().await;

    // 1. Create Account A
    let account_a = account_service
        .create_account(CreateAccountRequest {
            business_name: "Alice Corp".to_string(),
        })
        .await
        .expect("Failed to create account A");

    // 2. Create Account B
    let account_b = account_service
        .create_account(CreateAccountRequest {
            business_name: "Bob Inc".to_string(),
        })
        .await
        .expect("Failed to create account B");

    // 3. Deposit 100 to A
    let deposit = transaction_service
        .deposit(DepositRequest {
            account_id: account_a.id,
            amount: dec!(100.00),
            idempotency_key: Some(Uuid::new_v4().to_string()),
        })
        .await
        .expect("Failed to deposit");
    assert_eq!(deposit.amount, dec!(100.00));

    // Verify A balance
    let account_a = account_service
        .get_account(account_a.id)
        .await
        .expect("Failed to fetch A");
    assert_eq!(account_a.balance, dec!(100.00));

    // 4. Transfer 50 from A to B
    transaction_service
        .transfer(TransferRequest {
            from_account_id: account_a.id,
            to_account_id: account_b.id,
            amount: dec!(50.00),
            idempotency_key: Some(Uuid::new_v4().to_string()),
        })
        .await
        .expect("Failed to transfer");

    // 5. Withdraw 10 from B
    transaction_service
        .withdraw(WithdrawRequest {
            account_id: account_b.id,
            amount: dec!(10.00),
            idempotency_key: Some(Uuid::new_v4().to_string()),
        })
        .await
        .expect("Failed to withdraw");

    // 6. Verify Final Balances
    let account_a_final = account_service
        .get_account(account_a.id)
        .await
        .unwrap();
    let account_b_final = account_service
        .get_account(account_b.id)
        .await
        .unwrap();

    // A: 100 - 50 = 50
    assert_eq!(account_a_final.balance, dec!(50.00));
    // B: 0 + 50 - 10 = 40
    assert_eq!(account_b_final.balance, dec!(40.00));
}

#[tokio::test]
async fn test_concurrency_invariant() {
    let (account_service, transaction_service, _) = setup_test_context().await;

    // 1. Create two accounts
    let acc1 = account_service
        .create_account(CreateAccountRequest {
            business_name: "Conc1".to_string(),
        })
        .await
        .unwrap();
    let acc2 = account_service
        .create_account(CreateAccountRequest {
            business_name: "Conc2".to_string(),
        })
        .await
        .unwrap();

    // 2. Deposit 1000 into Acc1
    transaction_service
        .deposit(DepositRequest {
            account_id: acc1.id,
            amount: dec!(1000.00),
            idempotency_key: Some(Uuid::new_v4().to_string()),
        })
        .await
        .unwrap();

    // Initial system balance = 1000
    let initial_sum = dec!(1000.00);

    // 3. Spawn 10 concurrent transfers of 10.00 from Acc1 -> Acc2
    let mut handles = vec![];
    for _ in 0..10 {
        let ts = transaction_service.clone();
        let from = acc1.id;
        let to = acc2.id;
        handles.push(tokio::spawn(async move {
            ts.transfer(TransferRequest {
                from_account_id: from,
                to_account_id: to,
                amount: dec!(10.00),
                idempotency_key: Some(Uuid::new_v4().to_string()),
            })
            .await
        }));
    }

    // Wait for all
    for h in handles {
        let _ = h.await; // Some might fail if we run out of funds or deadlock (shouldn't deadlock)
                         // For this test, we expect all to succeed as we have 1000 and transfer 10*10=100
    }

    // 4. Verify Final Balances
    let acc1_final = account_service.get_account(acc1.id).await.unwrap();
    let acc2_final = account_service.get_account(acc2.id).await.unwrap();

    // Check Invariant: Sum should still be 1000 (money is conserved)
    let final_sum = acc1_final.balance + acc2_final.balance;
    assert_eq!(
        final_sum, initial_sum,
        "Money must be conserved in transfers"
    );

    // Check specific values
    // Acc1: 1000 - 10*10 = 900
    // Acc2: 0 + 10*10 = 100
    assert_eq!(acc1_final.balance, dec!(900.00));
    assert_eq!(acc2_final.balance, dec!(100.00));
}
