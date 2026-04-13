//! Involuntary body functions — blink, pupil, postural tone, startle.
//!
//! These run continuously and independently from idle behaviors and reactions.
//! Real animals blink every 3-8 seconds, pupils dilate/contract with arousal,
//! and muscle tone shifts between relaxed and tense. These aren't decisions —
//! they're the body running itself.
//!
//! ## Systems
//!
//! - **Blink**: periodic eye closure (0.15s), rate varies with ANS
//! - **Pupil**: dilation (sympathetic) / contraction (parasympathetic)
//! - **Postural tone**: body shape subtly shifts (relaxed = round, tense = tight)
//! - **Startle reflex**: instant flinch response to sudden touch events

use bevy::prelude::*;
use rand::Rng;

use crate::game::state::AppState;
use crate::mind::autonomic::AutonomicState;
use crate::mind::{Mind, MoodState};

/// Tracks involuntary body state.
#[derive(Resource)]
pub struct InvoluntaryState {
    // --- Blink ---
    /// Seconds until next blink.
    pub blink_timer: f32,
    /// Is the creature currently mid-blink? (duration in seconds)
    pub blink_active: f32,
    /// Blink duration (seconds). Longer when drowsy.
    pub blink_duration: f32,

    // --- Pupil ---
    /// Current pupil dilation (0.0 = pinpoint, 1.0 = fully dilated).
    pub pupil: f32,
    /// Target pupil (driven by ANS).
    pub pupil_target: f32,

    // --- Postural tone ---
    /// Body tension (0.0 = completely relaxed/loose, 1.0 = rigid/tense).
    pub tension: f32,
    pub tension_target: f32,

    // --- Startle ---
    /// Ticks remaining in startle response (0 = no startle).
    pub startle_ticks: f32,
}

impl Default for InvoluntaryState {
    fn default() -> Self {
        Self {
            blink_timer: 4.0,
            blink_active: 0.0,
            blink_duration: 0.15,
            pupil: 0.5,
            pupil_target: 0.5,
            tension: 0.3,
            tension_target: 0.3,
            startle_ticks: 0.0,
        }
    }
}

pub struct InvoluntaryPlugin;

impl Plugin for InvoluntaryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InvoluntaryState::default())
            .add_systems(Update, (
                blink_system,
                pupil_system,
                tension_system,
                startle_decay_system,
            ).run_if(in_state(AppState::Gameplay)));
    }
}

// ===================================================================
// BLINK — periodic involuntary eye closure
// ===================================================================
// Real humans blink every 3-8 seconds. Rate varies:
// - Alert (sympathetic): blink LESS (eyes wide, scanning)
// - Calm (parasympathetic): blink normally
// - Drowsy: blink MORE, slower, longer closure
// - Sleeping: no blinks (eyes already closed)

fn blink_system(
    time: Res<Time>,
    mind: Res<Mind>,
    ans: Option<Res<AutonomicState>>,
    mut state: ResMut<InvoluntaryState>,
) {
    let dt = time.delta_secs();

    // No blinking while sleeping (eyes already closed)
    if mind.mood == MoodState::Sleeping {
        state.blink_active = 0.0;
        return;
    }

    // If mid-blink, count down
    if state.blink_active > 0.0 {
        state.blink_active -= dt;
        return;
    }

    // Count down to next blink
    state.blink_timer -= dt;
    if state.blink_timer > 0.0 { return; }

    // BLINK! Close eyes briefly
    let arousal = ans.as_ref().map(|a| a.level).unwrap_or(0.4);

    // Blink duration: 0.1s (alert) to 0.3s (drowsy)
    state.blink_duration = 0.1 + (1.0 - arousal) * 0.2;
    state.blink_active = state.blink_duration;

    // Next blink interval: 2s (drowsy) to 8s (alert) + randomness
    let mut rng = rand::rng();
    let base_interval = 2.0 + arousal * 6.0; // alert = longer between blinks
    let jitter = rng.random_range(-1.0..1.0);
    state.blink_timer = (base_interval + jitter).max(1.0);
}

// ===================================================================
// PUPIL — dilation driven by ANS
// ===================================================================
// Sympathetic: pupils DILATE (let in more light, scanning for threats)
// Parasympathetic: pupils CONTRACT (focused, calm)

fn pupil_system(
    time: Res<Time>,
    ans: Option<Res<AutonomicState>>,
    mut state: ResMut<InvoluntaryState>,
) {
    let arousal = ans.as_ref().map(|a| a.level).unwrap_or(0.4);

    // Target: sympathetic = dilated (0.8-1.0), parasympathetic = contracted (0.2-0.4)
    state.pupil_target = 0.2 + arousal * 0.8;

    // Smooth blend (pupils don't snap)
    let dt = time.delta_secs();
    let diff = state.pupil_target - state.pupil;
    state.pupil += diff * 2.0 * dt; // blend speed
}

// ===================================================================
// POSTURAL TONE — body tension from ANS
// ===================================================================
// Alert: muscles tighten, body compresses slightly
// Calm: muscles relax, body settles and rounds out

fn tension_system(
    time: Res<Time>,
    ans: Option<Res<AutonomicState>>,
    mut state: ResMut<InvoluntaryState>,
) {
    let arousal = ans.as_ref().map(|a| a.level).unwrap_or(0.4);

    // Target tension maps directly from arousal
    state.tension_target = arousal;

    // Slow blend (posture shifts gradually)
    let dt = time.delta_secs();
    let diff = state.tension_target - state.tension;
    state.tension += diff * 0.5 * dt;
}

// ===================================================================
// STARTLE — decay after sudden stimulus
// ===================================================================

fn startle_decay_system(
    time: Res<Time>,
    mut state: ResMut<InvoluntaryState>,
) {
    if state.startle_ticks > 0.0 {
        state.startle_ticks = (state.startle_ticks - time.delta_secs()).max(0.0);
    }
}

/// Called externally when a sudden stimulus occurs (loud sound, unexpected touch).
pub fn trigger_startle(state: &mut InvoluntaryState) {
    state.startle_ticks = 0.5; // half-second startle
    state.pupil = 1.0;         // instant pupil dilation
    state.tension = 1.0;       // instant muscle tension
}
