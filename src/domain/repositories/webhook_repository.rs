use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::Webhook;
use crate::domain::errors::RepositoryError;

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn create(&self, webhook: Webhook) -> Result<Webhook, RepositoryError>;
    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Webhook>, RepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}