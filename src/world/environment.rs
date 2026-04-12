//! Environment system — temperature cycle and creature comfort.
//!
//! Temperature follows a day/night sine cycle. Creatures outside their
//! comfort zone drain energy and happiness. Fat and skin type provide insulation.

use bevy::prelude::*;

use crate::config::environment as cfg;
use crate::creature::anatomy::AnatomyState;
use crate::creature::anatomy::skin::SkinCovering;
use crate::game::state::AppState;
use crate::genome::Genome;
use crate::mind::Mind;

/// Tracks the current environment state.
#[derive(Resource, Debug)]
pub struct EnvironmentState {
    /// Current temperature in Celsius.
    pub temperature: f32,
    /// Temporary warmth buff from food (decays per tick).
    pub warmth_buff: f32,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self { temperature: cfg::BASE_TEMP, warmth_buff: 0.0 }
    }
}

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnvironmentState::default())
            .add_systems(Update, (
                temperature_cycle_system,
                comfort_effect_system,
            ).chain().run_if(in_state(AppState::Gameplay)));
    }
}

/// Updates temperature based on time of day (sine wave).
fn temperature_cycle_system(
    time: Res<Time>,
    mut env: ResMut<EnvironmentState>,
) {
    // Use real elapsed time for a smooth cycle (~24 min = 1 game day)
    let elapsed = time.elapsed_secs();
    let cycle = (elapsed / (24.0 * 60.0) * std::f32::consts::TAU).sin();
    env.temperature = cfg::BASE_TEMP + cycle * cfg::DAY_AMPLITUDE;

    // Decay warmth buff
    if env.warmth_buff > 0.0 {
        env.warmth_buff = (env.warmth_buff - 0.5).max(0.0);
    }
}

/// Applies comfort/discomfort effects based on temperature vs species preference.
fn comfort_effect_system(
    env: Res<EnvironmentState>,
    genome: Res<Genome>,
    anatomy: Option<Res<AnatomyState>>,
    mut mind: ResMut<Mind>,
) {
    let (min_comfort, max_comfort) = cfg::comfort::range(&genome.species);
    let temp = env.temperature + env.warmth_buff; // warming food shifts effective temperature

    // How far outside comfort zone (0.0 = comfortable, positive = uncomfortable)
    let discomfort = if temp < min_comfort {
        (min_comfort - temp) / 10.0
    } else if temp > max_comfort {
        (temp - max_comfort) / 10.0
    } else {
        return; // comfortable — no effect
    };

    // Insulation reduces discomfort
    let mut insulation = 0.0;
    if let Some(ref anat) = anatomy {
        // Fat insulation
        insulation += anat.fat.level * cfg::FAT_INSULATION_FACTOR;
        // Skin type insulation
        insulation += match anat.skin.covering {
            SkinCovering::Fur      => cfg::skin_insulation::FUR,
            SkinCovering::Plumage  => cfg::skin_insulation::PLUMAGE,
            SkinCovering::Scales   => cfg::skin_insulation::SCALES,
            SkinCovering::Membrane => cfg::skin_insulation::MEMBRANE,
        };
    }

    let effective_discomfort = (discomfort * (1.0 - insulation.min(0.9))).max(0.0);

    if effective_discomfort > 0.0 {
        mind.stats.energy = (mind.stats.energy - cfg::DISCOMFORT_ENERGY_DRAIN * effective_discomfort).max(0.0);
        mind.stats.happiness = (mind.stats.happiness - cfg::DISCOMFORT_HAPPINESS_DRAIN * effective_discomfort).max(0.0);
    }
}
