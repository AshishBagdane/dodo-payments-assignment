use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::WebhookEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: Uuid,
    pub url: String,
    pub event: WebhookEvent,
    pub account_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Webhook {
    pub fn new(account_id: Uuid, url: String, event: WebhookEvent) -> Result<Self, String> {
        if url.trim().is_empty() {
            return Err("URL cannot be empty".to_string());
        }
        // Basic URL validation
        if !url.starts_with("http") {
             return Err("URL must start with http or https".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            url,
            event,
            account_id,
            created_at: Utc::now(),
        })
    }
}
