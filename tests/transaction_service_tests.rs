use dodo_payments_assignment::application::dto::{DepositRequest, TransferRequest, WithdrawRequest};
use dodo_payments_assignment::application::services::TransactionService;
use dodo_payments_assignment::domain::entities::Transaction;
use dodo_payments_assignment::domain::errors::RepositoryError;
use dodo_payments_assignment::domain::repositories::TransactionRepository;
use dodo_payments_assignment::domain::value_objects::TransactionType;
use async_trait::async_trait;
use rust_decimal_macros::dec;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// Mock Repository
struct MockTransactionRepository {
    transactions: Mutex<Vec<Transaction>>,
}

impl MockTransactionRepository {
    fn new() -> Self {
        Self {
            transactions: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl TransactionRepository for MockTransactionRepository {
    async fn create(&self, transaction: &Transaction) -> Result<Transaction, RepositoryError> {
        self.transactions.lock().unwrap().push(transaction.clone());
        Ok(transaction.clone())
    }

    async fn find_by_id(&self, _id: Uuid) -> Result<Transaction, RepositoryError> {
        unimplemented!()
    }
    
    async fn find_by_idempotency_key(&self, _key: &str) -> Result<Transaction, RepositoryError> {
        unimplemented!()
    }
    
    async fn idempotency_key_exists(&self, _key: &str) -> Result<bool, RepositoryError> {
        unimplemented!()
    }

    async fn list_by_account(
        &self,
        account_id: Uuid,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<Transaction>, RepositoryError> {
        let transactions = self.transactions.lock().unwrap();
        Ok(transactions
            .iter()
            .filter(|t| t.from_account_id == Some(account_id) || t.to_account_id == Some(account_id))
            .cloned()
            .collect())
    }

    async fn list_by_type(
        &self,
        _transaction_type: TransactionType,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<Transaction>, RepositoryError> {
        unimplemented!()
    }

    async fn list(&self, _limit: i64, _offset: i64) -> Result<Vec<Transaction>, RepositoryError> {
        unimplemented!()
    }

    async fn execute_credit(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, RepositoryError> {
        self.transactions.lock().unwrap().push(transaction.clone());
        Ok(transaction.clone())
    }

    async fn execute_debit(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, RepositoryError> {
        self.transactions.lock().unwrap().push(transaction.clone());
        Ok(transaction.clone())
    }

    async fn execute_transfer(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, RepositoryError> {
        self.transactions.lock().unwrap().push(transaction.clone());
        Ok(transaction.clone())
    }
}

#[tokio::test]
async fn test_deposit() {
    let mock_repo = Arc::new(MockTransactionRepository::new());
    let service = TransactionService::new(mock_repo, None);
    let account_id = Uuid::new_v4();
    let amount = dec!(100.00);

    let request = DepositRequest { account_id, amount, idempotency_key: None };
    let response = service.deposit(request).await.expect("Deposit failed");

    assert_eq!(response.to_account_id, Some(account_id));
    assert_eq!(response.amount, amount);
    assert_eq!(response.transaction_type, "credit");
}

#[tokio::test]
async fn test_withdraw() {
    let mock_repo = Arc::new(MockTransactionRepository::new());
    let service = TransactionService::new(mock_repo, None);
    let account_id = Uuid::new_v4();
    let amount = dec!(50.00);

    let request = WithdrawRequest { account_id, amount, idempotency_key: None };
    let response = service.withdraw(request).await.expect("Withdraw failed");

    assert_eq!(response.from_account_id, Some(account_id));
    assert_eq!(response.amount, amount);
    assert_eq!(response.transaction_type, "debit");
}

#[tokio::test]
async fn test_transfer() {
    let mock_repo = Arc::new(MockTransactionRepository::new());
    let service = TransactionService::new(mock_repo, None);
    let from_id = Uuid::new_v4();
    let to_id = Uuid::new_v4();
    let amount = dec!(25.00);

    let request = TransferRequest { from_account_id: from_id, to_account_id: to_id, amount, idempotency_key: None };
    let response = service.transfer(request).await.expect("Transfer failed");

    assert_eq!(response.from_account_id, Some(from_id));
    assert_eq!(response.to_account_id, Some(to_id));
    assert_eq!(response.transaction_type, "transfer");
}

#[tokio::test]
async fn test_get_history() {
    let mock_repo = Arc::new(MockTransactionRepository::new());
    let service = TransactionService::new(mock_repo.clone(), None);
    let account_id = Uuid::new_v4();
    
    // Seed some transactions (using deposit helper for mock simplicity)
    service.deposit(DepositRequest { account_id, amount: dec!(100.00), idempotency_key: None }).await.unwrap();
    service.withdraw(WithdrawRequest { account_id, amount: dec!(20.00), idempotency_key: None }).await.unwrap();

    let history = service.get_history(account_id, 10, 0).await.expect("Failed to get history");
    
    assert_eq!(history.len(), 2);
}
