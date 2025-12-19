use std::sync::Arc;
use tokio::task;
use tracing::{error, Instrument};
use uuid::Uuid;

use crate::domain::services::WebhookDispatcher;
use crate::domain::repositories::{AccountRepository, WebhookRepository};
use crate::domain::value_objects::WebhookEvent;

#[derive(Clone)]
pub struct WebhookService {
    webhook_repository: Arc<dyn WebhookRepository>,
    account_repository: Arc<dyn AccountRepository>,
    dispatcher: Arc<dyn WebhookDispatcher>,
}

impl WebhookService {
    pub fn new(
        webhook_repository: Arc<dyn WebhookRepository>,
        account_repository: Arc<dyn AccountRepository>,
        dispatcher: Arc<dyn WebhookDispatcher>,
    ) -> Self {
        Self {
            webhook_repository,
            account_repository,
            dispatcher,
        }
    }

    #[tracing::instrument(skip(self, payload))]
    pub fn notify_async<T>(
        &self,
        account_id: Uuid,
        event: WebhookEvent,
        payload: T,
    ) where
        T: serde::Serialize + Send + Sync + 'static,
    {
        let repo = self.webhook_repository.clone();
        let account_repo = self.account_repository.clone();
        let dispatcher = self.dispatcher.clone();

        // Capture current span
        let span = tracing::Span::current();

        task::spawn(async move {
            // First, fetch the account to get the secret
            let account = match account_repo.find_by_id(account_id).await {
                Ok(acc) => acc,
                Err(e) => {
                    error!("Failed to fetch account {} for webhook dispatch: {}", account_id, e);
                    return;
                }
            };
            let secret = account.webhook_secret;

            match repo.list_by_account(account_id).await {
                Ok(webhooks) => {
                    for webhook in webhooks {
                        if webhook.event == event {
                            match serde_json::to_value(&payload) {
                                Ok(value) => {
                                    if let Err(e) = dispatcher.dispatch(&webhook.url, &value, &secret).await {
                                        error!(
                                            "Failed to dispatch webhook {} to {}: {}",
                                            webhook.id, webhook.url, e
                                        );
                                    }
                                }
                                Err(e) => {
                                     error!("Failed to serialize webhook payload: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch webhooks for account {}: {}", account_id, e);
                }
            }
        }.instrument(span));
    }
}
