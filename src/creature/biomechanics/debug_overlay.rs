//! Debug visualization of the biomechanics layers for a `Body`.
//!
//! Each function paints **one physically-real layer** of the simulation
//! over the pixel buffer. Layers that don't have per-segment physical
//! state yet (nerves, fat, skin) are intentionally absent — they will be
//! added here once the universal physical laws that govern them land in
//! `kokoro-rig` / `kokoro-body`.
//!
//! Layers painted today (every one corresponds to real state in the sim):
//!
//! - **`paint_bones`** — every bone, as an opaque NearBlack 1-pixel line.
//!   Reads `Skeleton::world_base`/`world_tip` directly.
//! - **`paint_joints`** — joint dots coloured by how close the current
//!   angle is to the ROM limit. Reads `Joint::range_min/max` +
//!   `JointState::angle`. Stress 0 = at rest (CyanBright), 1 = at ROM
//!   cap (Red). Fully opaque so the colour reads clearly.
//! - **`paint_muscles`** — for each active segment, an opaque CoralPink
//!   2×2 marker on the perp side that the mind is *currently firing
//!   above threshold*. The opposite (resting) side stays bare. This way
//!   the overlay only shows muscles that are actually doing work, so
//!   actual bone motion remains visible between markers.
//!
//! All paints are OPAQUE (no alpha blending) so colours stay vivid and
//! crisp at the 64×64 native canvas resolution.

use image::{Rgba, RgbaImage};
use kokoro_art_palette::Palette;
use kokoro_body::actuation::PairIntent;
use kokoro_rig::{BoneId, Skeleton};

// --- Palette anchors -------------------------------------------------------

const BONE_COLOR:       Rgba<u8> = Rgba(Palette::NearBlack.rgba(255));
const JOINT_REST_RGB:   [u8; 3]  = Palette::CyanBright.rgb();
const JOINT_STRESS_RGB: [u8; 3]  = Palette::Red.rgb();

/// Muscle intent threshold below which we don't paint at all. The
/// opposite-side muscle in an antagonist pair almost always sits at
/// 0–0.1 in a normal wave; cutting that off keeps the overlay honest
/// (only firing muscles paint) and stops the at-rest muscles from
/// hiding bone motion.
const MUSCLE_INTENT_THRESHOLD: f32 = 0.15;
/// Perpendicular distance from the bone axis at which the muscle marker
/// is drawn. The `FusiformTail` brush is 3 px wide (half-width ≈ 1.5),
/// so 2.5 px places the marker just outside the painted skin envelope
/// where it is clearly readable.
const MUSCLE_PERP_OFFSET: f32 = 2.5;

// --- Public layer entry points --------------------------------------------

/// Paints every bone in `skeleton` as a 1-pixel line. Caller must have
/// run `Skeleton::forward` already; if the dirty flag is set the function
/// is a no-op (we'd paint stale positions, which would lie).
pub fn paint_bones(img: &mut RgbaImage, skeleton: &Skeleton) {
    if skeleton.dirty() {
        return;
    }
    for (i, _bone) in skeleton.bones().iter().enumerate() {
        let id = BoneId(i as u16);
        let (x0, y0) = round(skeleton.world_base(id));
        let (x1, y1) = round(skeleton.world_tip(id));
        draw_line(img, x0, y0, x1, y1, BONE_COLOR);
    }
}

/// Paints a coloured dot at each joint endpoint. Colour interpolates from
/// `Palette::CyanBright` at rest to `Palette::Red` at the ROM cap, so the
/// dot visibly reddens as the joint approaches its hard limit — the
/// physical stress reading the integrator already sees every frame.
pub fn paint_joints(img: &mut RgbaImage, skeleton: &Skeleton) {
    if skeleton.dirty() {
        return;
    }
    for (i, _bone) in skeleton.bones().iter().enumerate() {
        let id = BoneId(i as u16);
        let stress = joint_stress(skeleton, id);
        let color = lerp_rgb(JOINT_REST_RGB, JOINT_STRESS_RGB, stress);
        let (x0, y0) = round(skeleton.world_base(id));
        let (x1, y1) = round(skeleton.world_tip(id));
        draw_joint_dot(img, x0, y0, color);
        draw_joint_dot(img, x1, y1, color);
    }
}

/// Paints the muscle layer for every active segment: an opaque CoralPink
/// 2×2 marker on the side currently firing above threshold. The side at
/// rest is intentionally not painted, so the visual is sparse and bone
/// motion shows through.
///
/// `intents[i]` corresponds to bone `i + 1` (bone 0 is the anchor root).
pub fn paint_muscles(img: &mut RgbaImage, skeleton: &Skeleton, intents: &[PairIntent]) {
    if skeleton.dirty() {
        return;
    }
    let [r, g, b] = Palette::CoralPink.rgb();
    let color = Rgba([r, g, b, 255]);
    let segment_count = intents.len().min(skeleton.len().saturating_sub(1));
    for seg in 0..segment_count {
        let id = BoneId((seg + 1) as u16);
        let base = skeleton.world_base(id);
        let tip = skeleton.world_tip(id);
        let dx = tip.x - base.x;
        let dy = tip.y - base.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.5 {
            continue; // degenerate segment — skip
        }
        // Unit perpendicular vector (rotate 90°). +perp side hosts the
        // flexor belly, -perp side hosts the extensor belly.
        let nx = -dy / len;
        let ny = dx / len;
        let mid_x = (base.x + tip.x) * 0.5;
        let mid_y = (base.y + tip.y) * 0.5;

        let flexor = intents[seg].flexor;
        let extensor = intents[seg].extensor;

        if flexor > MUSCLE_INTENT_THRESHOLD {
            paint_marker(
                img,
                mid_x + nx * MUSCLE_PERP_OFFSET,
                mid_y + ny * MUSCLE_PERP_OFFSET,
                color,
            );
        }
        if extensor > MUSCLE_INTENT_THRESHOLD {
            paint_marker(
                img,
                mid_x - nx * MUSCLE_PERP_OFFSET,
                mid_y - ny * MUSCLE_PERP_OFFSET,
                color,
            );
        }
    }
}

// --- Helpers ---------------------------------------------------------------

/// Stress = how close the joint is to its hard ROM cap, in `[0, 1]`.
/// At 0 the joint sits at `rest_angle`; at 1 it has reached either
/// `range_min` or `range_max` and would push back next frame.
fn joint_stress(skeleton: &Skeleton, id: BoneId) -> f32 {
    let Some(joint) = skeleton.joint(id) else { return 0.0; };
    if !joint.is_active() {
        return 0.0;
    }
    let Some(state) = skeleton.joint_state(id) else { return 0.0; };
    let delta = state.angle - joint.rest_angle;
    let span = if delta >= 0.0 {
        joint.range_max.max(1e-4)
    } else {
        joint.range_min.abs().max(1e-4)
    };
    (delta.abs() / span).clamp(0.0, 1.0)
}

/// Paints a 2×2 opaque square centred on the nearest pixel to `(cx, cy)`.
/// Compact, sharp, easy to spot on a 64×64 canvas.
fn paint_marker(img: &mut RgbaImage, cx: f32, cy: f32, color: Rgba<u8>) {
    let x = cx.round() as i32;
    let y = cy.round() as i32;
    put(img, x,     y,     color);
    put(img, x + 1, y,     color);
    put(img, x,     y + 1, color);
    put(img, x + 1, y + 1, color);
}

fn draw_line(img: &mut RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba<u8>) {
    let (mut x, mut y) = (x0, y0);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        put(img, x, y, color);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

fn draw_joint_dot(img: &mut RgbaImage, cx: i32, cy: i32, color: Rgba<u8>) {
    put(img, cx,     cy,     color);
    put(img, cx - 1, cy,     color);
    put(img, cx + 1, cy,     color);
    put(img, cx,     cy - 1, color);
    put(img, cx,     cy + 1, color);
}

fn put(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
        return;
    }
    img.put_pixel(x as u32, y as u32, color);
}

fn round(v: kokoro_rig::Vec2) -> (i32, i32) {
    (v.x.round() as i32, v.y.round() as i32)
}

fn lerp_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> Rgba<u8> {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| -> u8 {
        (x as f32 + (y as f32 - x as f32) * t).round().clamp(0.0, 255.0) as u8
    };
    Rgba([mix(a[0], b[0]), mix(a[1], b[1]), mix(a[2], b[2]), 255])
}
