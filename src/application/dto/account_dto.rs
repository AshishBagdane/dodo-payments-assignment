use crate::domain::entities::Account;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAccountRequest {
    pub business_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub business_name: String,
    pub balance: Decimal,
    pub created_at: DateTime<Utc>,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            business_name: account.business_name,
            balance: account.balance.amount(), // Assuming Money has an amount() method returning Decimal
            created_at: account.created_at,
        }
    }
}
