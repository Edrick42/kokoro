//! Debug visualisation: render a skeleton as bone-stick lines + joint dots
//! on an `RgbaImage` so authors can eyeball rest poses without writing a
//! full rendering pipeline. Lives behind the `image` feature.
//!
//! Use case: after authoring a new skeleton, drop a `#[test] #[ignore]`
//! into the species' rig module that calls `render_skeleton_overlay` and
//! saves to `target/sprite-snapshots/`. Inspect the PNG, confirm bone
//! positions look anatomically right, then move on to the brushes that
//! actually paint pixels at those positions.

use crate::{BoneId, Skeleton, Vec2};
use image::{Rgba, RgbaImage};

/// Render the skeleton onto `img` as line segments (one per bone) plus
/// joint dots. Skeleton must already have been `forward()`-ed.
///
/// - `line_color` strokes the bone segments.
/// - `joint_color` marks each bone's base (3×3 cross) and tip (1px dot).
/// - `tint_by_z`: when true, deeper-z bones (negative z = behind) are
///   darkened so the eye can spot z-order at a glance.
pub fn render_skeleton_overlay(
    img: &mut RgbaImage,
    skeleton: &Skeleton,
    line_color: Rgba<u8>,
    joint_color: Rgba<u8>,
    tint_by_z: bool,
) {
    for i in 0..skeleton.len() {
        let id = BoneId(i as u16);
        let bone = skeleton.bone(id);
        let base = skeleton.world_base(id);
        let tip = skeleton.world_tip(id);

        let stroke = if tint_by_z {
            tint_for_z(line_color, bone.z_layer)
        } else {
            line_color
        };

        if bone.effective_length() > 0.5 {
            draw_line(img, base, tip, stroke);
            draw_pixel(img, tip, joint_color);
        }
        draw_cross(img, base, joint_color);
    }
}

/// Multiply the stroke's RGB by an intensity factor derived from z. z=-2
/// is darkest (back), z=+2 is brightest (front). Alpha is preserved.
fn tint_for_z(base: Rgba<u8>, z: i8) -> Rgba<u8> {
    let factor = match z {
        i if i <= -2 => 0.40,
        -1 => 0.65,
        0 => 1.00,
        1 => 1.15,
        i if i >= 2 => 1.30,
        _ => 1.0,
    };
    let scale = |c: u8| ((c as f32 * factor).clamp(0.0, 255.0)) as u8;
    Rgba([scale(base.0[0]), scale(base.0[1]), scale(base.0[2]), base.0[3]])
}

fn draw_pixel(img: &mut RgbaImage, p: Vec2, color: Rgba<u8>) {
    let x = p.x.round() as i32;
    let y = p.y.round() as i32;
    if in_bounds(img, x, y) {
        img.put_pixel(x as u32, y as u32, color);
    }
}

fn draw_cross(img: &mut RgbaImage, p: Vec2, color: Rgba<u8>) {
    let x = p.x.round() as i32;
    let y = p.y.round() as i32;
    for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nx = x + dx;
        let ny = y + dy;
        if in_bounds(img, nx, ny) {
            img.put_pixel(nx as u32, ny as u32, color);
        }
    }
}

/// Bresenham's line algorithm. The classic — runs in i32 ops, never
/// allocates, paints at most one pixel per integer step.
fn draw_line(img: &mut RgbaImage, a: Vec2, b: Vec2, color: Rgba<u8>) {
    let mut x0 = a.x.round() as i32;
    let mut y0 = a.y.round() as i32;
    let x1 = b.x.round() as i32;
    let y1 = b.y.round() as i32;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if in_bounds(img, x0, y0) {
            img.put_pixel(x0 as u32, y0 as u32, color);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

#[inline]
fn in_bounds(img: &RgbaImage, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < img.width() as i32 && y < img.height() as i32
}
