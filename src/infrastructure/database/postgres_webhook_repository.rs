use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Webhook;
use crate::domain::errors::RepositoryError;
use crate::domain::repositories::WebhookRepository;
use crate::domain::value_objects::WebhookEvent;

pub struct PostgresWebhookRepository {
    pool: PgPool,
}

impl PostgresWebhookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WebhookRepository for PostgresWebhookRepository {
    async fn create(&self, webhook: Webhook) -> Result<Webhook, RepositoryError> {
        let event_str = webhook.event.to_string();

        sqlx::query!(
            r#"
            INSERT INTO webhooks (id, account_id, url, event, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $5)
            "#,
            webhook.id,
            webhook.account_id,
            webhook.url,
            event_str,
            webhook.created_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(webhook)
    }

    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Webhook>, RepositoryError> {
        let records = sqlx::query!(
            r#"
            SELECT id, account_id, url, event, created_at 
            FROM webhooks 
            WHERE account_id = $1
            ORDER BY created_at DESC
            "#,
            account_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut webhooks = Vec::new();
        for r in records {
            let event = serde_json::from_str::<WebhookEvent>(&format!("\"{}\"", r.event))
                .map_err(|e| RepositoryError::DatabaseError(format!("Invalid event type: {}", e)))?;

            webhooks.push(Webhook {
                id: r.id,
                account_id: r.account_id,
                url: r.url,
                event,
                created_at: r.created_at,
            });
        }

        Ok(webhooks)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM webhooks WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound("Webhook not found".to_string()));
        }

        Ok(())
    }
}
