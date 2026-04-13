//! Autonomic Nervous System — sympathetic/parasympathetic control layer.
//!
//! The ANS is a single float (0.0–1.0) that governs how ALL body systems behave:
//!
//! ```text
//! 0.0 ──────── 0.3 ──────── 0.6 ──────── 1.0
//!  PARA         neutral       SYMP
//!  rest         baseline      fight/flight
//!  digest       normal        alert
//!  heal         balanced      burn
//!  slow         moderate      fast
//! ```
//!
//! Every system reads this float instead of checking moods individually.
//! This creates emergent behavior: a scared creature heals slower, digests
//! less, breathes faster, and moves jerkily — all from ONE value.
//!
//! ## Real biology
//!
//! - **Sympathetic**: pupils dilate, heart races, muscles tense, digestion stops
//! - **Parasympathetic**: pupils contract, heart slows, muscles relax, digestion active
//! - Animals shift between these constantly — never fully one or the other
//! - Displacement behaviors (scratching, grooming) emerge when BOTH are high

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::autonomic as cfg;
use crate::game::state::AppState;
use crate::mind::Mind;

/// The creature's autonomic nervous system state.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct AutonomicState {
    /// Current level: 0.0 = full parasympathetic, 1.0 = full sympathetic.
    pub level: f32,
    /// Target level (driven by mood + external factors). Blends toward this.
    pub target: f32,
    /// Conflict intensity: high when BOTH sympathetic and parasympathetic are activated.
    /// Triggers displacement behaviors (scratching, grooming, shaking).
    pub conflict: f32,
}

impl Default for AutonomicState {
    fn default() -> Self {
        Self {
            level: 0.4,     // slightly above neutral at birth
            target: 0.3,
            conflict: 0.0,
        }
    }
}

#[allow(dead_code)]
impl AutonomicState {
    /// Is the creature in sympathetic (fight/flight) mode?
    pub fn is_sympathetic(&self) -> bool {
        self.level > cfg::SYMPATHETIC_THRESHOLD
    }

    /// Is the creature in parasympathetic (rest/digest) mode?
    pub fn is_parasympathetic(&self) -> bool {
        self.level < cfg::PARASYMPATHETIC_THRESHOLD
    }

    /// Is there internal conflict (both systems fighting)?
    pub fn is_conflicted(&self) -> bool {
        self.conflict > 0.4
    }

    /// Modifier for systems that should SPEED UP with sympathetic activation.
    /// Returns 0.5 (parasympathetic) to 1.5 (sympathetic).
    pub fn arousal_multiplier(&self) -> f32 {
        0.5 + self.level
    }

    /// Modifier for systems that should SLOW DOWN with sympathetic activation.
    /// Returns 1.5 (parasympathetic) to 0.5 (sympathetic).
    pub fn calm_multiplier(&self) -> f32 {
        1.5 - self.level
    }
}

pub struct AutonomicPlugin;

impl Plugin for AutonomicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AutonomicState::default())
            .add_systems(Update, autonomic_tick.run_if(in_state(AppState::Gameplay)));
    }
}

/// Updates the autonomic state based on mood and blends toward target.
fn autonomic_tick(
    mind: Res<Mind>,
    mut ans: ResMut<AutonomicState>,
) {
    // Compute target from mood
    let mood_target = cfg::mood_drive::target(&mind.mood);

    // Health stress: low health pushes sympathetic
    let health_stress = if mind.stats.health < 30.0 {
        (30.0 - mind.stats.health) / 30.0 * 0.3  // up to +0.3 sympathetic
    } else {
        0.0
    };

    // Energy modulation: very low energy forces parasympathetic (body shutting down)
    let energy_override = if mind.stats.energy < 10.0 {
        -0.2  // pull toward rest
    } else {
        0.0
    };

    ans.target = (mood_target + health_stress + energy_override).clamp(0.0, 1.0);

    // Blend toward target
    let diff = ans.target - ans.level;
    let step = diff * cfg::BLEND_SPEED;
    ans.level = (ans.level + step).clamp(0.0, 1.0);

    // Compute conflict: high when target is far from current AND current is in mid-range
    // (the body is being pulled in both directions)
    let mid_distance = (ans.level - 0.5).abs();  // 0.0 = dead center, 0.5 = extreme
    let target_pull = diff.abs();
    ans.conflict = (target_pull * (1.0 - mid_distance) * 2.0).clamp(0.0, 1.0);
}
