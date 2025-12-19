use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::application::dto::CreateAccountRequest;
use crate::presentation::api::map_service_error;
use crate::application::AppState;

/// Create a new account
pub async fn create_account(
    State(state): State<AppState>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let account = state
        .account_service
        .create_account(payload)
        .await
        .map_err(map_service_error)?;

    Ok((StatusCode::CREATED, Json(account)))
}

/// Get account by ID
pub async fn get_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let account = state
        .account_service
        .get_account(id)
        .await
        .map_err(map_service_error)?;

    Ok((StatusCode::OK, Json(account)))
}

/// List all accounts
pub async fn list_accounts(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let accounts = state
        .account_service
        .list_accounts()
        .await
        .map_err(map_service_error)?;

    Ok((StatusCode::OK, Json(accounts)))
}
