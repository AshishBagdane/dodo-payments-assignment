use utoipa::{Modify, OpenApi};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use crate::application::dto::account_dto::{AccountResponse, CreateAccountRequest};
use crate::application::dto::transaction_dto::{
    DepositRequest, TransactionResponse, TransferRequest, WithdrawRequest,
};
use crate::application::dto::webhook_dto::{CreateWebhookRequest, WebhookResponse};
use crate::domain::value_objects::webhook_event::WebhookEvent;
use crate::presentation::api::error::ErrorResponse;

use super::account;
use super::transaction;
use super::webhook;
use super::health;


#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        account::create_account,
        account::get_account,
        account::list_accounts,
        transaction::deposit,
        transaction::withdraw,
        transaction::transfer,
        transaction::get_history,
        webhook::create_webhook,
        webhook::list_webhooks,
        webhook::delete_webhook,
    ),
    components(
        schemas(
            CreateAccountRequest,
            AccountResponse,
            DepositRequest,
            WithdrawRequest,
            TransferRequest,
            TransactionResponse,
            CreateWebhookRequest,
            WebhookResponse,
            WebhookEvent,
            ErrorResponse
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "account", description = "Account management endpoints"),
        (name = "transaction", description = "Transaction management endpoints"),
        (name = "webhook", description = "Webhook management endpoints")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
            );
        }
    }
}
