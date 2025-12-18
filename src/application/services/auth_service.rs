use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::errors::{DomainError, ServiceError};
use crate::domain::repositories::ApiKeyRepository;

/// Principal authenticated via API Key
#[derive(Debug, Clone)]
pub struct AuthPrincipal {
    pub account_id: Uuid,
}

pub struct AuthService {
    repository: Arc<dyn ApiKeyRepository>,
}

impl AuthService {
    pub fn new(repository: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repository }
    }

    /// Verify an API key string against stored hashes
    pub async fn verify_api_key(&self, raw_key: &str) -> Result<AuthPrincipal, ServiceError> {
        // Hash the raw key
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        let hash_bytes = hasher.finalize();
        let hash_string = hex::encode(hash_bytes);

        // Check if key exists in repository
        let api_key = self
            .repository
            .find_by_hash(&hash_string)
            .await
            .map_err(|e| match e {
                crate::domain::errors::RepositoryError::NotFound(_) => {
                    ServiceError::Domain(DomainError::InvalidApiKey)
                }
                _ => ServiceError::from(e),
            })?;

        // Update last used timestamp (fire and forget or await?)
        // For strict reliability we await, but failures here shouldn't block auth ideally.
        // For this assignment, we await.
        if let Err(e) = self.repository.update_last_used(api_key.id).await {
            // Log error but don't fail auth?
            // println!("Failed to update last used for key {}: {}", api_key.id, e);
            // In production, use tracing::warn!
            // For now, ignoring failure is acceptable or returning internal error.
            // Let's safe-fail (ignore update error for auth success)
            eprintln!("Warning: Failed to update API key last_used: {}", e);
        }

        Ok(AuthPrincipal {
            account_id: api_key.account_id,
        })
    }
}
