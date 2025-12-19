use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::{DepositRequest, TransactionResponse, TransferRequest, WithdrawRequest};
use crate::application::services::WebhookService;
use crate::domain::entities::Transaction;
use crate::domain::errors::ServiceError;
use crate::domain::repositories::TransactionRepository;
use crate::domain::value_objects::{Money, WebhookEvent};

pub struct TransactionService {
    repository: Arc<dyn TransactionRepository>,
    webhook_service: Option<Arc<WebhookService>>,
}

impl TransactionService {
    pub fn new(
        repository: Arc<dyn TransactionRepository>,
        webhook_service: Option<Arc<WebhookService>>,
    ) -> Self {
        Self { 
            repository,
            webhook_service,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn deposit(
        &self,
        request: DepositRequest,
    ) -> Result<TransactionResponse, ServiceError> {
        let money = Money::new(request.amount).map_err(ServiceError::Domain)?;
        let transaction = Transaction::new_credit(
            request.account_id, 
            money, 
            request.idempotency_key.clone()
        ).map_err(ServiceError::Domain)?;

        let created_transaction = match self.repository.execute_credit(&transaction).await {
            Ok(tx) => tx,
            Err(e) => {
                if let crate::domain::errors::RepositoryError::DuplicateEntry(_) = e {
                    // Idempotency hit: return existing transaction
                    tracing::info!("Idempotency hit for key: {:?}", request.idempotency_key);
                    if let Some(key) = &request.idempotency_key {
                        return self.repository.find_by_idempotency_key(key)
                            .await
                            .map(TransactionResponse::from)
                            .map_err(ServiceError::from);
                    }
                }
                return Err(ServiceError::from(e));
            }
        };

        let response = TransactionResponse::from(created_transaction);
        
        if let Some(webhook_service) = &self.webhook_service {
             webhook_service.notify_async(
                request.account_id,
                WebhookEvent::TransactionCompleted, 
                response.clone(),
            );
        }

        Ok(response)
    }

    #[tracing::instrument(skip(self))]
    pub async fn withdraw(
        &self,
        request: WithdrawRequest,
    ) -> Result<TransactionResponse, ServiceError> {
        let money = Money::new(request.amount).map_err(ServiceError::Domain)?;
        let transaction = Transaction::new_debit(
            request.account_id, 
            money, 
            request.idempotency_key.clone()
        ).map_err(ServiceError::Domain)?;

        let created_transaction = match self.repository.execute_debit(&transaction).await {
            Ok(tx) => tx,
            Err(e) => {
                if let crate::domain::errors::RepositoryError::DuplicateEntry(_) = e {
                    // Idempotency hit
                    tracing::info!("Idempotency hit for key: {:?}", request.idempotency_key);
                    if let Some(key) = &request.idempotency_key {
                        return self.repository.find_by_idempotency_key(key)
                            .await
                            .map(TransactionResponse::from)
                            .map_err(ServiceError::from);
                    }
                }
                return Err(ServiceError::from(e));
            }
        };

        let response = TransactionResponse::from(created_transaction);
        
        if let Some(webhook_service) = &self.webhook_service {
             webhook_service.notify_async(
                request.account_id,
                WebhookEvent::TransactionCompleted, 
                response.clone(),
            );
        }

        Ok(response)
    }

    #[tracing::instrument(skip(self))]
    pub async fn transfer(
        &self,
        request: TransferRequest,
    ) -> Result<TransactionResponse, ServiceError> {
        let money = Money::new(request.amount).map_err(ServiceError::Domain)?;
        let transaction = Transaction::new_transfer(
            request.from_account_id,
            request.to_account_id,
            money,
            request.idempotency_key.clone(),
        )
        .map_err(ServiceError::Domain)?;

        let created_transaction = match self.repository.execute_transfer(&transaction).await {
            Ok(tx) => tx,
            Err(e) => {
                if let crate::domain::errors::RepositoryError::DuplicateEntry(_) = e {
                    // Idempotency hit
                    tracing::info!("Idempotency hit for key: {:?}", request.idempotency_key);
                    if let Some(key) = &request.idempotency_key {
                        return self.repository.find_by_idempotency_key(key)
                            .await
                            .map(TransactionResponse::from)
                            .map_err(ServiceError::from);
                    }
                }
                return Err(ServiceError::from(e));
            }
        };

        let response = TransactionResponse::from(created_transaction);
        
        if let Some(webhook_service) = &self.webhook_service {
             webhook_service.notify_async(
                request.from_account_id,
                WebhookEvent::TransactionCompleted,
                response.clone(),
            );
        }

        Ok(response)
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
