pub mod account;
pub mod health;
pub mod transaction;
pub mod webhook;

use axum::http::StatusCode;
use crate::domain::errors::ServiceError;

pub fn map_service_error(err: ServiceError) -> (StatusCode, String) {
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
