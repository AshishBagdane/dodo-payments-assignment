use dodo_payments_assignment::domain::entities::Webhook;
use dodo_payments_assignment::domain::repositories::WebhookRepository;
use dodo_payments_assignment::domain::value_objects::WebhookEvent;
use dodo_payments_assignment::infrastructure::database::PostgresWebhookRepository;
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

async fn setup_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres")
}

#[tokio::test]
async fn test_create_and_list_webhooks() {
    let pool = setup_pool().await;
    let repo = PostgresWebhookRepository::new(pool.clone());

    // 1. Create a dummy account (direct SQL to avoid dependency on AccountRepo in this test)
    let account_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO accounts (id, business_name, balance, created_at, updated_at) VALUES ($1, $2, 0, NOW(), NOW())",
        account_id,
        "Webhook Test Business"
    )
    .execute(&pool)
    .await
    .expect("Failed to create test account");

    // 2. Create Webhook
    let event = WebhookEvent::TransactionCompleted;
    let webhook = Webhook::new(
        account_id,
        "https://example.com/callback".to_string(),
        event.clone(),
    )
    .unwrap();

    let created = repo.create(webhook.clone()).await.expect("Failed to create webhook");
    assert_eq!(created.id, webhook.id);
    assert_eq!(created.url, webhook.url);

    // 3. List
    let list = repo.list_by_account(account_id).await.expect("Failed to list webhooks");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, webhook.id);
    assert_eq!(list[0].event, event);

    // Clean up
    repo.delete(webhook.id).await.expect("Failed to delete webhook");
    sqlx::query!("DELETE FROM accounts WHERE id = $1", account_id)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_delete_webhook() {
    let pool = setup_pool().await;
    let repo = PostgresWebhookRepository::new(pool.clone());

    let account_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO accounts (id, business_name, balance, created_at, updated_at) VALUES ($1, $2, 0, NOW(), NOW())",
        account_id,
        "Webhook Delete Test"
    )
    .execute(&pool)
    .await
    .expect("Failed to create test account");

    let webhook = Webhook::new(
        account_id,
        "https://example.com/delete".to_string(),
        WebhookEvent::AccountCreated,
    )
    .unwrap();

    repo.create(webhook.clone()).await.unwrap();

    // Delete
    repo.delete(webhook.id).await.expect("Failed to delete");

    // Verify gone
    let list = repo.list_by_account(account_id).await.unwrap();
    assert_eq!(list.len(), 0);
    
    // Clean up account
    sqlx::query!("DELETE FROM accounts WHERE id = $1", account_id)
        .execute(&pool)
        .await
        .unwrap();
}
