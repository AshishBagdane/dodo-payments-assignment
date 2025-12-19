use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::Transaction;

#[derive(Debug, Deserialize, Serialize)]
pub struct DepositRequest {
    pub account_id: Uuid,
    pub amount: Decimal,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WithdrawRequest {
    pub account_id: Uuid,
    pub amount: Decimal,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransferRequest {
    pub from_account_id: Uuid,
    pub to_account_id: Uuid,
    pub amount: Decimal,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub transaction_type: String,
    pub from_account_id: Option<Uuid>,
    pub to_account_id: Option<Uuid>,
    pub amount: Decimal,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Transaction> for TransactionResponse {
    fn from(transaction: Transaction) -> Self {
        Self {
            id: transaction.id,
            transaction_type: transaction.transaction_type.as_str().to_string(),
            from_account_id: transaction.from_account_id,
            to_account_id: transaction.to_account_id,
            amount: transaction.amount.amount(),
            idempotency_key: transaction.idempotency_key,
            created_at: transaction.created_at,
        }
    }
}
