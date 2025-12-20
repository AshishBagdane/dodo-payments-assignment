use thiserror::Error;

/// Domain-level errors that represent business rule violations
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Insufficient balance: available {available}, required {required}")]
    InsufficientBalance { available: String, required: String },

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid account state: {0}")]
    InvalidAccountState(String),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("Duplicate transaction: idempotency key {0} already exists")]
    DuplicateTransaction(String),

    #[error("Invalid transaction type: {0}")]
    InvalidTransactionType(String),

    #[error("Self transfer not allowed")]
    SelfTransferNotAllowed,

    #[error("API key not found")]
    ApiKeyNotFound,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Webhook not found: {0}")]
    WebhookNotFound(String),

    #[error("Invalid webhook URL: {0}")]
    InvalidWebhookUrl(String),

    #[error("Invalid webhook event: {0}")]
    InvalidWebhookEvent(String),
}

/// Repository-level errors for data access failures
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),
}

/// Service-level errors for application logic failures
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Repository error: {0}")]
    Repository(RepositoryError),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Webhook delivery error: {0}")]
    WebhookDeliveryError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}


/// API-level errors for HTTP responses
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

// Conversions from domain/service errors to API errors
impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::AccountNotFound(msg) => ApiError::NotFound(msg),
            DomainError::InsufficientBalance { .. } => ApiError::BadRequest(err.to_string()),
            DomainError::InvalidAmount(msg) => ApiError::BadRequest(msg),
            DomainError::InvalidAccountState(msg) => ApiError::BadRequest(msg),
            DomainError::TransactionNotFound(msg) => ApiError::NotFound(msg),
            DomainError::DuplicateTransaction(_) => ApiError::Conflict(err.to_string()),
            DomainError::InvalidTransactionType(msg) => ApiError::BadRequest(msg),
            DomainError::SelfTransferNotAllowed => ApiError::BadRequest(err.to_string()),
            DomainError::ApiKeyNotFound => ApiError::Unauthorized("Invalid API key".to_string()),
            DomainError::InvalidApiKey => ApiError::Unauthorized("Invalid API key".to_string()),
            DomainError::RateLimitExceeded(msg) => ApiError::TooManyRequests(msg),
            DomainError::WebhookNotFound(msg) => ApiError::NotFound(msg),
            DomainError::InvalidWebhookUrl(msg) => ApiError::BadRequest(msg),
            DomainError::InvalidWebhookEvent(msg) => ApiError::BadRequest(msg),
        }
    }
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::Domain(e) => e.into(),
            ServiceError::Repository(e) => match e {
                RepositoryError::NotFound(msg) => ApiError::NotFound(msg),
                RepositoryError::DuplicateEntry(msg) => ApiError::Conflict(msg),
                RepositoryError::ConstraintViolation(msg) => ApiError::BadRequest(msg),
                _ => ApiError::InternalServerError("Database error".to_string()),
            },
            ServiceError::ValidationError(msg) => ApiError::BadRequest(msg),
            ServiceError::AuthorizationError(msg) => ApiError::Forbidden(msg),
            ServiceError::WebhookDeliveryError(_) => {
                ApiError::InternalServerError("Webhook delivery failed".to_string())
            }
            ServiceError::ConfigurationError(_) => {
                ApiError::InternalServerError("Service configuration error".to_string())
            }
            ServiceError::ExternalService(_) => {
                ApiError::ServiceUnavailable("External service unavailable".to_string())
            }
            ServiceError::InternalError(_) => {
                ApiError::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<RepositoryError> for ServiceError {
    fn from(err: RepositoryError) -> Self {
        ServiceError::Repository(err)
    }
}

// SQLx error conversions
impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => RepositoryError::NotFound("Record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    // PostgreSQL error codes
                    match code.as_ref() {
                        "23505" => {
                            // Unique violation
                            RepositoryError::DuplicateEntry(
                                db_err.message().to_string(),
                            )
                        }
                        "23503" => {
                            // Foreign key violation
                            RepositoryError::ConstraintViolation(
                                db_err.message().to_string(),
                            )
                        }
                        "23514" => {
                            // Check constraint violation
                            RepositoryError::ConstraintViolation(
                                db_err.message().to_string(),
                            )
                        }
                        _ => RepositoryError::DatabaseError(db_err.message().to_string()),
                    }
                } else {
                    RepositoryError::DatabaseError(db_err.message().to_string())
                }
            }
            sqlx::Error::PoolTimedOut => {
                RepositoryError::ConnectionError("Connection pool timeout".to_string())
            }
            sqlx::Error::PoolClosed => {
                RepositoryError::ConnectionError("Connection pool closed".to_string())
            }
            _ => RepositoryError::QueryError(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_display() {
        let err = DomainError::InsufficientBalance {
            available: "100.00".to_string(),
            required: "200.00".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Insufficient balance: available 100.00, required 200.00"
        );
    }

    #[test]
    fn test_domain_to_api_error_conversion() {
        let domain_err = DomainError::AccountNotFound("acc_123".to_string());
        let api_err: ApiError = domain_err.into();
        assert!(matches!(api_err, ApiError::NotFound(_)));
    }

    #[test]
    fn test_rate_limit_error_conversion() {
        let domain_err = DomainError::RateLimitExceeded("Too many requests".to_string());
        let api_err: ApiError = domain_err.into();
        assert!(matches!(api_err, ApiError::TooManyRequests(_)));
    }

    #[test]
    fn test_service_error_from_domain() {
        let domain_err = DomainError::InvalidAmount("Negative amount".to_string());
        let service_err: ServiceError = domain_err.into();
        assert!(matches!(service_err, ServiceError::Domain(_)));
    }

    #[test]
    fn test_api_error_from_service() {
        let repo_err = RepositoryError::NotFound("Transaction not found".to_string());
        let service_err = ServiceError::Repository(repo_err);
        let api_err: ApiError = service_err.into();
        assert!(matches!(api_err, ApiError::NotFound(_)));
    }
}