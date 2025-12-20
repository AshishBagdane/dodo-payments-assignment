use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::application::dto::{CreateAccountRequest, AccountResponse};
use crate::application::AppState;
use crate::domain::errors::ApiError;
use crate::presentation::api::error::ErrorResponse;

/// Create a new account
#[utoipa::path(
    post,
    path = "/accounts",
    request_body = CreateAccountRequest,
    responses(
        (status = 201, description = "Account created successfully", body = AccountResponse),
        (status = 400, description = "Bad request", body = ErrorResponse)
    )
)]
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
#[utoipa::path(
    get,
    path = "/accounts/{id}",
    params(
        ("id" = Uuid, Path, description = "Account ID")
    ),
    responses(
        (status = 200, description = "Account details", body = AccountResponse),
        (status = 404, description = "Account not found", body = ErrorResponse)
    )
)]
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
#[utoipa::path(
    get,
    path = "/accounts",
    responses(
        (status = 200, description = "List of accounts", body = [AccountResponse])
    )
)]
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
