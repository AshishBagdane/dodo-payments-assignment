use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "retrying")]
    Retrying,
}

impl ToString for DeliveryStatus {
    fn to_string(&self) -> String {
        match self {
            DeliveryStatus::Pending => "pending".to_string(),
            DeliveryStatus::Success => "success".to_string(),
            DeliveryStatus::Failed => "failed".to_string(),
            DeliveryStatus::Retrying => "retrying".to_string(),
        }
    }
}
