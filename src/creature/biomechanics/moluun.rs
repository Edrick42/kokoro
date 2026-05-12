//! Moluun skeletons.
//!
//! Anatomy lean per `feedback_moluun_red_panda_lean.md`: red panda primary,
//! wombat juvenile body mass secondary, koala ear shape tertiary.
//!
//! ## Pose convention
//!
//! Cub is rendered in **quadrupedal side view**, head facing right, tail
//! curling left — matching the pixel-art ref the user picked. This breaks
//! with the bipedal-frontal pose used by the other species' draws today;
//! they'll either follow suit or stay frontal as their own design choices.
//!
//! All canvas math assumes the standard 64×64 sprite buffer. Pelvis sits
//! at canvas (22, 36) by default — the cub occupies roughly x=8..58 and
//! y=22..50, leaving room above for ear tips and below for ground shadow.
//!
//! ## Convention notes
//!
//! - **Pelvis is the root**, length 0, angle 0 — children with rest_angle
//!   0 continue along +x (which in this side-view rig is "forward",
//!   toward the head on the right).
//! - **Z layering** controls front-vs-back leg rendering and the back ear
//!   peeking around the head: front leg + front ear z=+1; back leg + back
//!   ear z=-1.
//! - **Stiffness**: skull / spine / hips = Rigid (skeleton wins).
//!   Tail beyond tail_2 + paws + ear tips = Soft (soft body wins).

use kokoro_rig::{Bone, BoneId, BoneTissue, Joint, Skeleton, Stiffness, Vec2};
use std::f32::consts::PI;

/// Build the Moluun cub skeleton in quadrupedal side-view pose.
///
/// 19 bones total. Default rest pose centred so cub silhouette covers
/// roughly x=8..58 and y=22..50 of the 64×64 canvas. Genome scale-factors
/// all default to 1.0.
//
// `dead_code` allow: cub_skeleton is the consumer-facing API but the
// consumer (moluun::draw_cub) lives in a sibling crate file. The unit
// tests here are the only direct callers in this module's view.
#[allow(dead_code)]
pub fn cub_skeleton() -> Skeleton {
    let mut bones = Vec::with_capacity(19);

    // ============================================================
    // ROOT — pelvis at the BACK of the body, pointing forward (+x).
    // ============================================================
    bones.push(Bone::root("pelvis", 0.0, 1.0));
    let pelvis = BoneId(0);

    // ============================================================
    // SPINE → NECK → HEAD — horizontal column running forward.
    // ============================================================
    // Spine length 12 → spine tip at (root + 12, root) = the shoulder
    // area. Wide width 6 = body's vertical thickness.
    bones.push(Bone::child("spine", pelvis, Vec2::ZERO, 0.0, 12.0, 6.0));
    let spine = BoneId(1);

    // Neck angles slightly upward (-π/8 from spine) so the head sits a
    // bit above the spine line — proper red-panda "alert" silhouette.
    bones.push(Bone::child("neck", spine, Vec2::ZERO, -PI / 8.0, 3.0, 3.0));
    let neck = BoneId(2);

    // Head — large for cub, length defines vertical extent of the cranium.
    // z=+1 so face renders over the body when overlapping.
    bones.push(Bone::child("head", neck, Vec2::ZERO, -PI / 12.0, 9.0, 8.0).with_z(1));
    let head = BoneId(3);

    // ============================================================
    // EARS — both above head; back ear partially hidden behind front.
    // ============================================================
    // Local +x in head frame ≈ "up the head" given head's tilted-up angle.
    // Local (4, -2) places ear bases above and slightly to the front of
    // the head's tip; angle differences spread the two ears apart.
    bones.push(
        Bone::child("ear_front", head, Vec2::new(4.0, -1.0), -PI / 4.0, 4.0, 3.0)
            .with_z(2)
            .with_stiffness(Stiffness::Soft),
    );
    bones.push(
        Bone::child("ear_back", head, Vec2::new(2.0, 1.0), -PI / 6.0, 4.0, 3.0)
            .with_z(0) // behind the head dome — only the tip pokes through
            .with_stiffness(Stiffness::Soft),
    );

    // ============================================================
    // FACE FEATURES — single visible eye + snout (side view).
    // ============================================================
    // Eye sits on the side of the head (toward viewer). Local (3, 1)
    // places it lower on the face per kindchenschema.
    bones.push(Bone::child("eye", head, Vec2::new(3.0, 1.0), 0.0, 1.0, 1.0).with_z(2));
    // Snout at the front-bottom of the head — caller paints it as a tiny
    // black wedge.
    bones.push(Bone::child("snout", head, Vec2::new(7.0, 1.0), 0.0, 2.0, 2.0).with_z(2));

    // ============================================================
    // FRONT LEG — at forward end of spine, dangling down.
    // ============================================================
    // Local (10, 2) places shoulder at world (~32, ~38) — under-front of
    // the body. Angle π/2 against pelvis (which is angle 0) → world π/2
    // = straight DOWN. Front leg renders in front (z=+1).
    bones.push(
        Bone::child("shoulder_front", spine, Vec2::new(10.0, 2.0), PI / 2.0, 0.0, 1.0).with_z(1),
    );
    let shoulder_front = BoneId(8);
    bones.push(
        Bone::child("leg_front", shoulder_front, Vec2::ZERO, 0.0, 6.0, 3.0).with_z(1),
    );
    let leg_front = BoneId(9);
    bones.push(
        Bone::child("paw_front", leg_front, Vec2::ZERO, 0.0, 2.0, 3.0)
            .with_z(1)
            .with_stiffness(Stiffness::Soft),
    );

    // ============================================================
    // BACK LEG — at pelvis end, dangling down.
    // ============================================================
    // Local (0, 2) places hip at world (~22, ~38) — under-back of body.
    // Same downward angle. Back leg renders BEHIND the body (z=-1).
    bones.push(
        Bone::child("hip_back", pelvis, Vec2::new(0.0, 2.0), PI / 2.0, 0.0, 1.0).with_z(-1),
    );
    let hip_back = BoneId(11);
    bones.push(
        Bone::child("leg_back", hip_back, Vec2::ZERO, 0.0, 6.0, 3.0).with_z(-1),
    );
    let leg_back = BoneId(12);
    bones.push(
        Bone::child("paw_back", leg_back, Vec2::ZERO, 0.0, 2.0, 3.0)
            .with_z(-1)
            .with_stiffness(Stiffness::Soft),
    );

    // ============================================================
    // TAIL — RED PANDA SIGNATURE. Extends BACKWARD (-x = leftward in
    // canvas) from pelvis with a gentle downward arc.
    // ============================================================
    // Local angle PI against pelvis (which is angle 0) → world angle
    // π = pointing in -x direction (LEFT). Each subsequent segment bends
    // π/10 downward so the tail forms a gentle arc going down-left.
    // Total tail length 4+4+4+3+2 = 17 px.
    bones.push(
        Bone::child("tail_1", pelvis, Vec2::new(-2.0, 0.0), PI, 4.0, 4.0).with_z(0),
    );
    let tail_1 = BoneId(14);
    bones.push(
        Bone::child("tail_2", tail_1, Vec2::ZERO, PI / 10.0, 4.0, 3.5)
            .with_z(0)
            .with_stiffness(Stiffness::Soft),
    );
    let tail_2 = BoneId(15);
    bones.push(
        Bone::child("tail_3", tail_2, Vec2::ZERO, PI / 10.0, 4.0, 3.0)
            .with_z(0)
            .with_stiffness(Stiffness::Soft),
    );
    let tail_3 = BoneId(16);
    bones.push(
        Bone::child("tail_4", tail_3, Vec2::ZERO, PI / 10.0, 3.0, 2.5)
            .with_z(0)
            .with_stiffness(Stiffness::Soft),
    );
    let tail_4 = BoneId(17);
    bones.push(
        Bone::child("tail_5", tail_4, Vec2::ZERO, PI / 10.0, 2.0, 2.0)
            .with_z(0)
            .with_stiffness(Stiffness::Soft),
    );

    let mut sk = Skeleton::new(bones);
    sk.set_root(Vec2::new(22.0, 36.0));
    sk
}

// =====================================================================
// STANDALONE TAIL — 16-segment cat-like tail for the "per-part" redesign.
// =====================================================================
//
// Built isolated so the tail's structural form can be reviewed in a
// snapshot before the rest of the cub anatomy is re-specced. Once the
// other body parts are defined, this chain will be folded into the full
// cub skeleton; until then it is its own skeleton with its own root.
//
// Species average per design conversation 2026-05-11:
// - 16 articulated segments → smooth curvature when bent
// - Equal segment length, total ≈ body length (placeholder 70px while
//   body is still TBD)
// - All segments `Stiffness::Soft` — cat-like flexibility
// - Width field unused at the rig level (FusiformTail brush owns the
//   thickness profile from a single peak_half_width parameter)

pub const STANDALONE_TAIL_SEGMENTS: usize = 16;
const STANDALONE_TAIL_LENGTH: f32 = 70.0;
const STANDALONE_TAIL_BONE_NAMES: [&str; STANDALONE_TAIL_SEGMENTS] = [
    "tail_01", "tail_02", "tail_03", "tail_04",
    "tail_05", "tail_06", "tail_07", "tail_08",
    "tail_09", "tail_10", "tail_11", "tail_12",
    "tail_13", "tail_14", "tail_15", "tail_16",
];

/// Species-average physical parameters for the Moluun cub tail. Tuned
/// 2026-05-11 so the rest pose recovers between mood-driven flicks
/// instead of accumulating drift, while still letting Playful read as
/// noticeably more energetic than Happy.
pub const TAIL_SEGMENT_MASS: f32 = 0.05;        // kg — thin cub tail segment
pub const TAIL_LIGAMENT_K: f32 = 4.5;           // N·px/rad — restoring spring
pub const TAIL_FRICTION: f32 = 1.2;             // N·px·s/rad — wet damping
pub const TAIL_ROM: f32 = std::f32::consts::FRAC_PI_4; // ±45° per joint

#[allow(dead_code)]
pub fn cub_tail_skeleton_standalone() -> Skeleton {
    let seg_len = STANDALONE_TAIL_LENGTH / STANDALONE_TAIL_SEGMENTS as f32;

    let mut bones: Vec<Bone> = Vec::with_capacity(STANDALONE_TAIL_SEGMENTS + 1);
    // Length-0 root anchors the chain to the skeleton's root_position
    // without contributing visible length. Width 0 because brushes draw
    // the tail from the joint polyline, not from per-bone widths.
    bones.push(Bone::root("tail_base", 0.0, 0.0));
    for (i, name) in STANDALONE_TAIL_BONE_NAMES.iter().enumerate() {
        let parent = BoneId(i as u16); // previous bone in the chain
        bones.push(
            Bone::child(name, parent, Vec2::ZERO, 0.0, seg_len, 1.0)
                .with_stiffness(Stiffness::Soft)
                .with_tissue(BoneTissue::new(TAIL_SEGMENT_MASS)),
        );
    }

    let mut sk = Skeleton::new(bones);
    // Place the tail base on the left margin of a 128×128 canvas, vertically
    // centred — rest pose extends in +x across the canvas, leaving margin
    // on every side for visual review.
    sk.set_root(Vec2::new(28.0, 64.0));
    sk
}

/// Same as `cub_tail_skeleton_standalone` but with a Hinge joint installed
/// on every segment using species-average ligament/friction/ROM. After
/// this call, `Skeleton::step_physics` can drive the segments physically
/// from applied torques (muscle, gravity, contact).
#[allow(dead_code)]
pub fn cub_tail_skeleton_physical() -> Skeleton {
    cub_tail_skeleton_for_genes(&crate::genome::TailGenes::default())
}

/// Concrete physical parameters of a Moluun cub tail, derived from a
/// per-creature `TailGenes`. Keeps the gene-to-physics mapping in one
/// place so the rig builder and the actuator builder agree.
#[derive(Copy, Clone, Debug)]
pub struct CubTailPhysiology {
    pub total_length: f32,
    pub segment_mass: f32,
    pub ligament_k: f32,
    pub friction: f32,
    pub rom: f32,
    pub muscle_max_force: f32,
    pub muscle_contraction_rate: f32,
}

impl CubTailPhysiology {
    /// Map a `TailGenes` to the cub tail's physical parameters. The
    /// formulas are species-local: a Moluun's "flexibility = 1" means
    /// roughly half the species's baseline ligament stiffness. Other
    /// species will write their own mappings against the same gene.
    pub fn from_genes(g: &crate::genome::TailGenes) -> Self {
        // Length spans ±20% around the species mean.
        let total_length = STANDALONE_TAIL_LENGTH * (0.8 + 0.4 * g.length);

        // Flexibility 0..1 → ligament_k from 2×baseline (stiff) down to
        // 0.5×baseline (floppy). ROM scales 70%..130% of the baseline so
        // floppier tails can also bend wider.
        let ligament_k = TAIL_LIGAMENT_K * (2.0 - 1.5 * g.flexibility);
        let rom = TAIL_ROM * (0.7 + 0.6 * g.flexibility);

        // Strength 0..1 → muscle force from 30% to 130% of baseline,
        // and contraction rate from 4..12 (slower vs snappier responses).
        let muscle_max_force = 3.0 * (0.3 + g.strength);
        let muscle_contraction_rate = 4.0 + 8.0 * g.strength;

        // Mass and friction stay close to baseline for now — they're
        // shared body properties more than tail-specific ones.
        Self {
            total_length,
            segment_mass: TAIL_SEGMENT_MASS,
            ligament_k,
            friction: TAIL_FRICTION,
            rom,
            muscle_max_force,
            muscle_contraction_rate,
        }
    }
}

/// Build the cub tail skeleton from a `TailGenes`. Mirrors
/// `cub_tail_skeleton_standalone` but uses gene-derived segment length
/// and joint parameters.
#[allow(dead_code)]
pub fn cub_tail_skeleton_for_genes(genes: &crate::genome::TailGenes) -> Skeleton {
    let phys = CubTailPhysiology::from_genes(genes);
    let seg_len = phys.total_length / STANDALONE_TAIL_SEGMENTS as f32;

    let mut bones: Vec<Bone> = Vec::with_capacity(STANDALONE_TAIL_SEGMENTS + 1);
    bones.push(Bone::root("tail_base", 0.0, 0.0));
    for (i, name) in STANDALONE_TAIL_BONE_NAMES.iter().enumerate() {
        let parent = BoneId(i as u16);
        bones.push(
            Bone::child(name, parent, Vec2::ZERO, 0.0, seg_len, 1.0)
                .with_stiffness(Stiffness::Soft)
                .with_tissue(BoneTissue::new(phys.segment_mass)),
        );
    }

    let mut sk = Skeleton::new(bones);
    sk.set_root(Vec2::new(28.0, 64.0));
    for i in 1..=STANDALONE_TAIL_SEGMENTS {
        let bone_id = BoneId(i as u16);
        sk.install_joint(
            bone_id,
            Joint::hinge(bone_id, 0.0, -phys.rom, phys.rom, phys.ligament_k, phys.friction),
        );
    }
    sk
}

/// Tail motion pattern derived from a creature's current mood. Each mood
/// has a characteristic wave period, amplitude, and bias — different
/// physical intents that the same neuromuscular machinery turns into
/// different visible motion.
///
/// All values are in 0..=1 amplitude space; the muscle layer scales them
/// to actual force via `Muscle::max_force × activation`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TailMoodPattern {
    /// Wave period in seconds — how long one full oscillation takes.
    /// `f32::INFINITY` means "no oscillation" (e.g. Sleeping or static droop).
    pub period_s: f32,
    /// Peak intent amplitude (0..=1). Higher = stronger muscle pull.
    pub amplitude: f32,
    /// Constant additive intent on the extensor side, 0..=1. Pulls the
    /// tail toward the +angle (image-space "down") direction; used by
    /// Lonely for a droop.
    pub extensor_bias: f32,
    /// If true, the wave is rectified — only its positive half drives a
    /// muscle. Produces sparse bursts of motion instead of smooth oscillation.
    pub burst_only: bool,
}

impl TailMoodPattern {
    pub const fn still() -> Self {
        Self {
            period_s: f32::INFINITY,
            amplitude: 0.0,
            extensor_bias: 0.0,
            burst_only: false,
        }
    }
}

/// Map a `MoodState` to its tail motion pattern. The choices below are
/// the species-average expression — genome flexibility/strength still
/// modulate how that pattern *looks*, but the underlying intent is
/// mood-driven.
pub fn cub_tail_pattern_for_mood(mood: &crate::mind::MoodState) -> TailMoodPattern {
    use crate::mind::MoodState;
    match mood {
        MoodState::Sleeping => TailMoodPattern::still(),
        MoodState::Sick    => TailMoodPattern { period_s: 5.0, amplitude: 0.35, extensor_bias: 0.0, burst_only: false },
        MoodState::Tired   => TailMoodPattern { period_s: 3.0, amplitude: 0.45, extensor_bias: 0.0, burst_only: false },
        MoodState::Happy   => TailMoodPattern { period_s: 1.5, amplitude: 0.65, extensor_bias: 0.0, burst_only: false },
        MoodState::Playful => TailMoodPattern { period_s: 0.7, amplitude: 0.90, extensor_bias: 0.0, burst_only: false },
        // Lonely: a sustained low extensor bias on every segment produces
        // a downward droop — no oscillation, just a held posture.
        MoodState::Lonely  => TailMoodPattern { period_s: f32::INFINITY, amplitude: 0.0, extensor_bias: 0.35, burst_only: false },
        // Hungry / Thirsty share the impatient-bursts pattern.
        MoodState::Hungry | MoodState::Thirsty =>
            TailMoodPattern { period_s: 2.0, amplitude: 0.75, extensor_bias: 0.0, burst_only: true },
    }
}

/// Per-segment `PairIntent` for a Moluun cub tail at simulation time `t`,
/// driven by a `TailMoodPattern`. The wave travels base→tip with one
/// full wavelength along the chain (segments offset by `2π/segments`).
pub fn cub_tail_intent(
    pattern: &TailMoodPattern,
    t: f32,
    seg: usize,
    segments: usize,
) -> kokoro_body::actuation::PairIntent {
    use kokoro_body::actuation::PairIntent;
    use std::f32::consts::TAU;

    // Always present: extensor DC bias from the pattern (droop).
    let bias = pattern.extensor_bias.clamp(0.0, 1.0);

    // Oscillating component, suppressed when period is infinite.
    let osc = if pattern.period_s.is_finite() && pattern.amplitude > 0.0 {
        let phase = TAU * (t / pattern.period_s - seg as f32 / segments.max(1) as f32);
        let mut drive = phase.sin() * pattern.amplitude;
        if pattern.burst_only {
            // Rectify so only positive half drives the extensor; flexor
            // never fires under burst_only patterns.
            drive = drive.max(0.0);
        }
        drive
    } else {
        0.0
    };

    if osc >= 0.0 {
        PairIntent::new(0.0, (osc + bias).clamp(0.0, 1.0))
    } else {
        // Negative oscillation → flexor; extensor still holds the bias.
        PairIntent::new((-osc).clamp(0.0, 1.0), bias)
    }
}

/// Build a cub tail body with explicit canvas-scaled length, attach
/// position, and base orientation. Used by the in-game integration to
/// fit the tail to the 64×64 sprite canvas; the demo builder below uses
/// the species-default parameters for the 128×128 snapshot canvas.
#[allow(dead_code)]
pub fn cub_tail_body_for_creature(
    genes: &crate::genome::TailGenes,
    total_length: f32,
    attach: Vec2,
    base_angle: f32,
) -> kokoro_body::Body {
    use kokoro_body::body::Actuator;
    use kokoro_body::{Body, Muscle, MuscleAttachment, MusclePair, Nerve};

    let phys = CubTailPhysiology::from_genes(genes);
    let seg_len = total_length / STANDALONE_TAIL_SEGMENTS as f32;

    let mut bones: Vec<Bone> = Vec::with_capacity(STANDALONE_TAIL_SEGMENTS + 1);
    bones.push(Bone::root("tail_base", 0.0, 0.0));
    for (i, name) in STANDALONE_TAIL_BONE_NAMES.iter().enumerate() {
        let parent = BoneId(i as u16);
        // First segment carries `base_angle` so the whole chain orients
        // the way the caller wants (e.g. PI = "tail faces left").
        let rest_angle = if i == 0 { base_angle } else { 0.0 };
        bones.push(
            Bone::child(name, parent, Vec2::ZERO, rest_angle, seg_len, 1.0)
                .with_stiffness(Stiffness::Soft)
                .with_tissue(BoneTissue::new(phys.segment_mass)),
        );
    }

    let mut sk = Skeleton::new(bones);
    sk.set_root(attach);
    for i in 1..=STANDALONE_TAIL_SEGMENTS {
        let bone_id = BoneId(i as u16);
        let rest_angle = if i == 1 { base_angle } else { 0.0 };
        sk.install_joint(
            bone_id,
            Joint::hinge(bone_id, rest_angle, -phys.rom, phys.rom, phys.ligament_k, phys.friction),
        );
    }

    let mut body = Body::new(sk);
    for i in 1..=STANDALONE_TAIL_SEGMENTS {
        let bone_id = BoneId(i as u16);
        // Real tails taper base → tip. Same rest_thickness for both sides
        // of the antagonist pair: a healthy cub's tail is radially
        // symmetric at rest. Strength gene scales the whole tube up/down.
        let taper = cub_tail_rest_thickness(i, STANDALONE_TAIL_SEGMENTS, &genes.strength);
        let mk_muscle = |name: &'static str| {
            let mut m = Muscle::new(
                name,
                MuscleAttachment::new(BoneId((i - 1) as u16), 1.0),
                MuscleAttachment::new(bone_id, 1.0),
                phys.muscle_max_force,
            );
            m.contraction_rate = phys.muscle_contraction_rate;
            m.rest_thickness = taper;
            m
        };
        let muscles = MusclePair::new(mk_muscle("flexor"), mk_muscle("extensor"));
        let nerve_flexor = Nerve::new("nerve_flexor", 20.0, 1.0);
        let nerve_extensor = Nerve::new("nerve_extensor", 20.0, 1.0);
        body.attach_actuator(Actuator::new(bone_id, nerve_flexor, nerve_extensor, muscles));
    }
    body
}

/// Tapered rest cross-section for muscle pair `seg` of a cub tail, in
/// canvas pixels. Linear taper from `BASE_THICK_PX` at the body to
/// `TIP_THICK_PX` at the last segment; the `strength` gene scales the
/// whole curve so a strong cub has a chunkier tail.
fn cub_tail_rest_thickness(seg: usize, segments: usize, strength: &f32) -> f32 {
    // Half-widths in canvas pixels for the *muscle/skin core* of the
    // tail. The bushy fusiform silhouette is produced by the fur layer
    // sitting on top — the anatomy itself is a slim cone, exactly like
    // the bones-and-muscle profile of a real red panda's tail.
    const BASE_THICK_PX: f32 = 1.2;
    const TIP_THICK_PX:  f32 = 0.5;
    let t = ((seg.saturating_sub(1)) as f32) / ((segments - 1).max(1) as f32);
    let interp = BASE_THICK_PX * (1.0 - t) + TIP_THICK_PX * t;
    // Strength gene 0..1 → 0.75x .. 1.25x scaling on the whole tube.
    interp * (0.75 + 0.5 * strength.clamp(0.0, 1.0))
}

/// Build a full neuromuscular `Body` for the cub tail using a `TailGenes`.
/// Includes per-joint antagonist muscle pair + two nerves, all sized from
/// the gene-derived physiology.
#[allow(dead_code)]
pub fn cub_tail_body_for_genes(genes: &crate::genome::TailGenes) -> kokoro_body::Body {
    use kokoro_body::body::Actuator;
    use kokoro_body::{Body, Muscle, MuscleAttachment, MusclePair, Nerve};

    let phys = CubTailPhysiology::from_genes(genes);
    let skeleton = cub_tail_skeleton_for_genes(genes);
    let mut body = Body::new(skeleton);

    for i in 1..=STANDALONE_TAIL_SEGMENTS {
        let bone_id = BoneId(i as u16);
        let taper = cub_tail_rest_thickness(i, STANDALONE_TAIL_SEGMENTS, &genes.strength);
        let mk_muscle = |name: &'static str| {
            let mut m = Muscle::new(
                name,
                MuscleAttachment::new(BoneId((i - 1) as u16), 1.0),
                MuscleAttachment::new(bone_id, 1.0),
                phys.muscle_max_force,
            );
            m.contraction_rate = phys.muscle_contraction_rate;
            m.rest_thickness = taper;
            m
        };
        let muscles = MusclePair::new(mk_muscle("flexor"), mk_muscle("extensor"));
        let nerve_flexor = Nerve::new("nerve_flexor", 20.0, 1.0);
        let nerve_extensor = Nerve::new("nerve_extensor", 20.0, 1.0);
        body.attach_actuator(Actuator::new(bone_id, nerve_flexor, nerve_extensor, muscles));
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn skeleton_has_19_bones() {
        // 1 pelvis + 3 spine/neck/head + 2 ears + 2 face + 3 front leg
        // + 3 back leg + 5 tail = 19.
        assert_eq!(cub_skeleton().len(), 19);
    }

    #[test]
    fn key_bones_resolvable_by_name() {
        let sk = cub_skeleton();
        for name in [
            "pelvis", "spine", "neck", "head", "ear_front", "ear_back",
            "eye", "snout",
            "shoulder_front", "leg_front", "paw_front",
            "hip_back", "leg_back", "paw_back",
            "tail_1", "tail_2", "tail_3", "tail_4", "tail_5",
        ] {
            assert!(sk.id_of(name).is_some(), "missing bone '{name}'");
        }
    }

    #[test]
    fn head_sits_to_the_right_of_pelvis() {
        let mut sk = cub_skeleton();
        sk.forward();
        let pelvis = sk.world_base(sk.id_of("pelvis").unwrap());
        let head_tip = sk.world_tip(sk.id_of("head").unwrap());
        assert!(
            head_tip.x > pelvis.x + 15.0,
            "head tip {head_tip:?} should be well to the right of pelvis {pelvis:?}"
        );
    }

    #[test]
    fn tail_extends_to_the_left() {
        let mut sk = cub_skeleton();
        sk.forward();
        let pelvis = sk.world_base(sk.id_of("pelvis").unwrap());
        let tail_tip = sk.world_tip(sk.id_of("tail_5").unwrap());
        assert!(
            tail_tip.x < pelvis.x - 10.0,
            "tail tip {tail_tip:?} should be well to the left of pelvis {pelvis:?}"
        );
    }

    #[test]
    fn legs_hang_below_body() {
        let mut sk = cub_skeleton();
        sk.forward();
        let pelvis_y = sk.world_base(sk.id_of("pelvis").unwrap()).y;
        for leg in ["paw_front", "paw_back"] {
            let paw_tip = sk.world_tip(sk.id_of(leg).unwrap());
            assert!(paw_tip.y > pelvis_y + 6.0, "{leg} tip should sit below pelvis");
        }
    }

    #[test]
    fn front_leg_sits_forward_of_back_leg() {
        let mut sk = cub_skeleton();
        sk.forward();
        let front = sk.world_base(sk.id_of("shoulder_front").unwrap());
        let back = sk.world_base(sk.id_of("hip_back").unwrap());
        assert!(
            front.x > back.x + 8.0,
            "front leg ({front:?}) should be forward of back leg ({back:?})"
        );
    }

    #[test]
    fn z_layering_separates_front_back() {
        let sk = cub_skeleton();
        let z = |name: &str| sk.bone(sk.id_of(name).unwrap()).z_layer;
        assert!(z("leg_front") > z("leg_back"));
        assert!(z("ear_front") > z("ear_back"));
        assert!(z("head") > z("spine"));
    }

    #[test]
    fn rigid_vs_soft_distribution_matches_design() {
        let sk = cub_skeleton();
        let stiff = |name: &str| sk.bone(sk.id_of(name).unwrap()).stiffness;
        // Skeleton-driven (Rigid).
        for name in ["pelvis", "spine", "head", "shoulder_front", "hip_back", "tail_1"] {
            assert_eq!(stiff(name), Stiffness::Rigid, "{name} should be Rigid");
        }
        // Soft-body-driven (Soft).
        for name in ["ear_front", "ear_back", "paw_front", "paw_back", "tail_3", "tail_5"] {
            assert_eq!(stiff(name), Stiffness::Soft, "{name} should be Soft");
        }
    }

    #[test]
    fn root_translation_moves_whole_skeleton() {
        let mut a = cub_skeleton();
        a.forward();
        let head_a = a.world_tip(a.id_of("head").unwrap());

        let mut b = cub_skeleton();
        b.set_root(Vec2::new(30.0, 40.0));
        b.forward();
        let head_b = b.world_tip(b.id_of("head").unwrap());

        assert!(approx(head_b.x - head_a.x, 8.0, 1e-3));
        assert!(approx(head_b.y - head_a.y, 4.0, 1e-3));
    }

    // ---------------------------------------------------------------
    // Standalone tail skeleton — the per-part redesign target.
    // ---------------------------------------------------------------

    // ---------------------------------------------------------------
    // Mood → tail intent pattern
    // ---------------------------------------------------------------

    #[test]
    fn sleeping_pattern_is_silent() {
        use crate::mind::MoodState;
        let p = cub_tail_pattern_for_mood(&MoodState::Sleeping);
        assert_eq!(p.amplitude, 0.0);
        assert_eq!(p.extensor_bias, 0.0);
        // Verify intent is rest at any time and any segment.
        let i = cub_tail_intent(&p, 1.7, 5, 16);
        assert_eq!(i.flexor, 0.0);
        assert_eq!(i.extensor, 0.0);
    }

    #[test]
    fn playful_has_faster_period_than_happy() {
        use crate::mind::MoodState;
        let happy = cub_tail_pattern_for_mood(&MoodState::Happy);
        let playful = cub_tail_pattern_for_mood(&MoodState::Playful);
        assert!(playful.period_s < happy.period_s);
        assert!(playful.amplitude > happy.amplitude);
    }

    #[test]
    fn lonely_droops_via_extensor_bias_only() {
        use crate::mind::MoodState;
        let p = cub_tail_pattern_for_mood(&MoodState::Lonely);
        assert!(p.extensor_bias > 0.0);
        assert_eq!(p.amplitude, 0.0);
        let i = cub_tail_intent(&p, 0.0, 0, 16);
        assert!(i.extensor > 0.0);
        assert_eq!(i.flexor, 0.0);
    }

    #[test]
    fn hungry_pattern_is_burst_only() {
        use crate::mind::MoodState;
        let p = cub_tail_pattern_for_mood(&MoodState::Hungry);
        assert!(p.burst_only);
        // Sample the wave across a full period: flexor should never fire.
        for k in 0..40 {
            let t = (k as f32 / 40.0) * p.period_s;
            let i = cub_tail_intent(&p, t, 8, 16);
            assert_eq!(i.flexor, 0.0, "flexor fired at t={t}");
        }
    }

    #[test]
    fn standalone_tail_has_seventeen_bones() {
        // 1 length-0 root + 16 articulated segments.
        assert_eq!(cub_tail_skeleton_standalone().len(), 17);
    }

    #[test]
    fn standalone_tail_total_length_matches_spec() {
        let mut sk = cub_tail_skeleton_standalone();
        sk.forward();
        let root = sk.world_base(BoneId(0));
        let tip = sk.world_tip(BoneId(16));
        // Last segment's tip should sit exactly STANDALONE_TAIL_LENGTH px
        // away from the root along +x (rest pose is straight).
        assert!(approx(tip.x - root.x, STANDALONE_TAIL_LENGTH, 1e-3));
        assert!(approx(tip.y - root.y, 0.0, 1e-3));
    }

    #[test]
    fn standalone_tail_segments_are_all_soft() {
        let sk = cub_tail_skeleton_standalone();
        for i in 1..=16 {
            assert_eq!(
                sk.bone(BoneId(i)).stiffness,
                Stiffness::Soft,
                "segment {i} should be Stiffness::Soft (cat-like flexibility)"
            );
        }
    }

    /// Tail-only visual snapshot for design review — see
    /// `cargo test snapshot_moluun_cub_tail -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn snapshot_moluun_cub_tail() {
        use image::{Rgba, RgbaImage};
        use kokoro_art_palette::Palette;
        use kokoro_art_palette::dsl::{Brush, FusiformTail};
        use std::path::PathBuf;

        let mut sk = cub_tail_skeleton_standalone();
        sk.forward();

        // Polyline of 17 joints: tail_base, then tip of each segment.
        let mut joints: Vec<(i32, i32)> = Vec::with_capacity(17);
        let base = sk.world_base(BoneId(0));
        joints.push((base.x.round() as i32, base.y.round() as i32));
        for i in 1..=16 {
            let tip = sk.world_tip(BoneId(i));
            joints.push((tip.x.round() as i32, tip.y.round() as i32));
        }

        // Mid-grey checker background so both the orange body and the
        // off-white rings have strong contrast (the off-white nearly
        // matches a flat cream background and the rings disappear).
        let mut img = RgbaImage::new(128, 128);
        for y in 0..128u32 {
            for x in 0..128u32 {
                let c: u8 = if ((x / 8) + (y / 8)) % 2 == 0 { 110 } else { 90 };
                img.put_pixel(x, y, Rgba([c, c, c, 255]));
            }
        }

        // Species-average tail: peak half-width 5px (10px diameter at
        // the belly), rings on every other joint, 3px-wide bands.
        FusiformTail::new(joints, 5, Palette::Orange, Palette::OffWhite)
            .with_rings(2, 1)
            .paint(&mut img);

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("sprite-snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        save_with_4x(&img, &dir, "moluun_cub_tail");

        eprintln!(
            "moluun_cub_tail: wrote {} (and @4x preview)",
            dir.join("moluun_cub_tail.png").display()
        );
    }

    /// Tail-in-motion sprite strip — eight phases of an S-wave travelling
    /// along the 16-segment chain. Each frame uses the same wave amplitude
    /// and wavelength; only the phase advances. Run with
    /// `cargo test snapshot_moluun_cub_tail_motion -- --ignored --nocapture`.
    ///
    /// Per-segment angle = AMPLITUDE × sin(2π × seg_index / SEGMENTS + phase).
    /// With one full wave along the tail this produces a clear S shape; the
    /// phase shift between frames slides the S from base toward tip,
    /// reproducing the visual of a cat tail flicking in S waves.
    #[test]
    #[ignore]
    fn snapshot_moluun_cub_tail_motion() {
        use image::{Rgba, RgbaImage};
        use kokoro_art_palette::Palette;
        use kokoro_art_palette::dsl::{Brush, FusiformTail};
        use std::f32::consts::TAU;
        use std::path::PathBuf;

        const SEGMENTS: usize = 16;
        const FRAMES: usize = 8;
        const AMPLITUDE: f32 = 0.18; // rad per segment at wave peak
        const WAVELENGTHS: f32 = 1.0; // one full S along the tail

        let frame_w: u32 = 128;
        let frame_h: u32 = 128;
        let strip_w: u32 = frame_w * FRAMES as u32;
        let mut strip = RgbaImage::new(strip_w, frame_h);
        for y in 0..frame_h {
            for x in 0..strip_w {
                let c: u8 = if ((x / 8) + (y / 8)) % 2 == 0 { 110 } else { 90 };
                strip.put_pixel(x, y, Rgba([c, c, c, 255]));
            }
        }

        for f in 0..FRAMES {
            let phase = TAU * f as f32 / FRAMES as f32;

            let mut sk = cub_tail_skeleton_standalone();
            for seg in 0..SEGMENTS {
                let s = seg as f32 / SEGMENTS as f32; // 0..1 along the chain
                let delta = AMPLITUDE * (WAVELENGTHS * TAU * s + phase).sin();
                sk.set_angle(BoneId((seg + 1) as u16), delta);
            }
            sk.forward();

            let mut joints: Vec<(i32, i32)> = Vec::with_capacity(17);
            let base = sk.world_base(BoneId(0));
            joints.push((base.x.round() as i32, base.y.round() as i32));
            for i in 1..=16 {
                let tip = sk.world_tip(BoneId(i));
                joints.push((tip.x.round() as i32, tip.y.round() as i32));
            }

            let mut frame_img = RgbaImage::new(frame_w, frame_h);
            FusiformTail::new(joints, 5, Palette::Orange, Palette::OffWhite)
                .with_rings(2, 1)
                .paint(&mut frame_img);

            // Blit non-transparent pixels onto the strip — preserves the
            // checker behind the tail outline.
            let off_x = f as u32 * frame_w;
            for y in 0..frame_h {
                for x in 0..frame_w {
                    let p = *frame_img.get_pixel(x, y);
                    if p.0[3] > 0 {
                        strip.put_pixel(off_x + x, y, p);
                    }
                }
            }
        }

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("sprite-snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        save_with_4x(&strip, &dir, "moluun_cub_tail_motion");

        eprintln!(
            "moluun_cub_tail_motion: wrote {}-phase S-wave strip to {}",
            FRAMES,
            dir.join("moluun_cub_tail_motion.png").display()
        );
    }

    /// End-to-end physical motion snapshot. Builds the cub tail with its
    /// joints, installs an antagonist muscle pair + two nerves per joint,
    /// then drives a traveling sinusoidal intent along the chain and
    /// integrates physics. The S-shape is NOT authored — it emerges from
    /// the actuator → muscle → joint → integrator pipeline. Run with
    /// `cargo test snapshot_moluun_cub_tail_physical -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn snapshot_moluun_cub_tail_physical() {
        use image::{Rgba, RgbaImage};
        use kokoro_art_palette::Palette;
        use kokoro_art_palette::dsl::{Brush, FusiformTail};
        use kokoro_body::{Body, Muscle, MuscleAttachment, MusclePair, Nerve};
        use kokoro_body::actuation::PairIntent;
        use kokoro_body::body::Actuator;
        use kokoro_rig::AppliedTorques;
        use std::f32::consts::TAU;
        use std::path::PathBuf;

        const FRAMES: usize = 8;
        const DT: f32 = 0.01;            // 10ms physics tick
        const TICKS_BETWEEN_FRAMES: u32 = 25; // 250ms between snapshot samples
        const WAVE_PERIOD_S: f32 = 1.2;  // one full S oscillation
        const WAVE_AMPLITUDE: f32 = 1.0; // peak intent (0..1)
        const SEGMENTS: usize = STANDALONE_TAIL_SEGMENTS;

        // Build the tail body: physical skeleton + per-joint actuators.
        let skeleton = cub_tail_skeleton_physical();
        let mut body = Body::new(skeleton);
        for i in 1..=SEGMENTS {
            let bone_id = BoneId(i as u16);
            let mk_muscle = |name: &'static str| {
                Muscle::new(
                    name,
                    MuscleAttachment::new(BoneId((i - 1) as u16), 1.0),
                    MuscleAttachment::new(bone_id, 1.0),
                    3.0, // max_force per muscle
                )
            };
            let muscles = MusclePair::new(mk_muscle("flexor"), mk_muscle("extensor"));
            let nerve_flexor = Nerve::new("nerve_flexor", 20.0, 1.0);
            let nerve_extensor = Nerve::new("nerve_extensor", 20.0, 1.0);
            body.attach_actuator(Actuator::new(bone_id, nerve_flexor, nerve_extensor, muscles));
        }

        let frame_w: u32 = 128;
        let frame_h: u32 = 128;
        let strip_w: u32 = frame_w * FRAMES as u32;
        let mut strip = RgbaImage::new(strip_w, frame_h);
        for y in 0..frame_h {
            for x in 0..strip_w {
                let c: u8 = if ((x / 8) + (y / 8)) % 2 == 0 { 110 } else { 90 };
                strip.put_pixel(x, y, Rgba([c, c, c, 255]));
            }
        }

        let mut sim_time = 0.0_f32;
        for f in 0..FRAMES {
            for _ in 0..TICKS_BETWEEN_FRAMES {
                let t = sim_time;
                body.step(
                    DT,
                    |bone_id| {
                        let seg = bone_id.0 as usize;
                        if !(1..=SEGMENTS).contains(&seg) {
                            return PairIntent::rest();
                        }
                        // Traveling wave of muscle drive — one full S per period,
                        // each segment lagging the previous by 2π/SEGMENTS.
                        let phase = TAU * (t / WAVE_PERIOD_S - seg as f32 / SEGMENTS as f32);
                        let drive = WAVE_AMPLITUDE * phase.sin();
                        if drive >= 0.0 {
                            PairIntent::new(0.0, drive)
                        } else {
                            PairIntent::new(-drive, 0.0)
                        }
                    },
                    |_| AppliedTorques::default(),
                );
                sim_time += DT;
            }
            body.skeleton.forward();

            // Render this frame's tail to its own image then blit onto strip.
            let mut joints: Vec<(i32, i32)> = Vec::with_capacity(SEGMENTS + 1);
            let base = body.skeleton.world_base(BoneId(0));
            joints.push((base.x.round() as i32, base.y.round() as i32));
            for i in 1..=SEGMENTS {
                let tip = body.skeleton.world_tip(BoneId(i as u16));
                joints.push((tip.x.round() as i32, tip.y.round() as i32));
            }

            let mut frame_img = RgbaImage::new(frame_w, frame_h);
            FusiformTail::new(joints, 5, Palette::Orange, Palette::OffWhite)
                .with_rings(2, 1)
                .paint(&mut frame_img);

            let off_x = f as u32 * frame_w;
            for y in 0..frame_h {
                for x in 0..frame_w {
                    let p = *frame_img.get_pixel(x, y);
                    if p.0[3] > 0 {
                        strip.put_pixel(off_x + x, y, p);
                    }
                }
            }
        }

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("sprite-snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        save_with_4x(&strip, &dir, "moluun_cub_tail_physical");

        eprintln!(
            "moluun_cub_tail_physical: wrote {}-frame physical-sim strip to {}",
            FRAMES,
            dir.join("moluun_cub_tail_physical.png").display()
        );
    }

    /// Compare three kobaras with different `TailGenes` driven by the
    /// **exact same** sinusoidal muscle intent. The differences in motion
    /// emerge entirely from the genome → physical-parameter mapping, not
    /// from any per-creature animation tweak. Run with
    /// `cargo test snapshot_moluun_cub_tail_genome -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn snapshot_moluun_cub_tail_genome() {
        use image::{Rgba, RgbaImage};
        use kokoro_art_palette::Palette;
        use kokoro_art_palette::dsl::{Brush, FusiformTail};
        use kokoro_body::actuation::PairIntent;
        use kokoro_rig::AppliedTorques;
        use std::f32::consts::TAU;
        use std::path::PathBuf;

        const FRAMES: usize = 8;
        const DT: f32 = 0.01;
        const TICKS_BETWEEN_FRAMES: u32 = 25;
        const WAVE_PERIOD_S: f32 = 1.2;
        const SEGMENTS: usize = STANDALONE_TAIL_SEGMENTS;

        // Three kobaras: stiff/weak, average, floppy/strong.
        let kobaras: [(&str, crate::genome::TailGenes); 3] = [
            (
                "stiff",
                crate::genome::TailGenes { length: 0.5, flexibility: 0.1, strength: 0.2 },
            ),
            (
                "average",
                crate::genome::TailGenes::default(),
            ),
            (
                "floppy",
                crate::genome::TailGenes { length: 0.8, flexibility: 0.9, strength: 0.9 },
            ),
        ];

        let frame_w: u32 = 128;
        let frame_h: u32 = 128;
        let strip_w: u32 = frame_w * FRAMES as u32;
        let strip_h: u32 = frame_h * kobaras.len() as u32;
        let mut sheet = RgbaImage::new(strip_w, strip_h);
        for y in 0..strip_h {
            for x in 0..strip_w {
                let c: u8 = if ((x / 8) + (y / 8)) % 2 == 0 { 110 } else { 90 };
                sheet.put_pixel(x, y, Rgba([c, c, c, 255]));
            }
        }

        for (row, (_label, genes)) in kobaras.iter().enumerate() {
            let mut body = cub_tail_body_for_genes(genes);
            let mut sim_time = 0.0_f32;
            for f in 0..FRAMES {
                for _ in 0..TICKS_BETWEEN_FRAMES {
                    let t = sim_time;
                    body.step(
                        DT,
                        |bone_id| {
                            let seg = bone_id.0 as usize;
                            if !(1..=SEGMENTS).contains(&seg) {
                                return PairIntent::rest();
                            }
                            let phase = TAU
                                * (t / WAVE_PERIOD_S - seg as f32 / SEGMENTS as f32);
                            let drive = phase.sin();
                            if drive >= 0.0 {
                                PairIntent::new(0.0, drive)
                            } else {
                                PairIntent::new(-drive, 0.0)
                            }
                        },
                        |_| AppliedTorques::default(),
                    );
                    sim_time += DT;
                }
                body.skeleton.forward();

                let mut joints: Vec<(i32, i32)> = Vec::with_capacity(SEGMENTS + 1);
                let base = body.skeleton.world_base(BoneId(0));
                joints.push((base.x.round() as i32, base.y.round() as i32));
                for i in 1..=SEGMENTS {
                    let tip = body.skeleton.world_tip(BoneId(i as u16));
                    joints.push((tip.x.round() as i32, tip.y.round() as i32));
                }

                let mut frame_img = RgbaImage::new(frame_w, frame_h);
                FusiformTail::new(joints, 5, Palette::Orange, Palette::OffWhite)
                    .with_rings(2, 1)
                    .paint(&mut frame_img);

                let off_x = f as u32 * frame_w;
                let off_y = row as u32 * frame_h;
                for y in 0..frame_h {
                    for x in 0..frame_w {
                        let p = *frame_img.get_pixel(x, y);
                        if p.0[3] > 0 {
                            sheet.put_pixel(off_x + x, off_y + y, p);
                        }
                    }
                }
            }
        }

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("sprite-snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        save_with_4x(&sheet, &dir, "moluun_cub_tail_genome");

        eprintln!(
            "moluun_cub_tail_genome: wrote {}×{} sheet (3 kobaras × 8 frames) to {}",
            strip_w,
            strip_h,
            dir.join("moluun_cub_tail_genome.png").display()
        );
    }

    /// Six moods × six frames sheet. Same average-genome kobara; the
    /// motion difference comes entirely from `cub_tail_pattern_for_mood`.
    /// Run with
    /// `cargo test snapshot_moluun_cub_tail_moods -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn snapshot_moluun_cub_tail_moods() {
        use crate::mind::MoodState;
        use image::{Rgba, RgbaImage};
        use kokoro_art_palette::Palette;
        use kokoro_art_palette::dsl::{Brush, FusiformTail};
        use kokoro_rig::AppliedTorques;
        use std::path::PathBuf;

        const FRAMES: usize = 6;
        const DT: f32 = 0.01;
        const TICKS_BETWEEN_FRAMES: u32 = 30; // 300ms between samples
        const SEGMENTS: usize = STANDALONE_TAIL_SEGMENTS;

        let moods: [MoodState; 6] = [
            MoodState::Sleeping,
            MoodState::Tired,
            MoodState::Happy,
            MoodState::Playful,
            MoodState::Lonely,
            MoodState::Hungry,
        ];

        let frame_w: u32 = 128;
        let frame_h: u32 = 128;
        let sheet_w: u32 = frame_w * FRAMES as u32;
        let sheet_h: u32 = frame_h * moods.len() as u32;
        let mut sheet = RgbaImage::new(sheet_w, sheet_h);
        for y in 0..sheet_h {
            for x in 0..sheet_w {
                let c: u8 = if ((x / 8) + (y / 8)) % 2 == 0 { 110 } else { 90 };
                sheet.put_pixel(x, y, Rgba([c, c, c, 255]));
            }
        }

        let average_genes = crate::genome::TailGenes::default();
        for (row, mood) in moods.iter().enumerate() {
            let pattern = cub_tail_pattern_for_mood(mood);
            let mut body = cub_tail_body_for_genes(&average_genes);
            let mut sim_time = 0.0_f32;
            for f in 0..FRAMES {
                for _ in 0..TICKS_BETWEEN_FRAMES {
                    let t = sim_time;
                    body.step(
                        DT,
                        |bone_id| {
                            let seg = bone_id.0 as usize;
                            if !(1..=SEGMENTS).contains(&seg) {
                                return kokoro_body::actuation::PairIntent::rest();
                            }
                            cub_tail_intent(&pattern, t, seg, SEGMENTS)
                        },
                        |_| AppliedTorques::default(),
                    );
                    sim_time += DT;
                }
                body.skeleton.forward();

                let mut joints: Vec<(i32, i32)> = Vec::with_capacity(SEGMENTS + 1);
                let base = body.skeleton.world_base(BoneId(0));
                joints.push((base.x.round() as i32, base.y.round() as i32));
                for i in 1..=SEGMENTS {
                    let tip = body.skeleton.world_tip(BoneId(i as u16));
                    joints.push((tip.x.round() as i32, tip.y.round() as i32));
                }

                let mut frame_img = RgbaImage::new(frame_w, frame_h);
                FusiformTail::new(joints, 5, Palette::Orange, Palette::OffWhite)
                    .with_rings(2, 1)
                    .paint(&mut frame_img);

                let off_x = f as u32 * frame_w;
                let off_y = row as u32 * frame_h;
                for y in 0..frame_h {
                    for x in 0..frame_w {
                        let p = *frame_img.get_pixel(x, y);
                        if p.0[3] > 0 {
                            sheet.put_pixel(off_x + x, off_y + y, p);
                        }
                    }
                }
            }
        }

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("sprite-snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        save_with_4x(&sheet, &dir, "moluun_cub_tail_moods");

        eprintln!(
            "moluun_cub_tail_moods: wrote 6 moods × 6 frames to {}",
            dir.join("moluun_cub_tail_moods.png").display()
        );
    }

    /// Visualise the skeleton — see `cargo test snapshot_moluun_cub_skeleton
    /// -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn snapshot_moluun_cub_skeleton() {
        use image::{Rgba, RgbaImage};
        use kokoro_rig::Bone;
        use std::path::PathBuf;

        let mut sk = cub_skeleton();
        sk.forward();

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("sprite-snapshots");
        std::fs::create_dir_all(&dir).unwrap();

        // Sticks
        let mut sticks = RgbaImage::new(64, 64);
        for px in sticks.pixels_mut() {
            *px = Rgba([217, 199, 174, 255]);
        }
        kokoro_rig::debug::render_skeleton_overlay(
            &mut sticks,
            &sk,
            Rgba([59, 36, 24, 255]),
            Rgba([217, 13, 67, 255]),
            true,
        );
        save_with_4x(&sticks, &dir, "moluun_cub_skeleton");

        // Tinted silhouette
        let mut silh = RgbaImage::new(64, 64);
        for px in silh.pixels_mut() {
            *px = Rgba([217, 199, 174, 255]);
        }
        let color_for = |bone: &Bone| -> Rgba<u8> {
            match bone.name {
                "head" | "neck" | "spine" => Rgba([160, 76, 3, 255]), // OrangeDark
                "ear_front" | "ear_back" => Rgba([240, 136, 40, 255]),
                "eye" => Rgba([27, 19, 13, 255]),
                "snout" => Rgba([27, 19, 13, 255]),
                "tail_1" | "tail_2" | "tail_3" | "tail_4" | "tail_5" => {
                    Rgba([217, 103, 4, 255]) // Orange
                }
                "shoulder_front" | "leg_front" | "paw_front"
                | "hip_back" | "leg_back" | "paw_back" => Rgba([90, 54, 34, 255]), // BrownDark
                _ => Rgba([0, 0, 0, 0]),
            }
        };
        kokoro_rig::debug::render_skeleton_silhouette(&mut silh, &sk, color_for);
        save_with_4x(&silh, &dir, "moluun_cub_silhouette");

        eprintln!("moluun_cub: wrote skeleton + silhouette PNGs");
    }

    fn save_with_4x(img: &image::RgbaImage, dir: &std::path::Path, name: &str) {
        img.save(dir.join(format!("{name}.png"))).unwrap();
        let (w, h) = (img.width(), img.height());
        let mut up = image::RgbaImage::new(w * 4, h * 4);
        for y in 0..h {
            for x in 0..w {
                let p = *img.get_pixel(x, y);
                for dy in 0..4 {
                    for dx in 0..4 {
                        up.put_pixel(x * 4 + dx, y * 4 + dy, p);
                    }
                }
            }
        }
        up.save(dir.join(format!("{name}@4x.png"))).unwrap();
    }
}
