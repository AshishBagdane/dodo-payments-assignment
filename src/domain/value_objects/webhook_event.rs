use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebhookEvent {
    #[serde(rename = "transaction.completed")]
    TransactionCompleted,
    #[serde(rename = "account.created")]
    AccountCreated,
}

impl std::fmt::Display for WebhookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookEvent::TransactionCompleted => write!(f, "transaction.completed"),
            WebhookEvent::AccountCreated => write!(f, "account.created"),
        }
    }
}
