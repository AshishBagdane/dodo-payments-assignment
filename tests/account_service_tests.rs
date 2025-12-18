use dodo_payments_assignment::application::dto::CreateAccountRequest;
use dodo_payments_assignment::application::services::AccountService;
use dodo_payments_assignment::domain::entities::Account;
use dodo_payments_assignment::domain::errors::RepositoryError;
use dodo_payments_assignment::domain::repositories::AccountRepository;
use dodo_payments_assignment::domain::value_objects::Money;
use async_trait::async_trait;
use rust_decimal_macros::dec;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// Mock Repository
struct MockAccountRepository {
    accounts: Mutex<Vec<Account>>,
}

impl MockAccountRepository {
    fn new() -> Self {
        Self {
            accounts: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl AccountRepository for MockAccountRepository {
    async fn create(&self, account: &Account) -> Result<Account, RepositoryError> {
        let mut accounts = self.accounts.lock().unwrap();
        accounts.push(account.clone());
        Ok(account.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Account, RepositoryError> {
        let accounts = self.accounts.lock().unwrap();
        accounts
            .iter()
            .find(|a| a.id == id)
            .cloned()
            .ok_or(RepositoryError::NotFound("Account not found".to_string()))
    }

    async fn update_balance(&self, _id: Uuid, _new_balance: Money) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn update_business_name(&self, _id: Uuid, _name: String) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let accounts = self.accounts.lock().unwrap();
        Ok(accounts.iter().any(|a| a.id == id))
    }

    async fn list(&self, _limit: i64, _offset: i64) -> Result<Vec<Account>, RepositoryError> {
        let accounts = self.accounts.lock().unwrap();
        Ok(accounts.clone())
    }

    async fn delete(&self, _id: Uuid) -> Result<(), RepositoryError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_create_account() {
    let mock_repo = Arc::new(MockAccountRepository::new());
    let service = AccountService::new(mock_repo);

    let request = CreateAccountRequest {
        name: "Test Corp".to_string(),
    };

    let response = service.create_account(request).await.expect("Failed to create account");

    assert_eq!(response.business_name, "Test Corp");
    assert_eq!(response.balance, dec!(0.00));
}

#[tokio::test]
async fn test_get_account() {
    let mock_repo = Arc::new(MockAccountRepository::new());
    let service = AccountService::new(mock_repo.clone());

    let request = CreateAccountRequest {
        name: "Test Corp".to_string(),
    };
    let created = service.create_account(request).await.expect("Failed to create account");

    let fetched = service.get_account(created.id).await.expect("Failed to get account");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.business_name, "Test Corp");
}

#[tokio::test]
async fn test_get_account_not_found() {
    let mock_repo = Arc::new(MockAccountRepository::new());
    let service = AccountService::new(mock_repo);

    let result = service.get_account(Uuid::new_v4()).await;
    assert!(result.is_err());
}
