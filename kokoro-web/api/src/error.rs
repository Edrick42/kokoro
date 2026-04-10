//! API error type — converts application errors into HTTP responses.
//!
//! Implements Axum's `IntoResponse` trait so any handler can return
//! `Result<Json<T>, ApiError>` and errors automatically become JSON
//! with the correct HTTP status code.

use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

/// Structured error response returned as JSON.
#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
    pub code: u16,
}

impl ApiError {
    /// 404 Not Found — resource doesn't exist.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self { error: msg.into(), code: 404 }
    }

    /// 500 Internal Server Error — something went wrong on our side.
    #[allow(dead_code)]
    pub fn internal(msg: impl Into<String>) -> Self {
        Self { error: msg.into(), code: 500 }
    }

    /// 401 Unauthorized — invalid or missing credentials.
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self { error: msg.into(), code: 401 }
    }

    /// 409 Conflict — resource already exists (e.g. duplicate email).
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self { error: msg.into(), code: 409 }
    }
}

/// Teaches Axum how to convert ApiError into an HTTP response.
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}
