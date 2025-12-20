use std::sync::Arc;
use uuid::Uuid;
use rust_decimal::dec;

use crate::application::dto::{AccountResponse, CreateAccountRequest};
use crate::domain::entities::Account;
use crate::domain::errors::ServiceError;
use crate::domain::repositories::AccountRepository;
use crate::domain::value_objects::Money;

pub struct AccountService {
    repository: Arc<dyn AccountRepository>,
}

impl AccountService {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_account(
        &self,
        request: CreateAccountRequest,
    ) -> Result<AccountResponse, ServiceError> {
        let account = Account::new(request.business_name, Money::new(dec!(0.00))?)
            .map_err(ServiceError::from)?;
        let created_account = self
            .repository
            .create(&account)
            .await
            .map_err(ServiceError::from)?;
        Ok(AccountResponse::from(created_account))
    }

    pub async fn get_account(&self, id: Uuid) -> Result<AccountResponse, ServiceError> {
        let account = self
            .repository
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?;
        Ok(AccountResponse::from(account))
    }

    pub async fn list_accounts(&self) -> Result<Vec<AccountResponse>, ServiceError> {
        // Default pagination for now
        let accounts = self
            .repository
            .list(100, 0)
            .await
            .map_err(ServiceError::from)?;
        Ok(accounts.into_iter().map(AccountResponse::from).collect())
    }

    pub async fn health_check(&self) -> Result<(), ServiceError> {
        self.repository.health_check().await.map_err(ServiceError::from)
    }
}
