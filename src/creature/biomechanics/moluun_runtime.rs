//! Bevy plugins that simulate the Moluun cub's biomechanics in real
//! time. Today: a 5-vertebra **spine** that anchors at the cub's
//! cervical end, and a 16-segment **tail** that hangs off the spine's
//! sacral tip. Resources, plugins, and step systems for each live in
//! this file; the static templates that describe their anatomy live in
//! `moluun.rs`.
//!
//! Pipeline each frame:
//!
//! 1. `step_spine_body` runs the spine physics (passive at rest today;
//!    a mood-driven posture pattern will land later).
//! 2. `bridge_spine_to_tail` reads the spine's sacral-tip position
//!    and pushes it into the tail's `root_position`. This is how the
//!    spine carries the tail around when it eventually starts moving.
//! 3. `step_tail_body` runs the tail's mood-driven muscle intent +
//!    physics on top of the (possibly-moved) root.
//!
//! Render happens elsewhere — see `tail_render` + `spine_render`. This
//! module owns only simulation; it draws nothing.

use bevy::prelude::*;
use kokoro_body::Body;
use kokoro_body::actuation::PairIntent;
use kokoro_rig::{AppliedTorques, BoneId, Vec2};

use crate::game::state::AppState;
use crate::genome::Genome;
use crate::mind::Mind;
use crate::visuals::evolution::{GrowthStage, GrowthState};

use super::moluun::{
    STANDALONE_SPINE_SEGMENTS, STANDALONE_TAIL_SEGMENTS,
    cub_spine_body_for_creature,
    cub_tail_body_for_creature, cub_tail_intent, cub_tail_pattern_for_mood,
};

/// In-game scaling for the cub spine on the 64×64 canvas. Cervical end
/// sits forward at the right; chain extends leftward (PI) so the
/// sacral end falls where the tail used to anchor.
const CUB_SPINE_LENGTH_PX: f32 = 20.0;
const CUB_SPINE_ATTACH_X:  f32 = 48.0;
const CUB_SPINE_ATTACH_Y:  f32 = 32.0;
const CUB_SPINE_BASE_ANGLE: f32 = std::f32::consts::PI;

/// In-game scaling for the cub tail. Length is a constant; the actual
/// attach position is overwritten every frame by `bridge_spine_to_tail`
/// from the spine's sacral tip, so the values below are just the
/// pre-bridge defaults used until the spine has stepped once.
const CUB_TAIL_LENGTH_PX: f32 = 32.0;
const CUB_TAIL_ATTACH_X: f32 = 28.0;
const CUB_TAIL_ATTACH_Y: f32 = 32.0;
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
/// Soft-tissue layers (fat, skin, fur) live on each bone's
/// `kokoro_rig::Tissue` directly — not on parallel Vecs here — so any
/// future body part inherits the same five-layer model with no extra
/// bookkeeping. See `docs/biomechanics.md` §10.1 ("add fields to
/// existing structs, don't introduce parallel hierarchies").
#[derive(Resource)]
pub struct MoluunCubTail {
    pub body: Body,
    pub sim_time: f32,
    pub last_intent: Vec<PairIntent>,
}

/// Per-creature physical spine body. Like `MoluunCubTail`: one global
/// resource for the active cub. Five vertebra-equivalent segments.
/// Postural patterns (breathing, mood-driven arch) will add a
/// `last_intent: Vec<PairIntent>` field when they land, mirroring
/// `MoluunCubTail`.
#[derive(Resource)]
pub struct MoluunCubSpine {
    pub body: Body,
    pub sim_time: f32,
}

pub struct MoluunCubTailPlugin;

impl Plugin for MoluunCubTailPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(AppState::Gameplay),
                (init_spine_body, init_tail_body).chain(),
            )
            .add_systems(
                Update,
                // Order matters: spine steps first, the bridge pushes
                // its sacral tip into the tail's root, then the tail
                // steps on top of that updated root.
                (step_spine_body, bridge_spine_to_tail, step_tail_body)
                    .chain()
                    .run_if(in_state(AppState::Gameplay)),
            );
    }
}

fn init_spine_body(mut commands: Commands, genome: Res<Genome>) {
    // Coat fluffiness is reused from the tail's strength gene for now —
    // a future `CoatGenes` will separate the trait properly.
    let coat_fluffiness = genome.tail.strength;
    let mut body = cub_spine_body_for_creature(
        genome.appetite,
        genome.resilience,
        coat_fluffiness,
        CUB_SPINE_LENGTH_PX,
        Vec2::new(CUB_SPINE_ATTACH_X, CUB_SPINE_ATTACH_Y),
        CUB_SPINE_BASE_ANGLE,
    );
    // Run FK once so the initial world positions are sane before the
    // first bridge tick reads the sacral tip.
    body.skeleton.forward();
    commands.insert_resource(MoluunCubSpine { body, sim_time: 0.0 });
}

fn step_spine_body(
    time: Res<Time>,
    growth: Res<GrowthState>,
    genome: Res<Genome>,
    spine: Option<ResMut<MoluunCubSpine>>,
) {
    use crate::genome::Species;
    let Some(mut spine) = spine else { return };
    if !matches!(genome.species, Species::Moluun) || !matches!(growth.stage, GrowthStage::Cub) {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    // Passive physics for now: rest intent everywhere, the ligament
    // springs hold the spine at rest_angle. Postural patterns
    // (breathing, mood-driven arch) land in a follow-up.
    spine.body.step(
        dt,
        |_| PairIntent::rest(),
        |_| AppliedTorques::default(),
    );
    spine.sim_time += dt;
    spine.body.skeleton.forward();
}

/// Reads the spine's sacral tip + posterior-segment world angle and
/// pushes them into the tail. The tail's first joint's `rest_angle` is
/// also updated to match, so the ligament spring naturally pulls the
/// tail toward whatever orientation the spine ends at.
fn bridge_spine_to_tail(
    spine: Option<Res<MoluunCubSpine>>,
    tail: Option<ResMut<MoluunCubTail>>,
    growth: Res<GrowthState>,
    genome: Res<Genome>,
) {
    use crate::genome::Species;
    let (Some(spine), Some(mut tail)) = (spine, tail) else { return };
    if !matches!(genome.species, Species::Moluun) || !matches!(growth.stage, GrowthStage::Cub) {
        return;
    }
    let sacral_bone = BoneId(STANDALONE_SPINE_SEGMENTS as u16);
    let tip = spine.body.skeleton.world_tip(sacral_bone);
    tail.body.skeleton.set_root(tip);
}

fn init_tail_body(mut commands: Commands, genome: Res<Genome>) {
    let body = cub_tail_body_for_creature(
        &genome.tail,
        genome.appetite,
        genome.resilience,
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
