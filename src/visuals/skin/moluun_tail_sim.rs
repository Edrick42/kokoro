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
use kokoro_rig::{AppliedTorques, BoneId, Vec2};

use crate::game::state::AppState;
use crate::genome::Genome;
use crate::mind::Mind;
use crate::visuals::evolution::{GrowthStage, GrowthState};

use super::super::rigs::moluun::{
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
#[derive(Resource)]
pub struct MoluunCubTail {
    pub body: Body,
    pub sim_time: f32,
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
    commands.insert_resource(MoluunCubTail { body, sim_time: 0.0 });
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
    tail.body.step(
        dt,
        |bone_id| {
            let seg = bone_id.0 as usize;
            if (1..=STANDALONE_TAIL_SEGMENTS).contains(&seg) {
                cub_tail_intent(&pattern, t_snapshot, seg, STANDALONE_TAIL_SEGMENTS)
            } else {
                PairIntent::rest()
            }
        },
        |_| AppliedTorques::default(),
    );
    tail.sim_time += dt;
    tail.body.skeleton.forward();
}

/// Extract the tail joint polyline in canvas coordinates, ready to feed
/// into a `FusiformTail` brush. Returns `None` if forward kinematics
/// hasn't been run yet (the dirty flag is set).
pub fn joints_for_overlay(tail: &MoluunCubTail) -> Option<Vec<(i32, i32)>> {
    if tail.body.skeleton.dirty() {
        return None;
    }
    let mut joints: Vec<(i32, i32)> = Vec::with_capacity(STANDALONE_TAIL_SEGMENTS + 1);
    let base = tail.body.skeleton.world_base(BoneId(0));
    joints.push((base.x.round() as i32, base.y.round() as i32));
    for i in 1..=STANDALONE_TAIL_SEGMENTS {
        let tip = tail.body.skeleton.world_tip(BoneId(i as u16));
        joints.push((tip.x.round() as i32, tip.y.round() as i32));
    }
    Some(joints)
}
