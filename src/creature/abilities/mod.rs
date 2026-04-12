//! Species abilities — unique passive/active skills per species.
//!
//! Each species has one signature ability:
//! - Moluun: Scent Trail (passive — marks territory)
//! - Pylum: Thermal Sight (active — heat overlay)
//! - Skael: Armored Curl (active — defensive pose)
//! - Nyxal: Chromatophore Pulse (passive — color shifts with mood)

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::abilities as cfg;
use crate::game::state::AppState;
use crate::genome::{Genome, Species};
use crate::mind::{Mind, MoodState};

/// Tracks the creature's ability state.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct AbilityState {
    /// Cooldown ticks remaining (0 = ready).
    pub cooldown: u32,
    /// Active ticks remaining (0 = inactive).
    pub active_ticks: u32,
    /// Which ability is available.
    pub kind: AbilityKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbilityKind {
    ScentTrail,
    ThermalSight,
    ArmoredCurl,
    ChromatophorePulse,
}

impl AbilityKind {
    pub fn for_species(species: &Species) -> Self {
        match species {
            Species::Moluun => AbilityKind::ScentTrail,
            Species::Pylum  => AbilityKind::ThermalSight,
            Species::Skael  => AbilityKind::ArmoredCurl,
            Species::Nyxal  => AbilityKind::ChromatophorePulse,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AbilityKind::ScentTrail        => "Scent Trail",
            AbilityKind::ThermalSight      => "Thermal Sight",
            AbilityKind::ArmoredCurl       => "Armored Curl",
            AbilityKind::ChromatophorePulse => "Chromatophore Pulse",
        }
    }

    pub fn is_passive(&self) -> bool {
        matches!(self, AbilityKind::ScentTrail | AbilityKind::ChromatophorePulse)
    }
}

impl Default for AbilityState {
    fn default() -> Self {
        Self {
            cooldown: 0,
            active_ticks: 0,
            kind: AbilityKind::ScentTrail,
        }
    }
}

/// Event fired when the player activates an ability (for active abilities only).
#[derive(Event)]
pub struct ActivateAbilityEvent;

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AbilityState::default())
            .add_event::<ActivateAbilityEvent>()
            .add_systems(Startup, init_ability)
            .add_systems(Update, (
                ability_tick_system,
                passive_ability_system,
            ).run_if(in_state(AppState::Gameplay)));
    }
}

fn init_ability(genome: Option<Res<Genome>>, mut ability: ResMut<AbilityState>) {
    if let Some(genome) = genome {
        ability.kind = AbilityKind::for_species(&genome.species);
    }
}

/// Ticks cooldowns and active durations.
fn ability_tick_system(
    mut ability: ResMut<AbilityState>,
    mut events: EventReader<ActivateAbilityEvent>,
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
) {
    // Tick down
    if ability.cooldown > 0 { ability.cooldown -= 1; }
    if ability.active_ticks > 0 {
        ability.active_ticks -= 1;

        // Armored Curl: recover energy while active
        if ability.kind == AbilityKind::ArmoredCurl {
            mind.stats.energy = (mind.stats.energy + cfg::armored_curl::ENERGY_RECOVERY * 0.1).min(100.0);
        }
    }

    // Handle activation events (active abilities only)
    for _event in events.read() {
        if ability.cooldown > 0 || ability.kind.is_passive() { continue; }

        match ability.kind {
            AbilityKind::ThermalSight => {
                if mind.stats.energy >= cfg::thermal_sight::ENERGY_COST {
                    mind.stats.energy -= cfg::thermal_sight::ENERGY_COST;
                    ability.active_ticks = cfg::thermal_sight::DURATION;
                    ability.cooldown = cfg::thermal_sight::COOLDOWN;
                }
            }
            AbilityKind::ArmoredCurl => {
                ability.active_ticks = cfg::armored_curl::DURATION;
                ability.cooldown = cfg::armored_curl::COOLDOWN;
            }
            _ => {}
        }
    }
}

/// Passive abilities that trigger automatically.
fn passive_ability_system(
    ability: Res<AbilityState>,
    mind: Res<Mind>,
) {
    match ability.kind {
        AbilityKind::ScentTrail => {
            // Scent particles emitted periodically (visual handled by effects system)
            // The actual particle spawning would go here when effects system is wired
        }
        AbilityKind::ChromatophorePulse => {
            // Color shift intensity based on mood (visual handled by skin system)
            // The SkinParams already has a `glow` field for this
        }
        _ => {}
    }
}
