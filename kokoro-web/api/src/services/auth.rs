//! Auth service — password hashing and JWT token management.
//!
//! This layer contains no HTTP logic — it works with plain Rust types.
//! Controllers call these functions; they never touch the database directly.

use argon2::{
    password_hash::{SaltString, PasswordHasher, PasswordVerifier},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

use crate::constants::auth::{JWT_SECRET, TOKEN_EXPIRY_SECS};
use crate::models::auth::Claims;

/// Hashes a plaintext password using Argon2id.
///
/// Returns the PHC-formatted hash string (contains salt + parameters).
/// Never store the plaintext password — only this hash.
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut rand_core::OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash password: {e}"))?;
    Ok(hash.to_string())
}

/// Verifies a plaintext password against a stored Argon2 hash.
///
/// Returns true if the password matches. Constant-time comparison
/// prevents timing attacks.
pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed = match argon2::PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// Creates a signed JWT token for the given user.
///
/// The token contains the user ID and email as claims, and expires
/// after TOKEN_EXPIRY_SECS (24 hours by default).
pub fn create_token(user_id: &str, email: &str) -> Result<String, String> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        iat: now,
        exp: now + TOKEN_EXPIRY_SECS,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .map_err(|e| format!("Failed to create token: {e}"))
}

/// Validates a JWT token and returns the decoded claims.
///
/// Checks signature and expiration. Returns Err if the token
/// is invalid, expired, or tampered with.
pub fn validate_token(token: &str) -> Result<Claims, String> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| format!("Invalid token: {e}"))?;
    Ok(data.claims)
}
