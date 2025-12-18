use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::application::dto::{DepositRequest, TransferRequest, WithdrawRequest};
use crate::application::AppState;
use crate::domain::errors::ServiceError;

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub account_id: Uuid,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_limit() -> i64 {
    10
}

fn default_offset() -> i64 {
    0
}

/// Deposit funds
pub async fn deposit(
    State(state): State<AppState>,
    Json(payload): Json<DepositRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let transaction = state
        .transaction_service
        .deposit(payload)
        .await
        .map_err(map_service_error)?;

    Ok((StatusCode::OK, Json(transaction)))
}

/// Withdraw funds
pub async fn withdraw(
    State(state): State<AppState>,
    Json(payload): Json<WithdrawRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let transaction = state
        .transaction_service
        .withdraw(payload)
        .await
        .map_err(map_service_error)?;

    Ok((StatusCode::OK, Json(transaction)))
}

/// Transfer funds
pub async fn transfer(
    State(state): State<AppState>,
    Json(payload): Json<TransferRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let transaction = state
        .transaction_service
        .transfer(payload)
        .await
        .map_err(map_service_error)?;

    Ok((StatusCode::OK, Json(transaction)))
}

/// Get transaction history
pub async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let history = state
        .transaction_service
        .get_history(params.account_id, params.limit, params.offset)
        .await
        .map_err(map_service_error)?;

    Ok((StatusCode::OK, Json(history)))
}

// Helper to map ServiceError to HTTP StatusCode
fn map_service_error(err: ServiceError) -> (StatusCode, String) {
    let api_error: crate::domain::errors::ApiError = err.into();
    match api_error {
        crate::domain::errors::ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        crate::domain::errors::ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        crate::domain::errors::ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
        crate::domain::errors::ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        crate::domain::errors::ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
        crate::domain::errors::ApiError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
        crate::domain::errors::ApiError::ServiceUnavailable(msg) => {
            (StatusCode::SERVICE_UNAVAILABLE, msg)
        }
        crate::domain::errors::ApiError::InternalServerError(msg) => {
            (StatusCode::INTERNAL_SERVER_ERROR, msg)
        }
    }
}
