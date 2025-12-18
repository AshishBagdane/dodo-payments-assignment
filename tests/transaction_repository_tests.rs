use dodo_payments_assignment::domain::entities::{Account, Transaction};
use dodo_payments_assignment::domain::repositories::{AccountRepository, TransactionRepository};
use dodo_payments_assignment::domain::value_objects::{Money, TransactionType};
use dodo_payments_assignment::infrastructure::config::Config;
use dodo_payments_assignment::infrastructure::database::{create_pool, PostgresAccountRepository, PostgresTransactionRepository};
use rust_decimal_macros::dec;
use uuid::Uuid;

#[tokio::test]
async fn test_create_transaction() {
    // Setup
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/dodo-payments");
    }
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.expect("Failed to create pool");
    let account_repo = PostgresAccountRepository::new(pool.clone());
    let transaction_repo = PostgresTransactionRepository::new(pool);

    // Create Account for tests
    let balance = Money::new(dec!(100.00)).unwrap();
    let account = Account::new("Test Corp Tx".to_string(), balance).unwrap();
    let created_account = account_repo.create(&account).await.expect("Failed to create account");

    // Create Credit Transaction
    let tx_amount = Money::new(dec!(50.00)).unwrap();
    let tx = Transaction::new_credit(
        created_account.id,
        tx_amount.clone(),
        Some(Uuid::new_v4().to_string()),
    ).unwrap();

    let created_tx = transaction_repo.create(&tx).await.expect("Failed to create transaction");

    // Assertions
    assert_eq!(created_tx.id, tx.id);
    assert_eq!(created_tx.amount, tx_amount);
    assert_eq!(created_tx.transaction_type, TransactionType::Credit);

    // Fetch and Verify
    let fetched_tx = transaction_repo.find_by_id(created_tx.id).await.expect("Failed to fetch tx");
    assert_eq!(fetched_tx.id, created_tx.id);

    // Test Idempotency
    let duplicate_res = transaction_repo.create(&tx).await;
    assert!(duplicate_res.is_err()); // Should verify duplicate entry error if implemented or constraints catch it
    
    // Verify Idempotency Check
    let exists = transaction_repo.idempotency_key_exists(tx.idempotency_key.as_ref().unwrap()).await.unwrap();
    assert!(exists);
}

#[tokio::test]
async fn test_execute_credit_atomic() {
    // Setup
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/dodo-payments");
    }
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.expect("Failed to create pool");
    let account_repo = PostgresAccountRepository::new(pool.clone());
    let transaction_repo = PostgresTransactionRepository::new(pool);

    // Create Account
    let initial_balance = Money::new(dec!(100.00)).unwrap();
    let account = Account::new("Credit Receiver".to_string(), initial_balance).unwrap();
    let created_account = account_repo.create(&account).await.unwrap();

    // Execute Credit
    let credit_amount = Money::new(dec!(50.00)).unwrap();
    let tx = Transaction::new_credit(
        created_account.id,
        credit_amount.clone(),
        Some(Uuid::new_v4().to_string())
    ).unwrap();

    let executed_tx = transaction_repo.execute_credit(&tx).await.expect("Failed to execute credit");

    assert_eq!(executed_tx.amount, credit_amount);

    // Verify Account Balance Updated
    let updated_account = account_repo.find_by_id(created_account.id).await.unwrap();
    assert_eq!(updated_account.balance.amount(), dec!(150.00));
}

#[tokio::test]
async fn test_execute_debit_atomic() {
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/dodo-payments");
    }
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.expect("Failed to create pool");
    let account_repo = PostgresAccountRepository::new(pool.clone());
    let transaction_repo = PostgresTransactionRepository::new(pool);

    let initial_balance = Money::new(dec!(100.00)).unwrap();
    let account = Account::new("Debit Payer".to_string(), initial_balance).unwrap();
    let created_account = account_repo.create(&account).await.unwrap();

    let debit_amount = Money::new(dec!(30.00)).unwrap();
    let tx = Transaction::new_debit(
        created_account.id,
        debit_amount,
        Some(Uuid::new_v4().to_string())
    ).unwrap();

    transaction_repo.execute_debit(&tx).await.expect("Failed to execute debit");

    let updated_account = account_repo.find_by_id(created_account.id).await.unwrap();
    assert_eq!(updated_account.balance.amount(), dec!(70.00));
}

#[tokio::test]
async fn test_execute_debit_insufficient_funds() {
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/dodo-payments");
    }
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.expect("Failed to create pool");
    let account_repo = PostgresAccountRepository::new(pool.clone());
    let transaction_repo = PostgresTransactionRepository::new(pool);

    let initial_balance = Money::new(dec!(10.00)).unwrap();
    let account = Account::new("Poor Payer".to_string(), initial_balance).unwrap();
    let created_account = account_repo.create(&account).await.unwrap();

    let debit_amount = Money::new(dec!(20.00)).unwrap(); // More than balance
    let tx = Transaction::new_debit(
        created_account.id,
        debit_amount,
        Some(Uuid::new_v4().to_string())
    ).unwrap();

    let result = transaction_repo.execute_debit(&tx).await;
    assert!(result.is_err());

    // Verify balance unchanged
    let updated_account = account_repo.find_by_id(created_account.id).await.unwrap();
    assert_eq!(updated_account.balance.amount(), dec!(10.00));
}

#[tokio::test]
async fn test_execute_transfer_atomic() {
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/dodo-payments");
    }
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.expect("Failed to create pool");
    let account_repo = PostgresAccountRepository::new(pool.clone());
    let transaction_repo = PostgresTransactionRepository::new(pool);

    let sender = Account::new("Sender".to_string(), Money::new(dec!(100.00)).unwrap()).unwrap();
    let receiver = Account::new("Receiver".to_string(), Money::new(dec!(50.00)).unwrap()).unwrap();

    let created_sender = account_repo.create(&sender).await.unwrap();
    let created_receiver = account_repo.create(&receiver).await.unwrap();

    let transfer_amount = Money::new(dec!(25.00)).unwrap();
    let tx = Transaction::new_transfer(
        created_sender.id,
        created_receiver.id,
        transfer_amount,
        Some(Uuid::new_v4().to_string())
    ).unwrap();

    transaction_repo.execute_transfer(&tx).await.expect("Failed to execute transfer");

    let updated_sender = account_repo.find_by_id(created_sender.id).await.unwrap();
    let updated_receiver = account_repo.find_by_id(created_receiver.id).await.unwrap();

    assert_eq!(updated_sender.balance.amount(), dec!(75.00));
    assert_eq!(updated_receiver.balance.amount(), dec!(75.00));
}
