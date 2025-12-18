use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::ApiKey;
use crate::domain::errors::RepositoryError;

/// Repository trait for API Key persistence operations
#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    /// Create a new API key
    async fn create(&self, api_key: &ApiKey) -> Result<ApiKey, RepositoryError>;

    /// Find API key by hash
    async fn find_by_hash(&self, key_hash: &str) -> Result<ApiKey, RepositoryError>;

    /// Find API keys by account ID
    async fn find_by_account(&self, account_id: Uuid) -> Result<Vec<ApiKey>, RepositoryError>;

    /// Update last used timestamp
    async fn update_last_used(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Delete API key
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Check if key hash exists
    async fn exists(&self, key_hash: &str) -> Result<bool, RepositoryError>;
}