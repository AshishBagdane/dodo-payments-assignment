use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::errors::RepositoryError;

/// Webhook event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebhookEvent {
    TransactionCreated,
    TransactionCompleted,
    AccountUpdated,
}

impl WebhookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TransactionCreated => "transaction.created",
            Self::TransactionCompleted => "transaction.completed",
            Self::AccountUpdated => "account.updated",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "transaction.created" => Some(Self::TransactionCreated),
            "transaction.completed" => Some(Self::TransactionCompleted),
            "account.updated" => Some(Self::AccountUpdated),
            _ => None,
        }
    }
}

/// Webhook data structure
#[derive(Debug, Clone)]
pub struct Webhook {
    pub id: Uuid,
    pub account_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<WebhookEvent>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Webhook delivery status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryStatus {
    Pending,
    Success,
    Failed,
    Retrying,
}

impl DeliveryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Retrying => "retrying",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "success" => Some(Self::Success),
            "failed" => Some(Self::Failed),
            "retrying" => Some(Self::Retrying),
            _ => None,
        }
    }
}

/// Webhook delivery data structure
#[derive(Debug, Clone)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub transaction_id: Uuid,
    pub event_type: WebhookEvent,
    pub status: DeliveryStatus,
    pub attempts: i32,
    pub last_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
    pub response_status_code: Option<i32>,
    pub response_body: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Repository trait for Webhook persistence operations
#[async_trait]
pub trait WebhookRepository: Send + Sync {
    /// Create a new webhook
    async fn create(&self, webhook: &Webhook) -> Result<Webhook, RepositoryError>;

    /// Find webhook by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Webhook, RepositoryError>;

    /// Find active webhooks for an account
    async fn find_by_account(&self, account_id: Uuid) -> Result<Vec<Webhook>, RepositoryError>;

    /// Find active webhooks subscribed to an event
    async fn find_by_event(&self, event: WebhookEvent) -> Result<Vec<Webhook>, RepositoryError>;

    /// Update webhook
    async fn update(&self, webhook: &Webhook) -> Result<(), RepositoryError>;

    /// Delete webhook
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Create webhook delivery record
    async fn create_delivery(
        &self,
        delivery: &WebhookDelivery,
    ) -> Result<WebhookDelivery, RepositoryError>;

    /// Update webhook delivery status
    async fn update_delivery_status(
        &self,
        delivery_id: Uuid,
        status: DeliveryStatus,
        response_status_code: Option<i32>,
        response_body: Option<String>,
    ) -> Result<(), RepositoryError>;

    /// Get pending deliveries for retry
    async fn get_pending_deliveries(&self) -> Result<Vec<WebhookDelivery>, RepositoryError>;

    /// Update delivery retry information
    async fn update_delivery_retry(
        &self,
        delivery_id: Uuid,
        attempts: i32,
        next_retry_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepositoryError>;
}