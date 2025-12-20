use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::ApiKey;
use crate::domain::errors::RepositoryError;
use crate::domain::repositories::ApiKeyRepository;

pub struct PostgresApiKeyRepository {
    pool: PgPool,
}

impl PostgresApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiKeyRepository for PostgresApiKeyRepository {
    async fn create(&self, api_key: &ApiKey) -> Result<ApiKey, RepositoryError> {
        let row = sqlx::query(
            r#"
            INSERT INTO api_keys (id, key_hash, account_id, rate_limit_per_hour, created_at, last_used_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, key_hash, account_id, rate_limit_per_hour, created_at, last_used_at
            "#,
        )
        .bind(api_key.id)
        .bind(&api_key.key_hash)
        .bind(api_key.account_id)
        .bind(api_key.rate_limit_per_hour as i32)
        .bind(api_key.created_at)
        .bind(api_key.last_used_at)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            ApiKey::from_db(
                row.get("id"),
                row.get("key_hash"),
                row.get("account_id"),
                row.get("rate_limit_per_hour"),
                row.get("created_at"),
                row.get("last_used_at"),
            )
        })
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return RepositoryError::DuplicateEntry(
                        "API key hash already exists".to_string()
                    );
                }
            }
            RepositoryError::from(e)
        })?;

        Ok(row)
    }

    async fn find_by_hash(&self, key_hash: &str) -> Result<ApiKey, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, key_hash, account_id, rate_limit_per_hour, created_at, last_used_at
            FROM api_keys
            WHERE key_hash = $1
            "#,
        )
        .bind(key_hash)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            ApiKey::from_db(
                row.get("id"),
                row.get("key_hash"),
                row.get("account_id"),
                row.get("rate_limit_per_hour"),
                row.get("created_at"),
                row.get("last_used_at"),
            )
        })
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        match row {
            Some(key) => Ok(key),
            None => Err(RepositoryError::NotFound("API key not found".to_string())),
        }
    }

    async fn find_by_account(&self, account_id: Uuid) -> Result<Vec<ApiKey>, RepositoryError> {
        let rows = sqlx::query(
            r#"
            SELECT id, key_hash, account_id, rate_limit_per_hour, created_at, last_used_at
            FROM api_keys
            WHERE account_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(account_id)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            ApiKey::from_db(
                row.get("id"),
                row.get("key_hash"),
                row.get("account_id"),
                row.get("rate_limit_per_hour"),
                row.get("created_at"),
                row.get("last_used_at"),
            )
        })
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(rows)
    }

    async fn update_last_used(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        if result.rows_affected() == 0 {
             return Err(RepositoryError::NotFound(format!("API key {} not found", id)));
        }

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            DELETE FROM api_keys
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        if result.rows_affected() == 0 {
             return Err(RepositoryError::NotFound(format!("API key {} not found", id)));
        }

        Ok(())
    }

    async fn exists(&self, key_hash: &str) -> Result<bool, RepositoryError> {
        let result = sqlx::query("SELECT 1 FROM api_keys WHERE key_hash = $1")
            .bind(key_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::from)?;

        Ok(result.is_some())
    }
}
