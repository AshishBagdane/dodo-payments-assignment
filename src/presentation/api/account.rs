use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::application::dto::CreateAccountRequest;
use crate::application::AppState;
use crate::domain::errors::ApiError;

/// Create a new account
pub async fn create_account(
    State(state): State<AppState>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let account = state
        .account_service
        .create_account(payload)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(account)))
}

/// Get account by ID
pub async fn get_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let account = state
        .account_service
        .get_account(id)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(account)))
}

/// List all accounts
pub async fn list_accounts(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let accounts = state
        .account_service
        .list_accounts()
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(accounts)))
}
