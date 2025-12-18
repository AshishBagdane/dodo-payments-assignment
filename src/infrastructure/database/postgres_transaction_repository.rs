use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Transaction;
use crate::domain::errors::RepositoryError;
use crate::domain::repositories::TransactionRepository;
use crate::domain::value_objects::TransactionType;

/// PostgreSQL implementation of the TransactionRepository
pub struct PostgresTransactionRepository {
    pool: PgPool,
}

impl PostgresTransactionRepository {
    /// Create a new PostgresTransactionRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionRepository for PostgresTransactionRepository {
    async fn create(&self, transaction: &Transaction) -> Result<Transaction, RepositoryError> {
        let row = sqlx::query(
            r#"
            INSERT INTO transactions (
                id, transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            )
            VALUES ($1, $2::transaction_type, $3, $4, $5, $6, $7)
            RETURNING id, transaction_type::text as transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            "#,
        )
        .bind(transaction.id)
        .bind(transaction.transaction_type.as_str())
        .bind(transaction.from_account_id)
        .bind(transaction.to_account_id)
        .bind(transaction.amount.amount())
        .bind(&transaction.idempotency_key)
        .bind(transaction.created_at)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            let type_str: String = row.get("transaction_type");
            let transaction_type = TransactionType::from_str(&type_str)?;
            let amount_decimal: rust_decimal::Decimal = row.get("amount");
            let amount = crate::domain::value_objects::Money::new(amount_decimal)?;

            Transaction::from_db(
                row.get("id"),
                transaction_type,
                row.get("from_account_id"),
                row.get("to_account_id"),
                amount,
                row.get("idempotency_key"),
                row.get("created_at"),
            )
        })
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return RepositoryError::DuplicateEntry(format!(
                        "Transaction with idempotency key {:?} already exists",
                        transaction.idempotency_key
                    ));
                }
            }
            RepositoryError::from(e)
        })?;

        row.map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e)))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Transaction, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, transaction_type::text as transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            FROM transactions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            let type_str: String = row.get("transaction_type");
            let transaction_type = TransactionType::from_str(&type_str)?;
            let amount_decimal: rust_decimal::Decimal = row.get("amount");
            let amount = crate::domain::value_objects::Money::new(amount_decimal)?;

            Transaction::from_db(
                row.get("id"),
                transaction_type,
                row.get("from_account_id"),
                row.get("to_account_id"),
                amount,
                row.get("idempotency_key"),
                row.get("created_at"),
            )
        })
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        match row {
            Some(tx_result) => tx_result
                .map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e))),
            None => Err(RepositoryError::NotFound(format!("Transaction {} not found", id))),
        }
    }

    async fn find_by_idempotency_key(&self, key: &str) -> Result<Transaction, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, transaction_type::text as transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            FROM transactions
            WHERE idempotency_key = $1
            "#,
        )
        .bind(key)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            let type_str: String = row.get("transaction_type");
            let transaction_type = TransactionType::from_str(&type_str)?;
            let amount_decimal: rust_decimal::Decimal = row.get("amount");
            let amount = crate::domain::value_objects::Money::new(amount_decimal)?;

            Transaction::from_db(
                row.get("id"),
                transaction_type,
                row.get("from_account_id"),
                row.get("to_account_id"),
                amount,
                row.get("idempotency_key"),
                row.get("created_at"),
            )
        })
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        match row {
            Some(tx_result) => tx_result
                .map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e))),
            None => Err(RepositoryError::NotFound(format!("Transaction with key {} not found", key))),
        }
    }

    async fn idempotency_key_exists(&self, key: &str) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            r#"
            SELECT 1 as exists
            FROM transactions
            WHERE idempotency_key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(result.is_some())
    }

    async fn list_by_account(
        &self,
        account_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, RepositoryError> {
        let rows = sqlx::query(
            r#"
            SELECT id, transaction_type::text as transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            FROM transactions
            WHERE from_account_id = $1 OR to_account_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .bind(offset)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            let type_str: String = row.get("transaction_type");
            let transaction_type = TransactionType::from_str(&type_str)?;
            let amount_decimal: rust_decimal::Decimal = row.get("amount");
            let amount = crate::domain::value_objects::Money::new(amount_decimal)?;

            Transaction::from_db(
                row.get("id"),
                transaction_type,
                row.get("from_account_id"),
                row.get("to_account_id"),
                amount,
                row.get("idempotency_key"),
                row.get("created_at"),
            )
        })
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        let mut transactions = Vec::new();
        for tx_result in rows {
            transactions.push(tx_result.map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e)))?);
        }

        Ok(transactions)
    }

    async fn list_by_type(
        &self,
        transaction_type: TransactionType,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, RepositoryError> {
        let rows = sqlx::query(
            r#"
            SELECT id, transaction_type::text as transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            FROM transactions
            WHERE transaction_type = $1::transaction_type
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(transaction_type.as_str())
        .bind(limit)
        .bind(offset)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            let type_str: String = row.get("transaction_type");
            let transaction_type = TransactionType::from_str(&type_str)?;
            let amount_decimal: rust_decimal::Decimal = row.get("amount");
            let amount = crate::domain::value_objects::Money::new(amount_decimal)?;

            Transaction::from_db(
                row.get("id"),
                transaction_type,
                row.get("from_account_id"),
                row.get("to_account_id"),
                amount,
                row.get("idempotency_key"),
                row.get("created_at"),
            )
        })
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        let mut transactions = Vec::new();
        for tx_result in rows {
            transactions.push(tx_result.map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e)))?);
        }

        Ok(transactions)
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Transaction>, RepositoryError> {
         let rows = sqlx::query(
            r#"
            SELECT id, transaction_type::text as transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            FROM transactions
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            let type_str: String = row.get("transaction_type");
            let transaction_type = TransactionType::from_str(&type_str)?;
            let amount_decimal: rust_decimal::Decimal = row.get("amount");
            let amount = crate::domain::value_objects::Money::new(amount_decimal)?;

            Transaction::from_db(
                row.get("id"),
                transaction_type,
                row.get("from_account_id"),
                row.get("to_account_id"),
                amount,
                row.get("idempotency_key"),
                row.get("created_at"),
            )
        })
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        let mut transactions = Vec::new();
        for tx_result in rows {
            transactions.push(tx_result.map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e)))?);
        }

        Ok(transactions)
    }

    async fn execute_credit(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(RepositoryError::from)?;
        let to_account_id = transaction.to_account_id.ok_or_else(|| {
             RepositoryError::ConstraintViolation("Credit transaction must have to_account_id".to_string())
        })?;

        // 1. Update Account Balance
        let update_result = sqlx::query(
            r#"
            UPDATE accounts
            SET balance = balance + $1, updated_at = NOW()
            WHERE id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(transaction.amount.amount())
        .bind(to_account_id)
        .execute(&mut *tx)
        .await
        .map_err(RepositoryError::from)?;

        if update_result.rows_affected() == 0 {
             return Err(RepositoryError::NotFound(format!("Account {} not found", to_account_id)));
        }

        // 2. Create Transaction Record
        let tx_row = sqlx::query(
             r#"
            INSERT INTO transactions (
                id, transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            )
            VALUES ($1, $2::transaction_type, $3, $4, $5, $6, $7)
            RETURNING id, transaction_type::text as transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            "#,
        )
        .bind(transaction.id)
        .bind(transaction.transaction_type.as_str())
        .bind(transaction.from_account_id)
        .bind(transaction.to_account_id)
        .bind(transaction.amount.amount())
        .bind(&transaction.idempotency_key)
        .bind(transaction.created_at)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            let type_str: String = row.get("transaction_type");
            let transaction_type = TransactionType::from_str(&type_str)?;
            let amount_decimal: rust_decimal::Decimal = row.get("amount");
            let amount = crate::domain::value_objects::Money::new(amount_decimal)?;

             Transaction::from_db(
                row.get("id"),
                transaction_type,
                row.get("from_account_id"),
                row.get("to_account_id"),
                amount,
                row.get("idempotency_key"),
                row.get("created_at"),
            )
        })
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
             if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return RepositoryError::DuplicateEntry(format!(
                        "Transaction with idempotency key {:?} already exists",
                        transaction.idempotency_key
                    ));
                }
            }
            RepositoryError::from(e)
        })?;

        tx.commit().await.map_err(RepositoryError::from)?;

        tx_row.map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e)))
    }

    async fn execute_debit(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(RepositoryError::from)?;
        let from_account_id = transaction.from_account_id.ok_or_else(|| {
             RepositoryError::ConstraintViolation("Debit transaction must have from_account_id".to_string())
        })?;

        // 1. Check Balance and Lock Row (SELECT FOR UPDATE)
        let account_row = sqlx::query(
            "SELECT balance FROM accounts WHERE id = $1 AND deleted_at IS NULL FOR UPDATE"
        )
        .bind(from_account_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(RepositoryError::from)?;

        let account_balance = match account_row {
            Some(row) => {
                use sqlx::Row;
                row.get::<rust_decimal::Decimal, _>("balance")
            },
            None => return Err(RepositoryError::NotFound(format!("Account {} not found", from_account_id))),
        };

        if account_balance < transaction.amount.amount() {
             return Err(RepositoryError::ConstraintViolation(format!(
                 "Insufficient funds for account {}", from_account_id
             )));
        }

        // 2. Deduct Balance
        sqlx::query(
             r#"
            UPDATE accounts
            SET balance = balance - $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(transaction.amount.amount())
        .bind(from_account_id)
        .execute(&mut *tx)
        .await
        .map_err(RepositoryError::from)?;

        // 3. Create Transaction Record
        let tx_row = sqlx::query(
             r#"
            INSERT INTO transactions (
                id, transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            )
            VALUES ($1, $2::transaction_type, $3, $4, $5, $6, $7)
            RETURNING id, transaction_type::text as transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            "#,
        )
        .bind(transaction.id)
        .bind(transaction.transaction_type.as_str())
        .bind(transaction.from_account_id)
        .bind(transaction.to_account_id)
        .bind(transaction.amount.amount())
        .bind(&transaction.idempotency_key)
        .bind(transaction.created_at)
        .map(|row: sqlx::postgres::PgRow| {
             use sqlx::Row;
            let type_str: String = row.get("transaction_type");
            let transaction_type = TransactionType::from_str(&type_str)?;
            let amount_decimal: rust_decimal::Decimal = row.get("amount");
            let amount = crate::domain::value_objects::Money::new(amount_decimal)?;

             Transaction::from_db(
                row.get("id"),
                transaction_type,
                row.get("from_account_id"),
                row.get("to_account_id"),
                amount,
                row.get("idempotency_key"),
                row.get("created_at"),
            )
        })
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
             if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return RepositoryError::DuplicateEntry(format!(
                        "Transaction with idempotency key {:?} already exists",
                        transaction.idempotency_key
                    ));
                }
            }
            RepositoryError::from(e)
        })?;

        tx.commit().await.map_err(RepositoryError::from)?;

        tx_row.map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e)))
    }

    async fn execute_transfer(
        &self,
        transaction: &Transaction,
    ) -> Result<Transaction, RepositoryError> {
        // Enforce ordering to prevent deadlocks: lock smaller ID first
        let from_id = transaction.from_account_id.ok_or_else(|| {
              RepositoryError::ConstraintViolation("Transfer must have from_account_id".to_string())
        })?;
        let to_id = transaction.to_account_id.ok_or_else(|| {
              RepositoryError::ConstraintViolation("Transfer must have to_account_id".to_string())
        })?;

         if from_id == to_id {
            return Err(RepositoryError::ConstraintViolation("Cannot transfer to same account".to_string()));
         }

        let mut tx = self.pool.begin().await.map_err(RepositoryError::from)?;

        // Lock rows in consistent order
        let first_id = if from_id < to_id { from_id } else { to_id };
        let second_id = if from_id < to_id { to_id } else { from_id };

        // 1. Check existence and lock both accounts
        let _ = sqlx::query("SELECT id FROM accounts WHERE id = $1 AND deleted_at IS NULL FOR UPDATE")
            .bind(first_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::NotFound(format!("Account {} not found", first_id)))?;

        let _ = sqlx::query("SELECT id FROM accounts WHERE id = $1 AND deleted_at IS NULL FOR UPDATE")
            .bind(second_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(RepositoryError::from)?
            .ok_or_else(|| RepositoryError::NotFound(format!("Account {} not found", second_id)))?;


        // 2. Check Balance of From Account
        let from_balance_row = sqlx::query("SELECT balance FROM accounts WHERE id = $1")
            .bind(from_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(RepositoryError::from)?;

        let from_balance: rust_decimal::Decimal = {
            use sqlx::Row;
            from_balance_row.get("balance")
        };

        if from_balance < transaction.amount.amount() {
            return Err(RepositoryError::ConstraintViolation(format!(
                 "Insufficient funds for account {}", from_id
             )));
        }

        // 3. Debit From-Account
        sqlx::query(
             r#"
            UPDATE accounts
            SET balance = balance - $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(transaction.amount.amount())
        .bind(from_id)
        .execute(&mut *tx)
        .await
        .map_err(RepositoryError::from)?;

        // 4. Credit To-Account
        sqlx::query(
             r#"
            UPDATE accounts
            SET balance = balance + $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(transaction.amount.amount())
        .bind(to_id)
        .execute(&mut *tx)
        .await
        .map_err(RepositoryError::from)?;

        // 5. Create Transaction Record
        let tx_row = sqlx::query(
             r#"
            INSERT INTO transactions (
                id, transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            )
            VALUES ($1, $2::transaction_type, $3, $4, $5, $6, $7)
            RETURNING id, transaction_type::text as transaction_type, from_account_id, to_account_id, amount, idempotency_key, created_at
            "#,
        )
        .bind(transaction.id)
        .bind(transaction.transaction_type.as_str())
        .bind(transaction.from_account_id)
        .bind(transaction.to_account_id)
        .bind(transaction.amount.amount())
        .bind(&transaction.idempotency_key)
        .bind(transaction.created_at)
        .map(|row: sqlx::postgres::PgRow| {
             use sqlx::Row;
            let type_str: String = row.get("transaction_type");
            let transaction_type = TransactionType::from_str(&type_str)?;
            let amount_decimal: rust_decimal::Decimal = row.get("amount");
            let amount = crate::domain::value_objects::Money::new(amount_decimal)?;

             Transaction::from_db(
                row.get("id"),
                transaction_type,
                row.get("from_account_id"),
                row.get("to_account_id"),
                amount,
                row.get("idempotency_key"),
                row.get("created_at"),
            )
        })
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
             if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return RepositoryError::DuplicateEntry(format!(
                        "Transaction with idempotency key {:?} already exists",
                        transaction.idempotency_key
                    ));
                }
            }
            RepositoryError::from(e)
        })?;

        tx.commit().await.map_err(RepositoryError::from)?;

        tx_row.map_err(|e| RepositoryError::DatabaseError(format!("Data integrity error: {}", e)))
    }
}
