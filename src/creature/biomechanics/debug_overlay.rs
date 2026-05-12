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
//! - **`paint_bones`** — every bone, as a black line + cyan dot at each
//!   joint. Reads `Skeleton::world_base`/`world_tip` directly.
//! - **`paint_joints`** — joint dots coloured by how close the current
//!   angle is to the ROM limit. Reads `Joint::range_min/max` +
//!   `JointState::angle`. Stress 0 = at rest (cyan), 1 = at ROM cap (red).
//! - **`paint_muscles`** — for each bone segment with an active joint, a
//!   pink "muscle belly" on each side of the bone. Brightness modulates
//!   with the `PairIntent` from the last sim step (flexor on one perp
//!   side, extensor on the opposite).
//!
//! All overlays are blits over the existing buffer — they don't clear or
//! own the canvas.

use image::{Rgba, RgbaImage};
use kokoro_art_palette::Palette;
use kokoro_body::actuation::PairIntent;
use kokoro_rig::{BoneId, Skeleton};

// --- Palette anchors -------------------------------------------------------

const BONE_COLOR:       Rgba<u8> = Rgba(Palette::NearBlack.rgba(255));
const JOINT_REST_RGB:   [u8; 3]  = Palette::CyanBright.rgb();
const JOINT_STRESS_RGB: [u8; 3]  = Palette::Red.rgb();
// Muscle baseline (at-rest tone, just barely visible) and saturated active.
const MUSCLE_REST_A:    u8 = 70;
const MUSCLE_ACTIVE_A:  u8 = 230;

// --- Public layer entry points --------------------------------------------

/// Paints every bone in `skeleton` as a 1-pixel line with a joint dot at
/// each endpoint. Caller must have run `Skeleton::forward` already; if
/// the dirty flag is set the function is a no-op (we'd paint stale
/// positions, which would lie).
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

/// Paints the muscle layer for every bone segment with an active joint:
/// a `CoralPink` shape on each perpendicular side of the bone. The shape
/// brightness rises with the matching half of the segment's `PairIntent`,
/// so the side that the mind is firing reads as the bright one and the
/// at-rest side stays a dim trace.
///
/// `intents[i]` is interpreted as the intent for bone `i + 1` (bone 0 is
/// the anchor root and has no muscle pair). The function clamps to the
/// shorter of `intents.len()` and the skeleton's active segment count, so
/// a mid-resize is safe.
pub fn paint_muscles(img: &mut RgbaImage, skeleton: &Skeleton, intents: &[PairIntent]) {
    if skeleton.dirty() {
        return;
    }
    let segment_count = intents.len().min(skeleton.len().saturating_sub(1));
    for seg in 0..segment_count {
        let id = BoneId((seg + 1) as u16);
        let base = skeleton.world_base(id);
        let tip = skeleton.world_tip(id);
        let dx = tip.x - base.x;
        let dy = tip.y - base.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.5 {
            continue; // degenerate segment — skip rather than divide by zero
        }
        // Unit perpendicular vector (rotate 90°). One side is the flexor
        // belly, the opposite side is the extensor belly. Hinge sign
        // convention: +perp ≈ where a positive angle takes the tip.
        let nx = -dy / len;
        let ny = dx / len;
        let mid_x = (base.x + tip.x) * 0.5;
        let mid_y = (base.y + tip.y) * 0.5;
        let belly_offset = 1.6_f32; // distance from bone axis to muscle centre

        let flexor_alpha = muscle_alpha(intents[seg].flexor);
        let extensor_alpha = muscle_alpha(intents[seg].extensor);

        paint_muscle_belly(
            img,
            mid_x + nx * belly_offset,
            mid_y + ny * belly_offset,
            len,
            nx,
            ny,
            flexor_alpha,
        );
        paint_muscle_belly(
            img,
            mid_x - nx * belly_offset,
            mid_y - ny * belly_offset,
            len,
            -nx,
            -ny,
            extensor_alpha,
        );
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

/// Maps muscle intent (0..=1, may exceed 1 if the closure returns a
/// surge) to a render alpha in `[MUSCLE_REST_A, MUSCLE_ACTIVE_A]`.
fn muscle_alpha(intent: f32) -> u8 {
    let t = intent.clamp(0.0, 1.0);
    let a = MUSCLE_REST_A as f32 + (MUSCLE_ACTIVE_A - MUSCLE_REST_A) as f32 * t;
    a.round().clamp(0.0, 255.0) as u8
}

/// Paints a muscle belly as a series of stacked filled circles forming a
/// tapered tube along the bone direction. Radius peaks at the middle
/// (biceps-like belly) and tapers to a thin attachment near both joints.
///
/// `cx, cy` is the belly centre, `len` the bone length (used to scale how
/// many samples we take), `(tx, ty)` the tangent along the bone, and
/// `alpha` the per-call opacity from current intent.
fn paint_muscle_belly(img: &mut RgbaImage, cx: f32, cy: f32, len: f32, tx: f32, ty: f32, alpha: u8) {
    if alpha < MUSCLE_REST_A.saturating_sub(2) {
        return;
    }
    let [r, g, b] = Palette::CoralPink.rgb();
    let color = Rgba([r, g, b, alpha]);
    // Tangent direction (already perpendicular to perp_normal). We sample
    // along the bone axis from -0.5 to +0.5 of its length.
    // Sample every 0.6 px so even short segments get at least a couple
    // of stamps; multi-stamp keeps the belly contiguous in pixel space.
    let samples = ((len * 1.7).round() as i32).max(3);
    let mid_radius_px = 1.3_f32;
    let edge_radius_px = 0.7_f32;
    for k in 0..samples {
        let t = -0.5 + (k as f32) / ((samples - 1).max(1) as f32);
        // Belly bulges: cosine taper, max at t=0.
        let bulge = 1.0 - (2.0 * t).abs(); // triangular fallback for any samples
        let radius = edge_radius_px + (mid_radius_px - edge_radius_px) * bulge.max(0.0);
        let px = cx + tx * len * t;
        let py = cy + ty * len * t;
        fill_disc(img, px, py, radius, color);
    }
}

fn fill_disc(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32, color: Rgba<u8>) {
    let r = radius.max(0.5);
    let r_sq = r * r;
    let x0 = (cx - r).floor() as i32;
    let x1 = (cx + r).ceil() as i32;
    let y0 = (cy - r).floor() as i32;
    let y1 = (cy + r).ceil() as i32;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy <= r_sq {
                blend_pixel(img, x, y, color);
            }
        }
    }
}

/// Alpha-blends `color` over the destination pixel. Used so muscle bellies
/// stack visually without fully repainting underlying bones.
fn blend_pixel(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
        return;
    }
    let dst = img.get_pixel_mut(x as u32, y as u32);
    let a = color.0[3] as f32 / 255.0;
    let inv = 1.0 - a;
    for c in 0..3 {
        dst.0[c] = (color.0[c] as f32 * a + dst.0[c] as f32 * inv).round() as u8;
    }
    dst.0[3] = dst.0[3].max(color.0[3]);
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
    put(img, cx, cy, color);
    put(img, cx - 1, cy, color);
    put(img, cx + 1, cy, color);
    put(img, cx, cy - 1, color);
    put(img, cx, cy + 1, color);
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
    Rgba([mix(a[0], b[0]), mix(a[1], b[1]), mix(a[2], b[2]), 230])
}
