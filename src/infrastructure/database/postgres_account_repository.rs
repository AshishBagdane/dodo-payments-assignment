use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Account;
use crate::domain::errors::RepositoryError;
use crate::domain::repositories::AccountRepository;
use crate::domain::value_objects::Money;

/// PostgreSQL implementation of the AccountRepository
pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    /// Create a new PostgresAccountRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn create(&self, account: &Account) -> Result<Account, RepositoryError> {
        let row = sqlx::query(
            r#"
            INSERT INTO accounts (id, business_name, balance, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, business_name, balance, created_at, updated_at, deleted_at
            "#,
        )
        .bind(account.id)
        .bind(&account.business_name)
        .bind(account.balance_as_decimal())
        .bind(account.created_at)
        .bind(account.updated_at)
        .bind(account.deleted_at)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            Account::from_db(
                row.get("id"),
                row.get("business_name"),
                row.get("balance"),
                row.get("created_at"),
                row.get("updated_at"),
                row.get("deleted_at"),
            )
        })
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(row.map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e)))?)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Account, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, business_name, balance, created_at, updated_at, deleted_at 
            FROM accounts 
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            Account::from_db(
                row.get("id"),
                row.get("business_name"),
                row.get("balance"),
                row.get("created_at"),
                row.get("updated_at"),
                row.get("deleted_at"),
            )
        })
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        match row {
            Some(account_result) => Ok(account_result
                .map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e)))?),
            None => Err(RepositoryError::NotFound(format!("Account {} not found", id))),
        }
    }

    async fn update_balance(&self, id: Uuid, new_balance: Money) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE accounts 
            SET balance = $1, updated_at = NOW() 
            WHERE id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(new_balance.amount())
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            // Check if account exists but is deleted, or doesn't exist
            let exists = self.exists(id).await?;
            if !exists {
                return Err(RepositoryError::NotFound(format!("Account {} not found or deleted", id)));
            }
        }

        Ok(())
    }

    async fn update_business_name(&self, id: Uuid, name: String) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE accounts 
            SET business_name = $1, updated_at = NOW() 
            WHERE id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(name)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            let exists = self.exists(id).await?;
            if !exists {
                return Err(RepositoryError::NotFound(format!("Account {} not found or deleted", id)));
            }
        }

        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            r#"
            SELECT 1 as exists 
            FROM accounts 
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Account>, RepositoryError> {
        let rows = sqlx::query(
            r#"
            SELECT id, business_name, balance, created_at, updated_at, deleted_at 
            FROM accounts 
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            Account::from_db(
                row.get("id"),
                row.get("business_name"),
                row.get("balance"),
                row.get("created_at"),
                row.get("updated_at"),
                row.get("deleted_at"),
            )
        })
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        let mut accounts = Vec::new();
        for account_result in rows {
            accounts.push(account_result.map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e)))?);
        }

        Ok(accounts)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE accounts 
            SET deleted_at = NOW() 
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
             return Err(RepositoryError::NotFound(format!("Account {} not found or already deleted", id)));
        }

        Ok(())
    }
}
