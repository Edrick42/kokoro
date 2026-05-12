//! Bevy plugin that simulates the Moluun cub tail in real time.
//!
//! Owns a single [`MoluunCubTail`] resource — the physical tail body of
//! the active creature. Two systems:
//!
//! - `init_tail_body` runs once on entering `AppState::Gameplay`. It
//!   builds the tail body from the current `Genome::tail` and inserts
//!   the resource.
//! - `step_tail_body` runs every Update frame while in Gameplay. It
//!   reads `Mind::mood`, computes per-segment muscle intent via the
//!   species mood-pattern, then advances physics by `Time::delta_secs`.
//!
//! The actual pixel rendering happens elsewhere — see
//! `moluun::overlay_tail` for the brush call. This module owns only
//! simulation; it draws nothing.

use bevy::prelude::*;
use kokoro_body::Body;
use kokoro_body::actuation::PairIntent;
use kokoro_rig::{AppliedTorques, Vec2};

use crate::game::state::AppState;
use crate::genome::Genome;
use crate::mind::Mind;
use crate::visuals::evolution::{GrowthStage, GrowthState};

use super::moluun::{
    STANDALONE_TAIL_SEGMENTS, cub_tail_body_for_creature, cub_tail_intent,
    cub_tail_pattern_for_mood,
};

/// In-game scaling for the cub tail on the 64×64 canvas. Tail attaches
/// at the back of the body and extends to the left (PI rad) since the
/// quadrupedal cub side-view faces right.
const CUB_TAIL_LENGTH_PX: f32 = 32.0;
const CUB_TAIL_ATTACH_X: f32 = 28.0;
const CUB_TAIL_ATTACH_Y: f32 = 35.0;
const CUB_TAIL_BASE_ANGLE: f32 = std::f32::consts::PI;

/// Per-creature physical tail body. Lives in this resource so the
/// integration runs once globally for the active creature; multi-creature
/// support comes when the collection becomes a Component-per-entity.
///
/// `last_intent` stores the per-segment `PairIntent` from the most recent
/// step so debug overlays can read what the mind asked of each muscle
/// without re-deriving it. Index `i` corresponds to segment bone `i+1`
/// (bone 0 is the anchor root).
#[derive(Resource)]
pub struct MoluunCubTail {
    pub body: Body,
    pub sim_time: f32,
    pub last_intent: Vec<PairIntent>,
}

pub struct MoluunCubTailPlugin;

impl Plugin for MoluunCubTailPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Gameplay), init_tail_body)
            .add_systems(
                Update,
                step_tail_body.run_if(in_state(AppState::Gameplay)),
            );
    }
}

fn init_tail_body(mut commands: Commands, genome: Res<Genome>) {
    let body = cub_tail_body_for_creature(
        &genome.tail,
        CUB_TAIL_LENGTH_PX,
        Vec2::new(CUB_TAIL_ATTACH_X, CUB_TAIL_ATTACH_Y),
        CUB_TAIL_BASE_ANGLE,
    );
    commands.insert_resource(MoluunCubTail {
        body,
        sim_time: 0.0,
        last_intent: vec![PairIntent::rest(); STANDALONE_TAIL_SEGMENTS],
    });
}

fn step_tail_body(
    time: Res<Time>,
    mind: Res<Mind>,
    growth: Res<GrowthState>,
    genome: Res<Genome>,
    tail: Option<ResMut<MoluunCubTail>>,
) {
    use crate::genome::Species;
    // Tail only animates for Moluun cubs; other species/stages still
    // hold a resource (so reading it later doesn't panic) but skip the
    // step so simulated state stays parked at rest.
    let Some(mut tail) = tail else { return };
    if !matches!(genome.species, Species::Moluun) || !matches!(growth.stage, GrowthStage::Cub) {
        return;
    }
    let pattern = cub_tail_pattern_for_mood(&mind.mood);
    // Cap dt to avoid explosions if the frame stalls (lag spike on load).
    let dt = time.delta_secs().min(0.05);
    let t_snapshot = tail.sim_time;

    // Pre-compute intent for every active segment so we can both feed it
    // to the integrator and keep a copy on the resource for debug overlays.
    let intents: Vec<PairIntent> = (1..=STANDALONE_TAIL_SEGMENTS)
        .map(|seg| cub_tail_intent(&pattern, t_snapshot, seg, STANDALONE_TAIL_SEGMENTS))
        .collect();

    tail.body.step(
        dt,
        |bone_id| {
            let seg = bone_id.0 as usize;
            if (1..=STANDALONE_TAIL_SEGMENTS).contains(&seg) {
                intents[seg - 1]
            } else {
                PairIntent::rest()
            }
        },
        |_| AppliedTorques::default(),
    );
    tail.last_intent = intents;
    tail.sim_time += dt;
    tail.body.skeleton.forward();
}
