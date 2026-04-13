//! Auth controller — HTTP handlers for registration, login, and profile.
//!
//! Each handler follows the pattern:
//!   1. Extract and validate input
//!   2. Call service/repository functions
//!   3. Return JSON response or ApiError

use std::sync::Arc;
use axum::{extract::State, Json};
use chrono::{Utc, Datelike};
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
    // Age gate — COPPA compliance (must be 13+)
    if let Some(birth_year) = body.birth_year {
        let current_year = Utc::now().year();
        let age = current_year - birth_year;
        if age < 13 {
            return Err(ApiError::forbidden("You must be at least 13 years old to create an account."));
        }
    }

    // Privacy consent required
    if body.privacy_consent != Some(true) {
        return Err(ApiError::forbidden("You must accept the privacy policy to create an account."));
    }

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

/// GET /api/user/data-export — LGPD/GDPR: export all user data as JSON.
pub async fn data_export(
    AuthUser(claims): AuthUser,
    State(db): State<Arc<Database>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = db::users::find_by_id(&db, &claims.sub)
        .ok_or_else(|| ApiError::not_found("User not found"))?;

    let creature = db::creatures::find_by_user(&db, &claims.sub);
    let balance = db::shop::get_balance(&db, &claims.sub).unwrap_or(0);

    Ok(Json(serde_json::json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "display_name": user.display_name,
            "created_at": user.created_at,
        },
        "creature": creature.map(|c| serde_json::json!({
            "species": c.species,
            "genome": c.genome_json,
            "mind": c.mind_json,
            "age_ticks": c.age_ticks,
        })),
        "wallet": { "crystals": balance },
        "exported_at": Utc::now().to_rfc3339(),
    })))
}

/// DELETE /api/user/account — LGPD/GDPR: permanently delete all user data.
pub async fn delete_account(
    AuthUser(claims): AuthUser,
    State(db): State<Arc<Database>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = db.conn.lock().map_err(|e| ApiError::internal(e.to_string()))?;

    // Delete all user data in order (foreign key safe)
    conn.execute("DELETE FROM purchases WHERE user_id = ?1", [&claims.sub])
        .map_err(|e| ApiError::internal(e.to_string()))?;
    conn.execute("DELETE FROM wallets WHERE user_id = ?1", [&claims.sub])
        .map_err(|e| ApiError::internal(e.to_string()))?;
    conn.execute("DELETE FROM creatures WHERE user_id = ?1", [&claims.sub])
        .map_err(|e| ApiError::internal(e.to_string()))?;
    conn.execute("DELETE FROM users WHERE id = ?1", [&claims.sub])
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "deleted": true,
        "message": "All data permanently deleted. This cannot be undone."
    })))
}
