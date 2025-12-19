use dodo_payments_assignment::application::dto::DepositRequest;
use dodo_payments_assignment::application::services::TransactionService;
use dodo_payments_assignment::domain::entities::Account;
use dodo_payments_assignment::domain::repositories::AccountRepository;
use dodo_payments_assignment::domain::value_objects::Money;
use dodo_payments_assignment::infrastructure::config::Config;
use dodo_payments_assignment::infrastructure::database::{
    create_pool, PostgresAccountRepository, PostgresTransactionRepository,
};
use rust_decimal_macros::dec;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_deposit_idempotency() {
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/dodo-payments");
    }
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.unwrap();

    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let transaction_repo = Arc::new(PostgresTransactionRepository::new(pool.clone()));

    let service = TransactionService::new(transaction_repo, None);

    // 1. Create Account
    let account = Account::new("Idempotency Test User".to_string(), Money::new(dec!(0.0)).unwrap()).unwrap();
    account_repo.create(&account).await.unwrap();

    let idempotency_key = format!("idempotency-test-{}", Uuid::new_v4());

    // 2. First Deposit
    let request1 = DepositRequest {
        account_id: account.id,
        amount: dec!(100.0),
        idempotency_key: Some(idempotency_key.clone()),
    };

    let tx1 = service.deposit(request1).await.expect("First deposit failed");

    // Verify Balance
    let updated_account = account_repo.find_by_id(account.id).await.unwrap();
    assert_eq!(updated_account.balance.amount(), dec!(100.0));

    // 3. Duplicate Deposit
    let request2 = DepositRequest {
        account_id: account.id,
        amount: dec!(100.0),
        idempotency_key: Some(idempotency_key),
    };

    let tx2 = service.deposit(request2).await.expect("Second deposit failed (should be idempotent)");

    // 4. Assertions
    assert_eq!(tx1.id, tx2.id, "Transaction IDs should match");
    assert_eq!(tx1.amount, tx2.amount);
    
    // Verify Balance (Should NOT double charge)
    let final_account = account_repo.find_by_id(account.id).await.unwrap();
    assert_eq!(final_account.balance.amount(), dec!(100.0), "Balance should not increase on duplicate request");
}
