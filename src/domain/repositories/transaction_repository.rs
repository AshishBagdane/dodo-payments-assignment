use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::Transaction;
use crate::domain::errors::RepositoryError;
use crate::domain::value_objects::TransactionType;

/// Repository trait for Transaction persistence operations
#[async_trait]
pub trait TransactionRepository: Send + Sync {
    /// Create a new transaction
    /// Returns error if idempotency key already exists
    async fn create(&self, transaction: &Transaction) -> Result<Transaction, RepositoryError>;

    /// Find transaction by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Transaction, RepositoryError>;

    /// Find transaction by idempotency key
    async fn find_by_idempotency_key(&self, key: &str) -> Result<Transaction, RepositoryError>;

    /// Check if idempotency key exists
    async fn idempotency_key_exists(&self, key: &str) -> Result<bool, RepositoryError>;

    /// List transactions for an account (paginated)
    async fn list_by_account(
        &self,
        account_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, RepositoryError>;

    /// List transactions by type (paginated)
    async fn list_by_type(
        &self,
        transaction_type: TransactionType,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, RepositoryError>;

    /// List all transactions (paginated)
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Transaction>, RepositoryError>;

    /// Execute credit transaction atomically
    /// Updates account balance and creates transaction record
    async fn execute_credit(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, RepositoryError>;

    /// Execute debit transaction atomically
    /// Updates account balance and creates transaction record
    async fn execute_debit(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, RepositoryError>;

    /// Execute transfer transaction atomically
    /// Updates both account balances and creates transaction record
    async fn execute_transfer(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, RepositoryError>;
}