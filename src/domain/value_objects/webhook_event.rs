use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebhookEvent {
    #[serde(rename = "transaction.completed")]
    TransactionCompleted,
    #[serde(rename = "account.created")]
    AccountCreated,
}

impl ToString for WebhookEvent {
    fn to_string(&self) -> String {
        match self {
            WebhookEvent::TransactionCompleted => "transaction.completed".to_string(),
            WebhookEvent::AccountCreated => "account.created".to_string(),
        }
    }
}
