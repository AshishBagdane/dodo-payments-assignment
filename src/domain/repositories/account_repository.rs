use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::Account;
use crate::domain::errors::RepositoryError;
use crate::domain::value_objects::Money;

/// Repository trait for Account persistence operations
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Create a new account
    async fn create(&self, account: &Account) -> Result<Account, RepositoryError>;

    /// Find account by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Account, RepositoryError>;

    /// Update account balance
    async fn update_balance(&self, id: Uuid, new_balance: Money) -> Result<(), RepositoryError>;

    /// Update account business name
    async fn update_business_name(&self, id: Uuid, name: String) -> Result<(), RepositoryError>;

    /// Check if account exists
    async fn exists(&self, id: Uuid) -> Result<bool, RepositoryError>;

    /// List all accounts (paginated)
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Account>, RepositoryError>;

    /// Delete account (soft delete recommended in production)
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}