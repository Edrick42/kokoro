//! Creature sync — uploads creature state to the Kokoro API.
//!
//! Runs asynchronously using Bevy's IoTaskPool to avoid blocking
//! the main game thread during autosave.

use bevy::prelude::*;

use super::{API_BASE, WebSession};
use crate::genome::Genome;
use crate::mind::Mind;

/// Syncs creature state to the API (non-blocking).
/// Called from the autosave system when a web session is active.
pub fn sync_creature_async(session: &WebSession, genome: &Genome, mind: &Mind) {
    let Some(ref data) = session.active else { return };

    let token = data.token.clone();
    let species = format!("{:?}", genome.species);
    let genome_json = serde_json::json!({
        "curiosity": genome.curiosity,
        "loneliness_sensitivity": genome.loneliness_sensitivity,
        "appetite": genome.appetite,
        "circadian": genome.circadian,
        "resilience": genome.resilience,
        "learning_rate": genome.learning_rate,
        "hue": genome.hue,
    });
    let mind_json = serde_json::json!({
        "hunger": mind.stats.hunger,
        "happiness": mind.stats.happiness,
        "energy": mind.stats.energy,
        "health": mind.stats.health,
        "mood": mind.mood.label(),
    });
    let age_ticks = mind.age_ticks as i64;

    // Spawn on IoTaskPool — doesn't block the game loop
    bevy::tasks::IoTaskPool::get().spawn(async move {
        let client = reqwest::Client::new();
        let result = client
            .post(format!("{API_BASE}/api/creature/sync"))
            .header("Authorization", format!("Bearer {token}"))
            .json(&serde_json::json!({
                "species": species,
                "genome": genome_json,
                "mind": mind_json,
                "age_ticks": age_ticks,
                "alive": true,
            }))
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                info!("Creature synced to web API");
            }
            Ok(resp) => {
                warn!("Creature sync failed: HTTP {}", resp.status());
            }
            Err(e) => {
                warn!("Creature sync failed: {e}");
            }
        }
    }).detach();
}
