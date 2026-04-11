//! Auth controller — HTTP handlers for registration, login, and profile.
//!
//! Each handler follows the pattern:
//!   1. Extract and validate input
//!   2. Call service/repository functions
//!   3. Return JSON response or ApiError

use std::sync::Arc;
use axum::{extract::State, Json};
use chrono::Utc;
use uuid::Uuid;

use crate::constants::auth::TOKEN_EXPIRY_SECS;
use crate::db::{self, Database};
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;
use crate::models::auth::*;
use crate::services::auth as auth_service;

/// POST /auth/register
///
/// Creates a new user account. Hashes the password with Argon2,
/// generates a UUID, and returns a JWT token on success.
pub async fn register(
    State(db): State<Arc<Database>>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    // Check if email already taken
    if db::users::find_by_email(&db, &body.email).is_some() {
        return Err(ApiError::conflict("Email already registered"));
    }

    // Hash password (never store plaintext)
    let password_hash = auth_service::hash_password(&body.password)
        .map_err(|e| ApiError::internal(e))?;

    // Create user record
    let user = UserRecord {
        id: Uuid::new_v4().to_string(),
        email: body.email.clone(),
        display_name: body.display_name,
        password_hash,
        created_at: Utc::now().to_rfc3339(),
    };

    db::users::create_user(&db, &user)
        .map_err(|e| ApiError::internal(e))?;

    // Generate token for immediate login
    let token = auth_service::create_token(&user.id, &user.email)
        .map_err(|e| ApiError::internal(e))?;

    Ok(Json(TokenResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: TOKEN_EXPIRY_SECS,
    }))
}

/// POST /auth/login
///
/// Authenticates with email + password. Returns a JWT token on success.
/// Uses a generic error message to avoid leaking whether the email exists.
pub async fn login(
    State(db): State<Arc<Database>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    // Find user — generic error message prevents email enumeration
    let user = db::users::find_by_email(&db, &body.email)
        .ok_or_else(|| ApiError::unauthorized("Invalid email or password"))?;

    // Verify password (constant-time comparison)
    if !auth_service::verify_password(&body.password, &user.password_hash) {
        return Err(ApiError::unauthorized("Invalid email or password"));
    }

    let token = auth_service::create_token(&user.id, &user.email)
        .map_err(|e| ApiError::internal(e))?;

    Ok(Json(TokenResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: TOKEN_EXPIRY_SECS,
    }))
}

/// GET /auth/profile
///
/// Returns the authenticated user's profile. Requires a valid
/// Bearer token in the Authorization header.
pub async fn profile(
    AuthUser(claims): AuthUser,
    State(db): State<Arc<Database>>,
) -> Result<Json<UserProfile>, ApiError> {
    let user = db::users::find_by_id(&db, &claims.sub)
        .ok_or_else(|| ApiError::not_found("User not found"))?;

    Ok(Json(UserProfile {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        created_at: user.created_at,
    }))
}
