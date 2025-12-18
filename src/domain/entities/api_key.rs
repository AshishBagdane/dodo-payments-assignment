use chrono::{DateTime, Utc};
use uuid::Uuid;

/// API Key data structure
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: Uuid,
    pub key_hash: String,
    pub account_id: Uuid,
    pub rate_limit_per_hour: u32,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl ApiKey {
    /// Create a new API key instance (for domain logic, not persistence)
    pub fn new(account_id: Uuid, key_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            key_hash,
            account_id,
            rate_limit_per_hour: 1000,
            created_at: Utc::now(),
            last_used_at: None,
        }
    }

    /// Reconstruct from database
    pub fn from_db(
        id: Uuid,
        key_hash: String,
        account_id: Uuid,
        rate_limit_per_hour: i32,
        created_at: DateTime<Utc>,
        last_used_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            key_hash,
            account_id,
            rate_limit_per_hour: rate_limit_per_hour as u32,
            created_at,
            last_used_at,
        }
    }
}
