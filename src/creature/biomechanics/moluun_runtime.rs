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
///
/// The three Vec<f32> below stack on top of the muscle in real
/// anatomical order, each in canvas pixels perpendicular to the bone:
/// bone → muscle → **fat** → **skin** → **fur**.
/// Treated as quasi-static (genome-derived) today; promoted to real
/// `kokoro_body::{Fat, Skin, Fur}` structs on `Body` when other body
/// parts also grow these layers and they need cross-body state.
#[derive(Resource)]
pub struct MoluunCubTail {
    pub body: Body,
    pub sim_time: f32,
    pub last_intent: Vec<PairIntent>,
    /// Subcutaneous fat thickness per segment. Tapers base → tip
    /// (animals carry more padding near the body); scales with the
    /// appetite gene (fatter cubs carry more).
    pub fat_thicknesses: Vec<f32>,
    /// Skin layer thickness per segment. Approximately uniform along
    /// the tail; scales with the resilience gene.
    pub skin_thicknesses: Vec<f32>,
    /// Fur length per segment. Bell-curve distribution (zero at ends,
    /// peak in middle) produces the bushy fusiform silhouette.
    pub fur_lengths: Vec<f32>,
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
    let appetite = genome.appetite;
    let resilience = genome.resilience;
    let strength = genome.tail.strength;
    let segments = STANDALONE_TAIL_SEGMENTS;

    let fat_thicknesses = (0..segments)
        .map(|seg| cub_tail_fat_thickness(seg, segments, appetite))
        .collect();
    let skin_thicknesses = (0..segments)
        .map(|seg| cub_tail_skin_thickness(seg, segments, resilience))
        .collect();
    let fur_lengths = (0..segments)
        .map(|seg| cub_tail_fur_length(seg, segments, strength))
        .collect();

    commands.insert_resource(MoluunCubTail {
        body,
        sim_time: 0.0,
        last_intent: vec![PairIntent::rest(); segments],
        fat_thicknesses,
        skin_thicknesses,
        fur_lengths,
    });
}

/// Subcutaneous fat distribution along the tail, in canvas pixels.
/// Tapers from `BASE` near the body to `TIP` at the end — real animals
/// carry more padding closer to the trunk. `appetite` scales the whole
/// layer 0.5x..1.5x so a chunky cub reads as chunky.
fn cub_tail_fat_thickness(seg: usize, segments: usize, appetite: f32) -> f32 {
    const BASE_FAT_PX: f32 = 0.6;
    const TIP_FAT_PX:  f32 = 0.2;
    let t = (seg as f32) / ((segments - 1).max(1) as f32);
    let interp = BASE_FAT_PX * (1.0 - t) + TIP_FAT_PX * t;
    interp * (0.5 + appetite.clamp(0.0, 1.0))
}

/// Skin layer thickness along the tail. Approximately uniform; scales
/// 0.75x..1.25x with the resilience gene (hardier cubs have tougher
/// skin).
fn cub_tail_skin_thickness(_seg: usize, _segments: usize, resilience: f32) -> f32 {
    const SKIN_PX: f32 = 0.3;
    SKIN_PX * (0.75 + 0.5 * resilience.clamp(0.0, 1.0))
}

/// Bell-curve fur distribution along the tail, in canvas pixels.
///
/// The classic fusiform "thin-thick-thin" silhouette of a red-panda-style
/// tail comes from the fur, not from any underlying anatomy: the bones
/// + muscles taper monotonically while the fur halo peaks in the middle.
/// Genome `strength` scales the whole halo so bushier cubs read bigger.
fn cub_tail_fur_length(seg: usize, segments: usize, strength: f32) -> f32 {
    const MAX_FUR_PX: f32 = 2.5;
    let t = (seg as f32) / ((segments - 1).max(1) as f32);
    let bell = (std::f32::consts::PI * t).sin();
    MAX_FUR_PX * bell * (0.75 + 0.5 * strength.clamp(0.0, 1.0))
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
