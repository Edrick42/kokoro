//! Auth API calls — login and register via the Kokoro API.
//!
//! Uses `reqwest::blocking` because auth is a one-shot action
//! triggered by button press, not a per-frame operation.

use super::{API_BASE, SessionData};

/// Login response from the API.
#[derive(serde::Deserialize)]
struct TokenResponse {
    token: String,
}

/// User profile from the API.
#[derive(serde::Deserialize)]
struct ProfileResponse {
    id: String,
    email: String,
    display_name: String,
}

/// Attempts to login with email and password.
/// Returns session data on success, error message on failure.
pub fn login(email: &str, password: &str) -> Result<SessionData, String> {
    let client = reqwest::blocking::Client::new();

    let resp = client
        .post(format!("{API_BASE}/auth/login"))
        .json(&serde_json::json!({
            "email": email,
            "password": password,
        }))
        .send()
        .map_err(|e| format!("API unreachable: {e}"))?;

    if !resp.status().is_success() {
        let body: serde_json::Value = resp.json().unwrap_or_default();
        let msg = body["error"].as_str().unwrap_or("Login failed");
        return Err(msg.to_string());
    }

    let token_resp: TokenResponse = resp.json()
        .map_err(|e| format!("Invalid response: {e}"))?;

    fetch_profile(&client, &token_resp.token)
}

/// Attempts to register a new account.
/// Returns session data on success (auto-login after register).
pub fn register(display_name: &str, email: &str, password: &str) -> Result<SessionData, String> {
    let client = reqwest::blocking::Client::new();

    let resp = client
        .post(format!("{API_BASE}/auth/register"))
        .json(&serde_json::json!({
            "display_name": display_name,
            "email": email,
            "password": password,
        }))
        .send()
        .map_err(|e| format!("API unreachable: {e}"))?;

    if !resp.status().is_success() {
        let body: serde_json::Value = resp.json().unwrap_or_default();
        let msg = body["error"].as_str().unwrap_or("Registration failed");
        return Err(msg.to_string());
    }

    let token_resp: TokenResponse = resp.json()
        .map_err(|e| format!("Invalid response: {e}"))?;

    fetch_profile(&client, &token_resp.token)
}

/// Fetches the user profile using the token.
fn fetch_profile(client: &reqwest::blocking::Client, token: &str) -> Result<SessionData, String> {
    let resp = client
        .get(format!("{API_BASE}/auth/profile"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .map_err(|e| format!("Profile fetch failed: {e}"))?;

    let profile: ProfileResponse = resp.json()
        .map_err(|e| format!("Invalid profile: {e}"))?;

    Ok(SessionData {
        token: token.to_string(),
        user_id: profile.id,
        email: profile.email,
        display_name: profile.display_name,
    })
}
