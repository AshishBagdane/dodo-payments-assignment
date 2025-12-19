use dodo_payments_assignment::domain::entities::Webhook;
use dodo_payments_assignment::domain::value_objects::WebhookEvent;
use uuid::Uuid;

#[test]
fn test_create_webhook_success() {
    let account_id = Uuid::new_v4();
    let url = "https://example.com/webhook".to_string();
    let event = WebhookEvent::TransactionCompleted;

    let webhook = Webhook::new(account_id, url.clone(), event.clone()).expect("Failed to create webhook");

    assert_eq!(webhook.account_id, account_id);
    assert_eq!(webhook.url, url);
    assert_eq!(webhook.event, event);
}

#[test]
fn test_create_webhook_empty_url() {
    let account_id = Uuid::new_v4();
    let url = "   ".to_string();
    let event = WebhookEvent::AccountCreated;

    let result = Webhook::new(account_id, url, event);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "URL cannot be empty");
}

#[test]
fn test_create_webhook_invalid_protocol() {
    let account_id = Uuid::new_v4();
    let url = "ftp://example.com".to_string();
    let event = WebhookEvent::AccountCreated;

    let result = Webhook::new(account_id, url, event);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "URL must start with http or https");
}

#[test]
fn test_serialize_event() {
    let event = WebhookEvent::TransactionCompleted;
    let json = serde_json::to_string(&event).unwrap();
    assert_eq!(json, "\"transaction.completed\"");
}
