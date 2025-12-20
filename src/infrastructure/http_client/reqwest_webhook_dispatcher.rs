use async_trait::async_trait;
use hmac::Mac;
use reqwest::Client;
use std::time::Duration;

use crate::domain::services::WebhookDispatcher;


use rand::Rng; // For jitter
use tracing::{error, warn};

#[derive(Clone)]
pub struct ReqwestWebhookDispatcher {
    client: Client,
    max_retries: u32,
    initial_backoff_ms: u64,
}

impl ReqwestWebhookDispatcher {
    pub fn new(max_retries: u32, initial_backoff_ms: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { client, max_retries, initial_backoff_ms }
    }
}

#[async_trait]
impl WebhookDispatcher for ReqwestWebhookDispatcher {
    async fn dispatch(&self, url: &str, payload: &serde_json::Value, secret: &str) -> Result<(), String> {
        let payload_string = serde_json::to_string(payload).map_err(|e| e.to_string())?;
        
        // Compute HMAC
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())
            .map_err(|e| format!("Invalid HMAC secret: {}", e))?;
        hmac::Mac::update(&mut mac, payload_string.as_bytes());
        let result = mac.finalize();
        let signature_hex = hex::encode(result.into_bytes());

        let mut attempt = 0;
        let mut backoff = self.initial_backoff_ms;

        loop {
            let result = self.client
                .post(url)
                .header("Content-Type", "application/json")
                .header("X-Dodo-Signature", format!("sha256={}", signature_hex))
                .body(payload_string.clone())
                .send()
                .await;

            match result {
                Ok(res) => {
                    if res.status().is_success() {
                        return Ok(());
                    } else {
                        warn!("Webhook dispatch attempt {} failed with status: {}", attempt + 1, res.status());
                    }
                }
                Err(e) => {
                    warn!("Webhook dispatch attempt {} failed with error: {}", attempt + 1, e);
                }
            }

            attempt += 1;
            if attempt > self.max_retries {
                let msg = format!("Webhook dispatch failed after {} attempts", self.max_retries);
                error!("{}", msg);
                return Err(msg);
            }

            // Exponential backoff with jitter
            let jitter: u64 = rand::rng().random_range(0..100);
            let sleep_duration = Duration::from_millis(backoff + jitter);
            tokio::time::sleep(sleep_duration).await;
            
            backoff *= 2;
        }
    }
}
