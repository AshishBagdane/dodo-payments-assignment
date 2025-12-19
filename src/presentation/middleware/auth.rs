use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum::http::HeaderMap;

use crate::application::AppState;

pub async fn require_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let api_key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing x-api-key header".to_string()))?;

    let principal = state
        .auth_service
        .verify_api_key(api_key)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid API key".to_string()))?;

    request.extensions_mut().insert(principal);

    Ok(next.run(request).await)
}
