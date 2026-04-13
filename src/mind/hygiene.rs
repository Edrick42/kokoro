//! Hygiene system — creatures get dirty over time and need cleaning.
//!
//! Species-specific cleaning actions:
//! - Moluun: Groom (lick fur) — slow but thorough
//! - Pylum: Preen (arrange feathers) — medium
//! - Skael: Shed (shed old scales) — fast burst
//! - Nyxal: Ink Clean (expel ink cloud) — instant but costs energy

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::hygiene as cfg;
use crate::game::state::AppState;
use crate::genome::{Genome, Species};
use crate::mind::{Mind, MoodState};

/// Tracks the creature's hygiene level (0.0 = filthy, 100.0 = clean).
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct HygieneState {
    pub level: f32,
}

impl Default for HygieneState {
    fn default() -> Self {
        Self { level: 80.0 }
    }
}

/// Event fired when the player triggers a clean action.
#[derive(Event)]
pub struct CleanEvent;

pub struct HygienePlugin;

impl Plugin for HygienePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HygieneState::default())
            .add_event::<CleanEvent>()
            .add_systems(Update, (
                hygiene_decay_system,
                hygiene_effect_system,
                clean_action_system,
            ).chain().run_if(in_state(AppState::Gameplay)));
    }
}

/// Decays hygiene based on activity level.
fn hygiene_decay_system(
    mind: Res<Mind>,
    mut hygiene: ResMut<HygieneState>,
    ans: Option<Res<crate::mind::autonomic::AutonomicState>>,
) {
    let base_decay = match mind.mood {
        MoodState::Sleeping => cfg::SLEEP_DECAY,
        MoodState::Playful  => cfg::ACTIVE_DECAY,
        MoodState::Happy    => cfg::IDLE_DECAY * 1.5,
        _ => cfg::IDLE_DECAY,
    };

    // Sympathetic arousal = sweating, more dirt. Parasympathetic = slower decay.
    let arousal = ans.as_ref().map(|a| a.arousal_multiplier()).unwrap_or(1.0);
    let decay = base_decay * arousal;

    hygiene.level = (hygiene.level - decay).max(0.0);

    // Auto-clean: creature grooms itself when idle and dirty
    if hygiene.level < cfg::AUTO_CLEAN_THRESHOLD
        && mind.mood != MoodState::Sleeping
        && mind.mood != MoodState::Sick
        && mind.stats.energy > 20.0
    {
        // Small self-clean (10% of manual clean effectiveness)
        hygiene.level = (hygiene.level + 0.5).min(100.0);
    }
}

/// Applies penalties when hygiene is low.
fn hygiene_effect_system(
    hygiene: Res<HygieneState>,
    mut mind: ResMut<Mind>,
) {
    if hygiene.level < cfg::HEALTH_PENALTY_THRESHOLD {
        let severity = 1.0 - (hygiene.level / cfg::HEALTH_PENALTY_THRESHOLD);
        mind.stats.health = (mind.stats.health - cfg::HEALTH_PENALTY * severity).max(0.0);
    }

    if hygiene.level < cfg::HAPPINESS_PENALTY_THRESHOLD {
        let severity = 1.0 - (hygiene.level / cfg::HAPPINESS_PENALTY_THRESHOLD);
        mind.stats.happiness = (mind.stats.happiness - cfg::HAPPINESS_PENALTY * severity).max(0.0);
    }
}

/// Processes clean events — applies species-specific cleaning.
fn clean_action_system(
    mut events: EventReader<CleanEvent>,
    mut hygiene: ResMut<HygieneState>,
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
    mut reaction_events: EventWriter<crate::creature::reactions::CreatureReaction>,
) {
    for _event in events.read() {
        let effectiveness = match genome.species {
            Species::Moluun => cfg::clean::MOLUUN_GROOM,
            Species::Pylum  => cfg::clean::PYLUM_PREEN,
            Species::Skael  => cfg::clean::SKAEL_SHED,
            Species::Nyxal  => {
                // Ink clean costs energy
                if mind.stats.energy >= cfg::clean::NYXAL_INK_ENERGY_COST {
                    mind.stats.energy -= cfg::clean::NYXAL_INK_ENERGY_COST;
                    cfg::clean::NYXAL_INK
                } else {
                    // Too tired to ink clean — reduced effect
                    cfg::clean::NYXAL_INK * 0.3
                }
            }
        };

        hygiene.level = (hygiene.level + effectiveness).min(100.0);

        // Small happiness boost from being clean
        mind.stats.happiness = (mind.stats.happiness + 3.0).min(100.0);

        // Visual reaction
        reaction_events.write(crate::creature::reactions::CreatureReaction::Cleaning);
    }
}

/// Returns the species-specific cleaning action name.
pub fn clean_action_name(species: &Species) -> &'static str {
    match species {
        Species::Moluun => "Groom",
        Species::Pylum  => "Preen",
        Species::Skael  => "Shed",
        Species::Nyxal  => "Ink Clean",
    }
}
