use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::application::dto::{AccountResponse, CreateAccountRequest};
use crate::application::services::AccountService;
use crate::domain::errors::ServiceError;
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
        crate::domain::errors::ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
        crate::domain::errors::ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    }
}
