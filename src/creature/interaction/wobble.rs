//! Ambient wobble — continuous tiny impulses on extremity points so the
//! creature is never fully still. Tail tips, wingtips, tentacle tips, ears
//! drift with sine waves at slightly offset phases, giving a living feel
//! even when nothing is happening.
//!
//! Wobble amplitude scales inversely with arousal: calm creatures sway more
//! (breathing, floating), alert creatures hold tense and still.

use std::f32::consts::TAU;
use bevy::prelude::*;

use super::soft_body::SoftBody;
use crate::game::state::AppState;
use crate::genome::{Genome, Species};

pub struct WobblePlugin;

impl Plugin for WobblePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_wobble.run_if(in_state(AppState::Gameplay)));
    }
}

/// Amplitude of wobble impulse per second on extremities.
const BASE_AMPLITUDE: f32 = 3.5;

/// Each extremity has its own phase speed so they don't all move in sync.
fn wobble(phase: f32, speed: f32, offset: f32) -> f32 {
    ((phase * speed) + offset).sin()
}

fn apply_wobble(
    time: Res<Time>,
    genome: Res<Genome>,
    ans: Option<Res<crate::mind::autonomic::AutonomicState>>,
    mut body: Option<ResMut<SoftBody>>,
) {
    let Some(ref mut body) = body else { return };

    let t = time.elapsed_secs();
    let dt = time.delta_secs();

    // Calmer = more wobble (rest = life). Arousal ≈ 1 → near zero wobble.
    let arousal = ans.as_ref().map(|a| a.level).unwrap_or(0.3);
    let amplitude = BASE_AMPLITUDE * (1.0 - arousal).max(0.2);

    // Scale impulse by dt so it's framerate-independent.
    // The force is a small nudge each frame; the soft-body damping keeps it small.
    let a = amplitude * dt;

    match genome.species {
        Species::Moluun => {
            // Floppy ear drift + subtle paw sway
            body.impulse("ear_anchor", Vec2::new(wobble(t, 1.3, 0.0) * a, wobble(t, 0.9, 1.0) * a * 0.6));
            body.impulse("paw_l", Vec2::new(wobble(t, 0.7, 2.1) * a * 0.5, 0.0));
            body.impulse("paw_r", Vec2::new(wobble(t, 0.7, 3.6) * a * 0.5, 0.0));
        }
        Species::Pylum => {
            // Wingtips ripple, tail feathers sway, casque barely moves
            body.impulse("wingtip_l", Vec2::new(wobble(t, 1.6, 0.0) * a * 1.2, wobble(t, 1.2, 0.5) * a * 0.8));
            body.impulse("wingtip_r", Vec2::new(wobble(t, 1.6, TAU / 2.0) * a * 1.2, wobble(t, 1.2, 2.0) * a * 0.8));
            body.impulse("tail", Vec2::new(wobble(t, 0.9, 1.1) * a * 0.8, 0.0));
            body.impulse("casque", Vec2::new(wobble(t, 0.5, 0.3) * a * 0.3, 0.0));
        }
        Species::Skael => {
            // Long tail snakes, horns barely move
            body.impulse("tail_3", Vec2::new(wobble(t, 0.9, 0.0) * a * 1.5, wobble(t, 0.7, 0.8) * a * 0.6));
            body.impulse("tail_2", Vec2::new(wobble(t, 0.9, 1.1) * a * 1.0, 0.0));
            body.impulse("horn_l", Vec2::new(wobble(t, 0.4, 0.4) * a * 0.3, 0.0));
            body.impulse("horn_r", Vec2::new(wobble(t, 0.4, 3.0) * a * 0.3, 0.0));
        }
        Species::Nyxal => {
            // All four tentacle tips drift in slow offset patterns (underwater feel)
            body.impulse("tip_fl", Vec2::new(wobble(t, 0.7, 0.0) * a * 1.3, wobble(t, 0.6, 0.5) * a * 0.8));
            body.impulse("tip_fr", Vec2::new(wobble(t, 0.7, TAU / 2.0) * a * 1.3, wobble(t, 0.6, 2.0) * a * 0.8));
            body.impulse("tip_bl", Vec2::new(wobble(t, 0.6, 1.0) * a * 1.1, wobble(t, 0.5, 1.5) * a * 0.7));
            body.impulse("tip_br", Vec2::new(wobble(t, 0.6, 3.1) * a * 1.1, wobble(t, 0.5, 3.5) * a * 0.7));
            body.impulse("fin_l", Vec2::new(wobble(t, 1.0, 0.2) * a * 0.6, 0.0));
            body.impulse("fin_r", Vec2::new(wobble(t, 1.0, 2.7) * a * 0.6, 0.0));
        }
    }
}
