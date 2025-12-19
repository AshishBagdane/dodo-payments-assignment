use async_trait::async_trait;


#[async_trait]
pub trait WebhookDispatcher: Send + Sync {
    async fn dispatch(&self, url: &str, payload: &serde_json::Value, secret: &str) -> Result<(), String>;
}
