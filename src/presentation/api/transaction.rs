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
use crate::presentation::api::map_service_error;

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
        .map_err(|e| map_service_error(e.into()))?;

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
        .map_err(|e| map_service_error(e.into()))?;

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
        .map_err(|e| map_service_error(e.into()))?;

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
        .map_err(|e| map_service_error(e.into()))?;

    Ok((StatusCode::OK, Json(history)))
}

