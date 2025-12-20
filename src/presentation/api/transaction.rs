use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::application::dto::{DepositRequest, TransferRequest, WithdrawRequest, TransactionResponse};
use crate::application::AppState;
use crate::domain::errors::ApiError;
use crate::presentation::api::error::ErrorResponse;

use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
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
#[utoipa::path(
    post,
    path = "/transactions/deposit",
    request_body = DepositRequest,
    responses(
        (status = 200, description = "Deposit successful", body = TransactionResponse),
        (status = 400, description = "Bad request", body = ErrorResponse)
    )
)]
pub async fn deposit(
    State(state): State<AppState>,
    Json(payload): Json<DepositRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let transaction = state
        .transaction_service
        .deposit(payload)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(transaction)))
}

/// Withdraw funds
#[utoipa::path(
    post,
    path = "/transactions/withdraw",
    request_body = WithdrawRequest,
    responses(
        (status = 200, description = "Withdraw successful", body = TransactionResponse),
        (status = 400, description = "Insufficient funds or bad request", body = ErrorResponse)
    )
)]
pub async fn withdraw(
    State(state): State<AppState>,
    Json(payload): Json<WithdrawRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let transaction = state
        .transaction_service
        .withdraw(payload)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(transaction)))
}

/// Transfer funds
#[utoipa::path(
    post,
    path = "/transactions/transfer",
    request_body = TransferRequest,
    responses(
        (status = 200, description = "Transfer successful", body = TransactionResponse),
        (status = 400, description = "Insufficient funds or bad request", body = ErrorResponse)
    )
)]
pub async fn transfer(
    State(state): State<AppState>,
    Json(payload): Json<TransferRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let transaction = state
        .transaction_service
        .transfer(payload)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(transaction)))
}

/// Get transaction history
#[utoipa::path(
    get,
    path = "/transactions/history",
    params(
        HistoryQuery
    ),
    responses(
        (status = 200, description = "Transaction history", body = [TransactionResponse]),
        (status = 400, description = "Bad request", body = ErrorResponse)
    )
)]
pub async fn get_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let history = state
        .transaction_service
        .get_history(params.account_id, params.limit, params.offset)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(history)))
}

