//! Health check controller — verifies the API is running.

use axum::Json;
use crate::models::common::HealthResponse;

/// GET /health — returns server status and version.
pub async fn check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}
