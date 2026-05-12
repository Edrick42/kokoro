//! Anatomical tail renderer — the *visible* shape of the tail is derived
//! from the live muscle state, not a fixed brush.
//!
//! Each segment's silhouette has two perpendicular thicknesses, one per
//! muscle in the antagonist pair:
//!
//! - The **flexor** (firing side `+perp`) bulges by
//!   `rest_thickness × (1 + 0.4 × activation)` — same volume-conservation
//!   approximation [`kokoro_body::muscle::MUSCLE_BULGE_FACTOR`] codifies.
//! - The **extensor** does the same on `−perp`.
//!
//! So a Playful wave inflates one side of the tail and deflates the
//! other in lockstep with the muscle the mind is firing. The pixels
//! that paint the tail come straight from physical state; no animation
//! curve is invented at render time.
//!
//! Z-order inside this module:
//! 1. Filled polygon body (`Palette::Orange`)
//! 2. Edge outline (`Palette::OrangeDark`) so the silhouette reads
//!    cleanly against any background
//! 3. Cream rings stamped at every Nth segment for the species'
//!    signature banded look
//!
//! The biomechanics debug overlay (bones, joints, muscles) is layered
//! *on top* of this by `skin::mod.rs`; this renderer paints only the
//! "skin" the player sees in the released game.

use image::{Rgba, RgbaImage};
use kokoro_art_palette::Palette;
use kokoro_rig::{BoneId, Skeleton};

use super::moluun::STANDALONE_TAIL_SEGMENTS;
use super::moluun_runtime::MoluunCubTail;

const BODY:        Rgba<u8> = Rgba(Palette::Orange.rgba(255));
const EDGE:        Rgba<u8> = Rgba(Palette::OrangeDark.rgba(255));
const RING:        Rgba<u8> = Rgba(Palette::OffWhite.rgba(255));
/// Every Nth segment along the tail gets a cream ring stamped on top.
const RING_STRIDE: usize    = 4;

/// Paint the cub's tail anatomically: each segment widens or narrows
/// with the matching muscle's current thickness. The shape literally
/// IS the muscle state.
pub fn paint_anatomical_tail(img: &mut RgbaImage, tail: &MoluunCubTail) {
    let skeleton = &tail.body.skeleton;
    if skeleton.dirty() {
        return;
    }

    // Build the two side polylines: top = flexor side, bot = extensor side.
    // One vertex per bone joint, plus the chain's base, so the polygon has
    // (segments + 1) vertices per side and stays continuous along the
    // entire tail.
    let mut top: Vec<(f32, f32)> = Vec::with_capacity(STANDALONE_TAIL_SEGMENTS + 1);
    let mut bot: Vec<(f32, f32)> = Vec::with_capacity(STANDALONE_TAIL_SEGMENTS + 1);

    for v in 0..=STANDALONE_TAIL_SEGMENTS {
        let Some((px, py, nx, ny)) = segment_frame(skeleton, v) else { continue };
        let (flex_t, ext_t) = vertex_thickness(tail, v);
        top.push((px + nx * flex_t, py + ny * flex_t));
        bot.push((px - nx * ext_t, py - ny * ext_t));
    }

    if top.len() < 2 {
        return;
    }

    // Fill: scanline through quads (top[i], top[i+1], bot[i+1], bot[i]).
    // Each quad is small (segment-length × thickness), so quad-by-quad is
    // both simpler and cheaper than a single polygon scan over all 16.
    for i in 0..(top.len() - 1) {
        fill_quad(img, top[i], top[i + 1], bot[i + 1], bot[i], BODY);
    }

    // Outline pass: 1-pixel edges along both polylines + the two end caps.
    for i in 0..(top.len() - 1) {
        draw_line_f(img, top[i], top[i + 1], EDGE);
        draw_line_f(img, bot[i], bot[i + 1], EDGE);
    }
    if let (Some(&t0), Some(&b0)) = (top.first(), bot.first()) {
        draw_line_f(img, t0, b0, EDGE);
    }
    if let (Some(&tn), Some(&bn)) = (top.last(), bot.last()) {
        draw_line_f(img, tn, bn, EDGE);
    }

    // Rings: a cream stripe across the tail's cross-section every
    // RING_STRIDE segments. Stamps the chord top[i]→bot[i].
    for i in (0..top.len()).step_by(RING_STRIDE).skip(1) {
        if i >= bot.len() { break; }
        draw_line_f(img, top[i], bot[i], RING);
    }
}

/// `v` is a vertex index 0..=N where N = number of segments. v = 0 is
/// the base (root tip), v = i is the tip of bone i.  Returns the
/// vertex's world position plus a unit perpendicular pointing toward
/// the flexor side. Returns `None` if either adjacent bone is too short
/// to derive a direction.
fn segment_frame(sk: &Skeleton, v: usize) -> Option<(f32, f32, f32, f32)> {
    let segments = STANDALONE_TAIL_SEGMENTS;
    // Sample the bone direction adjacent to this vertex. For interior
    // vertices we average the incoming and outgoing bone directions so
    // the perpendicular tracks the local curve instead of jumping at
    // every joint. End vertices use whichever single bone they touch.
    let pos = if v == 0 {
        sk.world_base(BoneId(1))
    } else {
        sk.world_tip(BoneId(v as u16))
    };
    let dir = if v == 0 {
        bone_direction(sk, 1)?
    } else if v >= segments {
        bone_direction(sk, segments as u16)?
    } else {
        // Average tangent of the two adjacent bones.
        let d_in = bone_direction(sk, v as u16)?;
        let d_out = bone_direction(sk, (v + 1) as u16)?;
        let avg = (d_in.0 + d_out.0, d_in.1 + d_out.1);
        normalize(avg)?
    };
    let perp = (-dir.1, dir.0);
    Some((pos.x, pos.y, perp.0, perp.1))
}

fn bone_direction(sk: &Skeleton, bone: u16) -> Option<(f32, f32)> {
    let base = sk.world_base(BoneId(bone));
    let tip  = sk.world_tip(BoneId(bone));
    normalize((tip.x - base.x, tip.y - base.y))
}

fn normalize(v: (f32, f32)) -> Option<(f32, f32)> {
    let len = (v.0 * v.0 + v.1 * v.1).sqrt();
    if len < 1e-4 { None } else { Some((v.0 / len, v.1 / len)) }
}

/// For vertex `v`, return `(flexor_thickness, extensor_thickness)` in
/// pixels. The thicknesses live on the muscles flanking the bone(s)
/// adjacent to the vertex — interior vertices average the two
/// neighbours so the silhouette is continuous, end vertices use only
/// the touching bone's muscle pair.
fn vertex_thickness(tail: &MoluunCubTail, v: usize) -> (f32, f32) {
    let segments = STANDALONE_TAIL_SEGMENTS;
    let muscle_thickness_at = |seg_idx: usize| -> Option<(f32, f32)> {
        // seg_idx is 1-based to match BoneId. Actuators are indexed by
        // attachment order, which mirrors that ordering in
        // cub_tail_body_for_creature.
        let actuator = tail.body.actuators.iter().find(|a| a.bone == BoneId(seg_idx as u16))?;
        Some((
            actuator.muscles.flexor.current_thickness(),
            actuator.muscles.extensor.current_thickness(),
        ))
    };

    if v == 0 {
        muscle_thickness_at(1).unwrap_or((1.0, 1.0))
    } else if v >= segments {
        muscle_thickness_at(segments).unwrap_or((1.0, 1.0))
    } else {
        // Interior: average adjacent muscle pairs so the polygon edge is
        // continuous (no width jumps at every joint).
        let a = muscle_thickness_at(v).unwrap_or((1.0, 1.0));
        let b = muscle_thickness_at(v + 1).unwrap_or((1.0, 1.0));
        ((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5)
    }
}

// ---------------------------------------------------------------------------
// Pixel primitives
// ---------------------------------------------------------------------------

/// Fill a convex quad given in CCW order around the perimeter. Pixel art
/// scanline: walk every integer y in the bounding box, find the polygon's
/// x range on that scanline (point-in-quad test by checking if the y is
/// between every edge's endpoints), and fill the run.
fn fill_quad(
    img: &mut RgbaImage,
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    d: (f32, f32),
    color: Rgba<u8>,
) {
    let pts = [a, b, c, d];
    let min_y = pts.iter().map(|p| p.1).fold(f32::INFINITY, f32::min).floor() as i32;
    let max_y = pts.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
    let min_x = pts.iter().map(|p| p.0).fold(f32::INFINITY, f32::min).floor() as i32;
    let max_x = pts.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if point_in_quad(x as f32 + 0.5, y as f32 + 0.5, &pts) {
                put(img, x, y, color);
            }
        }
    }
}

/// Half-plane test for a CCW quad. Each consecutive edge defines a line;
/// the point is inside iff it lies on the same side (≥0 cross product)
/// of every edge.
fn point_in_quad(px: f32, py: f32, pts: &[(f32, f32); 4]) -> bool {
    let mut sign: f32 = 0.0;
    for i in 0..4 {
        let (ax, ay) = pts[i];
        let (bx, by) = pts[(i + 1) % 4];
        let cross = (bx - ax) * (py - ay) - (by - ay) * (px - ax);
        if cross.abs() < 1e-6 { continue; }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    true
}

fn draw_line_f(img: &mut RgbaImage, a: (f32, f32), b: (f32, f32), color: Rgba<u8>) {
    let (x0, y0) = (a.0.round() as i32, a.1.round() as i32);
    let (x1, y1) = (b.0.round() as i32, b.1.round() as i32);
    let dx =  (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        put(img, x, y, color);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

fn put(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
        return;
    }
    img.put_pixel(x as u32, y as u32, color);
}
