use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::errors::RepositoryError;

/// API Key data structure
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: Uuid,
    pub key_hash: String,
    pub account_id: Uuid,
    pub rate_limit_per_hour: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

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