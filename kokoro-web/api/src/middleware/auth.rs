//! Auth middleware — Axum extractor that validates Bearer tokens.
//!
//! Usage in handlers:
//! ```ignore
//! async fn my_handler(AuthUser(claims): AuthUser) -> impl IntoResponse { ... }
//! ```

use std::sync::Arc;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};

use crate::db::Database;
use crate::error::ApiError;
use crate::models::auth::Claims;
use crate::services::auth as auth_service;

/// Extractor that validates the Bearer token and provides the user's claims.
///
/// Rejects the request with 401 if the token is missing, malformed, or expired.
pub struct AuthUser(pub Claims);

impl FromRequestParts<Arc<Database>> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &Arc<Database>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::unauthorized("Missing Authorization header"))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::unauthorized("Invalid Authorization format"))?;

        let claims = auth_service::validate_token(token)
            .map_err(|_| ApiError::unauthorized("Invalid or expired token"))?;

        Ok(AuthUser(claims))
    }
}
