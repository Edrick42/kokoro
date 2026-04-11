//! Auth server functions — call the Kokoro API from the Leptos server.
//!
//! These run server-side only (SSR feature). The client calls them
//! transparently via Leptos server function RPC.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The API base URL. In production, this would come from an env var.
#[cfg(feature = "ssr")]
const API_BASE: &str = "http://localhost:8080";

// ===================================================================
// Shared types (serialized between server and client)
// ===================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatureData {
    pub species: String,
    pub age_ticks: i64,
    pub alive: bool,
    pub synced_at: String,
    pub genome: serde_json::Value,
    pub mind: serde_json::Value,
}

// ===================================================================
// Server functions
// ===================================================================

#[server(Login, "/api")]
pub async fn login(email: String, password: String) -> Result<UserSession, ServerFnError> {
    let client = reqwest::Client::new();

    // Call the API login endpoint
    let resp = client
        .post(format!("{API_BASE}/auth/login"))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("API unreachable: {e}")))?;

    if !resp.status().is_success() {
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        let msg = body["error"].as_str().unwrap_or("Login failed");
        return Err(ServerFnError::new(msg.to_string()));
    }

    let token_resp: serde_json::Value = resp.json().await
        .map_err(|e| ServerFnError::new(format!("Invalid response: {e}")))?;

    let token = token_resp["token"].as_str()
        .ok_or_else(|| ServerFnError::new("No token in response"))?
        .to_string();

    // Fetch the user profile with the token
    let profile_resp = client
        .get(format!("{API_BASE}/auth/profile"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Profile fetch failed: {e}")))?;

    let profile: serde_json::Value = profile_resp.json().await
        .map_err(|e| ServerFnError::new(format!("Invalid profile: {e}")))?;

    Ok(UserSession {
        token,
        user_id: profile["id"].as_str().unwrap_or("").to_string(),
        email: profile["email"].as_str().unwrap_or("").to_string(),
        display_name: profile["display_name"].as_str().unwrap_or("").to_string(),
    })
}

#[server(Register, "/api")]
pub async fn register(
    display_name: String,
    email: String,
    password: String,
) -> Result<UserSession, ServerFnError> {
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{API_BASE}/auth/register"))
        .json(&serde_json::json!({
            "display_name": display_name,
            "email": email,
            "password": password,
        }))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("API unreachable: {e}")))?;

    if !resp.status().is_success() {
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        let msg = body["error"].as_str().unwrap_or("Registration failed");
        return Err(ServerFnError::new(msg.to_string()));
    }

    let token_resp: serde_json::Value = resp.json().await
        .map_err(|e| ServerFnError::new(format!("Invalid response: {e}")))?;

    let token = token_resp["token"].as_str()
        .ok_or_else(|| ServerFnError::new("No token in response"))?
        .to_string();

    // Fetch profile
    let profile_resp = client
        .get(format!("{API_BASE}/auth/profile"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Profile fetch failed: {e}")))?;

    let profile: serde_json::Value = profile_resp.json().await
        .map_err(|e| ServerFnError::new(format!("Invalid profile: {e}")))?;

    Ok(UserSession {
        token,
        user_id: profile["id"].as_str().unwrap_or("").to_string(),
        email: profile["email"].as_str().unwrap_or("").to_string(),
        display_name: profile["display_name"].as_str().unwrap_or("").to_string(),
    })
}

#[server(FetchCreature, "/api")]
pub async fn fetch_creature(token: String) -> Result<Option<CreatureData>, ServerFnError> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{API_BASE}/api/creature"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("API unreachable: {e}")))?;

    if resp.status().as_u16() == 404 {
        return Ok(None);
    }

    if !resp.status().is_success() {
        return Err(ServerFnError::new("Failed to fetch creature"));
    }

    let data: serde_json::Value = resp.json().await
        .map_err(|e| ServerFnError::new(format!("Invalid creature data: {e}")))?;

    Ok(Some(CreatureData {
        species: data["species"].as_str().unwrap_or("Unknown").to_string(),
        age_ticks: data["age_ticks"].as_i64().unwrap_or(0),
        alive: data["alive"].as_bool().unwrap_or(true),
        synced_at: data["synced_at"].as_str().unwrap_or("Never").to_string(),
        genome: data["genome"].clone(),
        mind: data["mind"].clone(),
    }))
}
