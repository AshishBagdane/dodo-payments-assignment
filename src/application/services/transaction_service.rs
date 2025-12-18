use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{DepositRequest, TransactionResponse, TransferRequest, WithdrawRequest};
use crate::domain::entities::Transaction;
use crate::domain::errors::ServiceError;
use crate::domain::repositories::TransactionRepository;
use crate::domain::value_objects::Money;

pub struct TransactionService {
    repository: Arc<dyn TransactionRepository>,
}

impl TransactionService {
    pub fn new(repository: Arc<dyn TransactionRepository>) -> Self {
        Self { repository }
    }

    pub async fn deposit(
        &self,
        request: DepositRequest,
    ) -> Result<TransactionResponse, ServiceError> {
        let money = Money::new(request.amount).map_err(ServiceError::Domain)?;
        let transaction = Transaction::new_credit(request.account_id, money, None)
            .map_err(ServiceError::Domain)?;

        let created_transaction = self
            .repository
            .execute_credit(&transaction)
            .await
            .map_err(ServiceError::from)?;

        Ok(TransactionResponse::from(created_transaction))
    }

    pub async fn withdraw(
        &self,
        request: WithdrawRequest,
    ) -> Result<TransactionResponse, ServiceError> {
        let money = Money::new(request.amount).map_err(ServiceError::Domain)?;
        let transaction = Transaction::new_debit(request.account_id, money, None)
            .map_err(ServiceError::Domain)?;

        let created_transaction = self
            .repository
            .execute_debit(&transaction)
            .await
            .map_err(ServiceError::from)?;

        Ok(TransactionResponse::from(created_transaction))
    }

    pub async fn transfer(
        &self,
        request: TransferRequest,
    ) -> Result<TransactionResponse, ServiceError> {
        let money = Money::new(request.amount).map_err(ServiceError::Domain)?;
        let transaction = Transaction::new_transfer(
            request.from_account_id,
            request.to_account_id,
            money,
            None,
        )
        .map_err(ServiceError::Domain)?;

        let created_transaction = self
            .repository
            .execute_transfer(&transaction)
            .await
            .map_err(ServiceError::from)?;

        Ok(TransactionResponse::from(created_transaction))
    }

    pub async fn get_history(
        &self,
        account_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TransactionResponse>, ServiceError> {
        let transactions = self
            .repository
            .list_by_account(account_id, limit, offset)
            .await
            .map_err(ServiceError::from)?;

        Ok(transactions
            .into_iter()
            .map(TransactionResponse::from)
            .collect())
    }
}
