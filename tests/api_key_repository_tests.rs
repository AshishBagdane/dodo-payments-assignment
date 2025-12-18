use dodo_payments_assignment::domain::entities::{Account, ApiKey};
use dodo_payments_assignment::domain::repositories::{AccountRepository, ApiKeyRepository};
use dodo_payments_assignment::domain::value_objects::Money;
use dodo_payments_assignment::infrastructure::config::Config;
use dodo_payments_assignment::infrastructure::database::{create_pool, PostgresAccountRepository, PostgresApiKeyRepository};
use rust_decimal_macros::dec;
use uuid::Uuid;

async fn setup_repo() -> PostgresApiKeyRepository {
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/dodo-payments");
    }
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.expect("Failed to create pool");
    PostgresApiKeyRepository::new(pool)
}

async fn create_test_account() -> Account {
    unsafe {
        std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/dodo-payments");
    }
    let config = Config::from_env().unwrap();
    let pool = create_pool(&config).await.expect("Failed to create pool");
    let repo = PostgresAccountRepository::new(pool);
    
    let account = Account::new("Test Corp for Keys".to_string(), Money::new(dec!(0)).unwrap()).unwrap();
    repo.create(&account).await.expect("Failed to create account")
}

#[tokio::test]
async fn test_create_and_retrieve_api_key() {
    let repo = setup_repo().await;
    let account = create_test_account().await;
    let key_hash = Uuid::new_v4().to_string(); // Use UUID as unique hash for test

    let api_key = ApiKey::new(account.id, key_hash.clone());
    let created = repo.create(&api_key).await.expect("Failed to create key");

    assert_eq!(created.key_hash, key_hash);
    assert_eq!(created.rate_limit_per_hour, 1000);

    let retrieved = repo.find_by_hash(&key_hash).await.expect("Failed to find key");
    assert_eq!(retrieved.id, created.id);
}

#[tokio::test]
async fn test_find_by_account() {
    let repo = setup_repo().await;
    let account = create_test_account().await;
    
    let key1 = ApiKey::new(account.id, Uuid::new_v4().to_string());
    repo.create(&key1).await.unwrap();

    let key2 = ApiKey::new(account.id, Uuid::new_v4().to_string());
    repo.create(&key2).await.unwrap();

    let keys = repo.find_by_account(account.id).await.expect("Failed to list keys");
    assert_eq!(keys.len(), 2);
}

#[tokio::test]
async fn test_update_last_used() {
    let repo = setup_repo().await;
    let account = create_test_account().await;
    let key = ApiKey::new(account.id, Uuid::new_v4().to_string());
    let created = repo.create(&key).await.unwrap();

    assert!(created.last_used_at.is_none());

    repo.update_last_used(created.id).await.expect("Failed to update last used");

    let updated = repo.find_by_hash(&key.key_hash).await.unwrap();
    assert!(updated.last_used_at.is_some());
}

#[tokio::test]
async fn test_delete_api_key() {
    let repo = setup_repo().await;
    let account = create_test_account().await;
    let key = ApiKey::new(account.id, Uuid::new_v4().to_string());
    let created = repo.create(&key).await.unwrap();

    repo.delete(created.id).await.expect("Failed to delete key");

    let result = repo.find_by_hash(&key.key_hash).await;
    assert!(result.is_err());
    
    let exists = repo.exists(&key.key_hash).await.unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn test_create_duplicate_hash_fails() {
    let repo = setup_repo().await;
    let account = create_test_account().await;
    let hash = Uuid::new_v4().to_string();

    let key1 = ApiKey::new(account.id, hash.clone());
    repo.create(&key1).await.unwrap();

    let key2 = ApiKey::new(account.id, hash);
    let result = repo.create(&key2).await;
    
    assert!(result.is_err());
    // Should verify it is DuplicateEntry error
}
