use serde::{Deserialize, Serialize};

/// Registration request body.
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub display_name: String,
    pub password: String,
    /// Year of birth — for COPPA age gate. Must be 13+ to register.
    pub birth_year: Option<i32>,
    /// User consents to privacy policy (required).
    pub privacy_consent: Option<bool>,
}

/// Login request body.
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Successful auth response — contains the JWT token.
#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Public user profile (no password hash).
#[derive(Serialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub created_at: String,
}

/// Internal user record (includes password hash, never serialized to HTTP).
pub struct UserRecord {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub created_at: String,
}

/// JWT claims — encoded inside every token.
#[derive(Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: String,
    /// User email.
    pub email: String,
    /// Expiration timestamp (Unix seconds).
    pub exp: i64,
    /// Issued-at timestamp (Unix seconds).
    pub iat: i64,
}
