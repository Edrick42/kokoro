//! Creature service — business logic for creature sync.
//!
//! No HTTP types here — pure Rust types only.

use chrono::Utc;
use uuid::Uuid;

use crate::db::{self, Database};
use crate::models::creature::*;

/// Syncs creature state from the game. Creates or updates the record.
///
/// Returns the creature ID and sync timestamp.
pub fn sync_creature(
    db: &Database,
    user_id: &str,
    req: &SyncRequest,
) -> Result<(String, String), String> {
    let now = Utc::now().to_rfc3339();

    // Use existing creature ID if one exists, otherwise generate new
    let creature_id = db::creatures::find_by_user(db, user_id)
        .map(|c| c.id)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let record = CreatureRecord {
        id: creature_id.clone(),
        user_id: user_id.to_string(),
        species: req.species.clone(),
        genome_json: req.genome.to_string(),
        mind_json: req.mind.to_string(),
        age_ticks: req.age_ticks,
        alive: req.alive,
        synced_at: now.clone(),
    };

    db::creatures::upsert(db, &record)?;

    Ok((creature_id, now))
}

/// Fetches the user's creature and converts it to a response.
pub fn get_creature(
    db: &Database,
    user_id: &str,
) -> Option<CreatureResponse> {
    let record = db::creatures::find_by_user(db, user_id)?;

    let genome: serde_json::Value = serde_json::from_str(&record.genome_json).ok()?;
    let mind: serde_json::Value = serde_json::from_str(&record.mind_json).ok()?;

    Some(CreatureResponse {
        id: record.id,
        species: record.species,
        genome,
        mind,
        age_ticks: record.age_ticks,
        alive: record.alive,
        synced_at: record.synced_at,
    })
}
