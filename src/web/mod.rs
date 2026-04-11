//! Web API client — connects the Bevy game to the Kokoro API.
//!
//! Provides login, register, and creature sync functionality.
//! Uses `reqwest::blocking` for auth (one-shot) and Bevy's
//! `IoTaskPool` for async creature sync during gameplay.

pub mod auth;
pub mod sync;

use bevy::prelude::*;

/// API base URL. In production, use an environment variable.
pub const API_BASE: &str = "http://localhost:8080";

/// Active web session — `None` for guest mode.
#[derive(Resource, Default, Clone)]
pub struct WebSession {
    pub active: Option<SessionData>,
}

/// Authenticated session data.
#[derive(Clone, Debug)]
pub struct SessionData {
    pub token: String,
    #[allow(dead_code)]
    pub user_id: String,
    pub email: String,
    pub display_name: String,
}

/// Wallet balance in etharin crystals (synced from API).
#[derive(Resource, Default)]
pub struct WalletBalance(#[allow(dead_code)] pub u32);

pub struct WebPlugin;

impl Plugin for WebPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WebSession::default())
           .insert_resource(WalletBalance::default());
    }
}
