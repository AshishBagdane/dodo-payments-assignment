use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::domain::entities::Webhook;
use crate::domain::value_objects::WebhookEvent;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub event: WebhookEvent,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub url: String,
    pub event: WebhookEvent,
    pub account_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<Webhook> for WebhookResponse {
    fn from(webhook: Webhook) -> Self {
        Self {
            id: webhook.id,
            url: webhook.url,
            event: webhook.event,
            account_id: webhook.account_id,
            created_at: webhook.created_at,
        }
    }
}
