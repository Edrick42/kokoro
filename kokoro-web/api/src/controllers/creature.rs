//! Creature controller — HTTP handlers for creature sync.
//!
//! All endpoints require authentication via the AuthUser extractor.

use std::sync::Arc;
use axum::{extract::State, Json};

use crate::db::Database;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;
use crate::models::creature::*;
use crate::services::creature as creature_service;

/// POST /api/creature/sync
///
/// Uploads the current creature state from the game.
/// Creates or updates the user's creature record.
pub async fn sync(
    AuthUser(claims): AuthUser,
    State(db): State<Arc<Database>>,
    Json(body): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, ApiError> {
    let (creature_id, synced_at) = creature_service::sync_creature(&db, &claims.sub, &body)
        .map_err(|e| ApiError::internal(e))?;

    Ok(Json(SyncResponse {
        status: "synced".to_string(),
        creature_id,
        synced_at,
    }))
}

/// GET /api/creature
///
/// Downloads the user's latest creature state.
/// Returns 404 if no creature has been synced yet.
pub async fn get(
    AuthUser(claims): AuthUser,
    State(db): State<Arc<Database>>,
) -> Result<Json<CreatureResponse>, ApiError> {
    creature_service::get_creature(&db, &claims.sub)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("No creature found — play the game first!"))
}
