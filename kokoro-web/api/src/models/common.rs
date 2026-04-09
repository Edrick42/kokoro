//! Common response types shared across multiple controllers.

use serde::Serialize;

/// Response for the health check endpoint.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
