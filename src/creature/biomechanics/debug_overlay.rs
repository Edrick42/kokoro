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
//! - **`paint_muscles`** — for **every** muscle in the rig, a fusiform
//!   belly is painted alongside its bone (flexor on `+perp`, extensor
//!   on `−perp`). The belly's width comes from
//!   [`kokoro_body::Muscle::current_thickness`] — `rest_thickness` at
//!   activation 0, bulging by 40 % at activation 1 (volume
//!   conservation). Brightness rises with activation so a firing muscle
//!   reads bright pink while a resting one is faint pink — the muscle
//!   never disappears, because *real muscles don't disappear when an
//!   animal sleeps*; they just stop firing.
//!
//! All paints are OPAQUE (no alpha blending) so colours stay vivid and
//! crisp at the 64×64 native canvas resolution.

use image::{Rgba, RgbaImage};
use kokoro_art_palette::Palette;
use kokoro_rig::{BoneId, Skeleton};

use super::moluun::STANDALONE_TAIL_SEGMENTS;
use super::moluun_runtime::MoluunCubTail;

// --- Palette anchors -------------------------------------------------------

const BONE_COLOR:       Rgba<u8> = Rgba(Palette::NearBlack.rgba(255));
const JOINT_REST_RGB:   [u8; 3]  = Palette::CyanBright.rgb();
const JOINT_STRESS_RGB: [u8; 3]  = Palette::Red.rgb();
/// Resting-state alpha for muscle bellies. Visible but understated so
/// firing muscles clearly stand out against them.
const MUSCLE_REST_ALPHA:   u8 = 120;
/// Fully firing muscle alpha. Saturated CoralPink jumps out against
/// the orange tail body underneath.
const MUSCLE_FIRING_ALPHA: u8 = 240;

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

/// Paints every muscle in the tail as a fusiform pink belly alongside
/// its bone. The belly always renders — muscles exist whether the mind
/// is firing them or not. Activation modulates two things:
///
/// 1. **Width**: `current_thickness = rest × (1 + 0.4 × activation)`,
///    same volume-conservation formula the tail skin uses.
/// 2. **Brightness**: alpha lerps from `MUSCLE_REST_ALPHA` (resting,
///    faint) to `MUSCLE_FIRING_ALPHA` (firing, saturated).
///
/// At sleep with activation = 0 across the chain you see 32 faint pink
/// fusiform dots lining the tail (flexor + extensor per segment); when
/// Playful fires the wave each muscle in turn brightens and bulges as
/// the signal passes.
pub fn paint_muscles(img: &mut RgbaImage, tail: &MoluunCubTail) {
    let skeleton = &tail.body.skeleton;
    if skeleton.dirty() {
        return;
    }
    for seg in 0..STANDALONE_TAIL_SEGMENTS {
        let bone_id = BoneId((seg + 1) as u16);
        let base = skeleton.world_base(bone_id);
        let tip  = skeleton.world_tip(bone_id);
        let dx = tip.x - base.x;
        let dy = tip.y - base.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.5 {
            continue;
        }
        let nx = -dy / len; // unit perpendicular, +perp = flexor side
        let ny = dx / len;

        let Some(actuator) = tail.body.actuators.iter().find(|a| a.bone == bone_id) else { continue };

        // Flexor on +perp, extensor on −perp. Each belly's centre sits
        // half its thickness off the bone axis, so the belly fills the
        // tube between the bone (perp 0) and the skin edge.
        let flex_th  = actuator.muscles.flexor.current_thickness();
        let flex_act = actuator.muscles.flexor.activation;
        paint_muscle_belly(img, base, tip, dx, dy, len, nx, ny, flex_th, flex_act);

        let ext_th  = actuator.muscles.extensor.current_thickness();
        let ext_act = actuator.muscles.extensor.activation;
        paint_muscle_belly(img, base, tip, dx, dy, len, -nx, -ny, ext_th, ext_act);
    }
}

/// Stamp one fusiform muscle belly on the `(perp_x, perp_y)` side of
/// the bone. The shape tapers near the tendons (longitudinal ends) so
/// it reads as a muscle belly rather than a uniform tube.
fn paint_muscle_belly(
    img: &mut RgbaImage,
    base: kokoro_rig::Vec2,
    _tip: kokoro_rig::Vec2,
    dx: f32, dy: f32, len: f32,
    perp_x: f32, perp_y: f32,
    thickness: f32,
    activation: f32,
) {
    let half_th = thickness * 0.5;
    if half_th < 0.2 {
        return;
    }
    // Belly centre = half a thickness off the bone, at the segment midpoint.
    let mid_x = base.x + dx * 0.5;
    let mid_y = base.y + dy * 0.5;

    let alpha = (MUSCLE_REST_ALPHA as f32
        + (MUSCLE_FIRING_ALPHA - MUSCLE_REST_ALPHA) as f32 * activation.clamp(0.0, 1.0))
        .round() as u8;
    let [r, g, b] = Palette::CoralPink.rgb();
    let color = Rgba([r, g, b, alpha]);

    // Sample points along the bone direction (tangent), tapering off
    // toward both ends so the silhouette is fusiform (wider in the
    // middle, narrower at the tendons).
    let samples = (len.ceil() as i32 * 2).max(3);
    for k in 0..=samples {
        let s = (k as f32) / (samples as f32); // 0..1 along bone length
        let along = s - 0.5;                   // −0.5..+0.5
        // Parabolic taper: full radius in the middle, half at the ends.
        let taper = 1.0 - (2.0 * along).abs().powi(2);
        let r_local = half_th * (0.5 + 0.5 * taper.max(0.0));
        let centre_offset = half_th; // pure perp distance from bone axis
        let cx = mid_x + (dx / len) * len * along + perp_x * centre_offset;
        let cy = mid_y + (dy / len) * len * along + perp_y * centre_offset;
        blended_disc(img, cx, cy, r_local, color);
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

/// Alpha-blend a filled disc over the existing buffer. The tail skin
/// underneath is opaque orange; with muscle alpha < 255 the firing
/// belly tints the orange peach instead of replacing it.
fn blended_disc(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32, color: Rgba<u8>) {
    if radius <= 0.0 {
        return;
    }
    let r_sq = radius * radius;
    let x0 = (cx - radius - 0.5).floor() as i32;
    let x1 = (cx + radius + 0.5).ceil() as i32;
    let y0 = (cy - radius - 0.5).floor() as i32;
    let y1 = (cy + radius + 0.5).ceil() as i32;
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
