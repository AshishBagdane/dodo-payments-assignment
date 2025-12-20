use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Extension,
};
use uuid::Uuid;

use crate::application::dto::{CreateWebhookRequest, WebhookResponse};
use crate::application::AppState;
use crate::domain::entities::Webhook;
use crate::domain::repositories::WebhookRepository;
use crate::domain::errors::{ApiError, ServiceError};
use crate::application::services::AuthPrincipal;

pub async fn create_webhook(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthPrincipal>,
    Json(payload): Json<CreateWebhookRequest>,
) ->  Result<impl IntoResponse, ApiError> {
    let webhook = Webhook::new(_auth.account_id, payload.url, payload.event)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let created_webhook = state
        .webhook_repository
        .create(webhook)
        .await
        .map_err(ServiceError::from)
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(WebhookResponse::from(created_webhook))))
}

pub async fn list_webhooks(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthPrincipal>,
) -> Result<impl IntoResponse, ApiError> {
    let webhooks = state
        .webhook_repository
        .list_by_account(_auth.account_id)
        .await
        .map_err(ServiceError::from)
        .map_err(ApiError::from)?;

    let response: Vec<WebhookResponse> = webhooks.into_iter().map(WebhookResponse::from).collect();

    Ok((StatusCode::OK, Json(response)))
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthPrincipal>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .webhook_repository
        .delete(id)
        .await
        .map_err(ServiceError::from)
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
