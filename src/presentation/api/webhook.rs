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
use crate::presentation::api::error::ErrorResponse;

#[utoipa::path(
    post,
    path = "/webhooks",
    request_body = CreateWebhookRequest,
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 201, description = "Webhook created", body = WebhookResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
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

#[utoipa::path(
    get,
    path = "/webhooks",
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, description = "List of webhooks", body = [WebhookResponse]),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
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

#[utoipa::path(
    delete,
    path = "/webhooks/{id}",
    params(
        ("id" = Uuid, Path, description = "Webhook ID")
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 204, description = "Webhook deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
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
