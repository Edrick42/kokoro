//! Creature models — sync payloads and internal records.

use serde::{Deserialize, Serialize};

/// Internal DB record for a creature.
pub struct CreatureRecord {
    pub id: String,
    pub user_id: String,
    pub species: String,
    pub genome_json: String,
    pub mind_json: String,
    pub age_ticks: i64,
    pub alive: bool,
    pub synced_at: String,
}

/// Request body for syncing creature state from the game.
#[derive(Deserialize)]
pub struct SyncRequest {
    pub species: String,
    pub genome: serde_json::Value,
    pub mind: serde_json::Value,
    pub age_ticks: i64,
    pub alive: bool,
}

/// Response for creature download.
#[derive(Serialize)]
pub struct CreatureResponse {
    pub id: String,
    pub species: String,
    pub genome: serde_json::Value,
    pub mind: serde_json::Value,
    pub age_ticks: i64,
    pub alive: bool,
    pub synced_at: String,
}

/// Response after a successful sync.
#[derive(Serialize)]
pub struct SyncResponse {
    pub status: String,
    pub creature_id: String,
    pub synced_at: String,
}
