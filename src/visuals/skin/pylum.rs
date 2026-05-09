//! Pylum skin — golden highland bird (secretary bird / cassowary).
//!
//! Structural evolution:
//! - Cub:   fluffy down ball, tiny beak, short legs, no wings/crest
//! - Young: LEGS grow dramatically, body stays small on top, gangly, wing nubs
//! - Adult: tall imposing form, casque crown, full wings, powerful taloned legs
//! - Elder: white crest tip, faded wing edges

use image::{RgbaImage, Rgba};
use bevy::prelude::Res;
use kokoro_art_palette::Palette;
use kokoro_art_palette::dsl::{BumpyDome, TaperedTail};
use crate::creature::interaction::soft_body::SoftBody;
use crate::mind::MoodState;
use super::{SpeciesSkin, fill_circle, fill_rect, fill_ellipse, put, draw_eyes, fade, NEAR_BLACK_PX};

// Master palette anchors (docs/aesthetic-targets.md §3 + §4.2)
const HIGHLIGHT:        Rgba<u8> = Rgba(Palette::OffWhite.rgba(180));
const CLAW:             Rgba<u8> = Rgba(Palette::DeepBrown.rgba(255));
const RESONANCE:        Rgba<u8> = Rgba(Palette::Gold.rgba(70));
const RESONANCE_BRIGHT: Rgba<u8> = Rgba(Palette::OrangeBright.rgba(110));
// Blush ramps with kawaii_factor (§4b/§4c): full Cub, subtle Young, absent past.
const BLUSH:            Rgba<u8> = Rgba(Palette::CoralPink.rgba(255));
const BLUSH_SUBTLE:     Rgba<u8> = Rgba(Palette::CoralPink.rgba(128));

// ===================================================================
// EGG
// ===================================================================

pub fn draw_egg(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32) {
    let cy = 30;
    BumpyDome::new(cx, cy, 11, Palette::Orange)
        .with_height(15)
        .with_bumpiness(0.08)
        .with_bumps(8)
        .with_seed(103)
        .paint_with(img, p.egg, None);
    fill_ellipse(img, cx, cy - 3, 9, 10, fade(p.egg, 0.05));
    for &(dx, dy) in &[(-3,-7), (4,-4), (-5,1), (2,4), (5,-8), (-1,7), (3,0), (-4,-3), (6,2)] {
        put(img, cx + dx, cy + dy, p.egg_spot);
        put(img, cx + dx + 1, cy + dy, p.egg_spot);
    }
}

// ===================================================================
// CUB — fluffy down ball
// ===================================================================

pub fn draw_cub(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    let (_, by) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 22));
    let br = 17;

    // "Fluffy down ball" silhouette via BumpyDome — moderate bumpiness reads
    // as down-feathers rather than a smooth orb. Speckle highlights stay as
    // a separate pass on top.
    BumpyDome::new(cx, by, br as u32, Palette::Orange)
        .with_bumpiness(0.35)
        .with_bumps(9)
        .with_seed(13)
        .paint_with(img, p.body, None);
    for &(dx, dy) in &[(-7,-6), (6,-4), (-4,4), (8,2), (-2,-10), (5,7), (-9,1), (3,-8)] {
        put(img, cx + dx, by + dy, p.body_light);
    }
    fill_circle(img, cx, by + 5, 11, p.body_light);

    // Tiny tuft
    fill_rect(img, cx - 1, by - br, 3, 3, p.body_light);

    // Short stubby legs
    fill_rect(img, cx - 5, by + br - 2, 4, 5, p.mouth);
    fill_rect(img, cx + 2, by + br - 2, 4, 5, p.mouth);

    fill_circle(img, cx, by + 3, 3, RESONANCE);

    draw_eyes(img, cx, by + 1, 6, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 6, by + 1, HIGHLIGHT);
        put(img, cx + 2, by + 1, HIGHLIGHT);
    }

    // Cub blush
    fill_rect(img, cx - 8, by + 4, 2, 2, BLUSH);
    fill_rect(img, cx + 6, by + 4, 2, 2, BLUSH);

    fill_rect(img, cx - 1, by + 9, 3, 2, NEAR_BLACK_PX);
    put(img, cx, by + 11, NEAR_BLACK_PX);
}

// ===================================================================
// YOUNG — gangly: LEGS explode, body small on top
// ===================================================================

pub fn draw_young(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // Soft body positions
    let (hx, hy) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 12));
    let (bx, by) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 26));
    let (csq_x, csq_y) = sb.as_ref().map(|b| b.point("casque").px()).unwrap_or((cx, 5));
    let (wl_x, wl_y) = sb.as_ref().map(|b| b.point("wing_l").px()).unwrap_or((cx - 11, 22));
    let (wr_x, wr_y) = sb.as_ref().map(|b| b.point("wing_r").px()).unwrap_or((cx + 11, 22));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 4, 48));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 4, 48));
    let br = 11;

    // Long legs from body to feet
    let leg_top = by + br - 2;
    let ll_h = (fl_y - leg_top).max(4);
    let lr_h = (fr_y - leg_top).max(4);
    fill_rect(img, fl_x - 2, leg_top, 4, ll_h, p.mouth);
    fill_rect(img, fr_x - 2, leg_top, 4, lr_h, p.mouth);
    // Knee joints (mid-leg)
    fill_rect(img, fl_x - 3, leg_top + ll_h / 2, 6, 3, p.mouth);
    fill_rect(img, fr_x - 3, leg_top + lr_h / 2, 6, 3, p.mouth);
    // Feet
    fill_rect(img, fl_x - 3, fl_y - 1, 5, 3, p.mouth);
    fill_rect(img, fr_x - 2, fr_y - 1, 5, 3, p.mouth);
    for dx in [-1, 1, 3] {
        put(img, fl_x - 3 + dx, fl_y + 2, CLAW);
        put(img, fr_x - 2 + dx, fr_y + 2, CLAW);
    }

    // Sprouting wings — TaperedTail angled outward + downward so they read
    // as actual wing nubs branching off the body, not boxy panels glued to
    // the side. Base 2 (5px wide) at the shoulder, taper to a point ~7px
    // diagonally outward.
    TaperedTail::new((wl_x + 2, wl_y), (wl_x - 3, wl_y + 7), 2, Palette::Orange)
        .paint_with(img, p.accent, None);
    TaperedTail::new((wr_x - 1, wr_y), (wr_x + 4, wr_y + 7), 2, Palette::Orange)
        .paint_with(img, p.accent, None);

    // Body — BumpyDome continues the kawaii_factor curve. Slightly less
    // bumpy than the cub down-ball (0.28 vs 0.35) as plumage tightens up
    // around the gangly young silhouette.
    BumpyDome::new(bx, by, br as u32, Palette::Orange)
        .with_bumpiness(0.28)
        .with_bumps(9)
        .with_seed(29)
        .paint_with(img, p.body, None);
    for &(dx, dy) in &[(-4,-3), (3,-2), (-2,3), (5,1)] {
        put(img, bx + dx, by + dy, p.body_light);
    }
    fill_circle(img, bx, by + 3, 7, p.body_light);

    // Neck
    let neck_cx = (hx + bx) / 2;
    let neck_top = hy.min(by);
    let neck_h = (by - hy).max(1);
    fill_rect(img, neck_cx - 3, neck_top, 7, neck_h, p.body);

    // Head
    fill_circle(img, hx, hy, 9, p.body);

    // Casque crown (moves with soft body)
    fill_rect(img, csq_x - 1, csq_y, 3, 4, p.body);
    put(img, csq_x, csq_y - 1, p.accent);
    put(img, csq_x + 1, csq_y - 1, p.accent);

    fill_circle(img, bx, by + 2, 4, RESONANCE);

    draw_eyes(img, hx, hy + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, hx - 5, hy + 1, HIGHLIGHT);
        put(img, hx + 2, hy + 1, HIGHLIGHT);
    }

    // Young blush — sutil
    fill_rect(img, hx - 6, hy + 3, 2, 2, BLUSH_SUBTLE);
    fill_rect(img, hx + 4, hy + 3, 2, 2, BLUSH_SUBTLE);

    // Beak
    fill_rect(img, hx - 2, hy + 5, 5, 3, NEAR_BLACK_PX);
    fill_rect(img, hx - 1, hy + 8, 3, 2, NEAR_BLACK_PX);
}

// ===================================================================
// ADULT — tall imposing: casque, wings, talons
// ===================================================================

pub fn draw_adult(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    let (hx, hy) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 10));
    let hr = 9;
    let (_, body_y) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 24));
    let body_r = 12;
    let (wl_x, wl_y) = sb.as_ref().map(|b| b.point("wing_l").px()).unwrap_or((cx - 14, 20));
    let (wr_x, wr_y) = sb.as_ref().map(|b| b.point("wing_r").px()).unwrap_or((cx + 14, 20));
    let (wtl_x, wtl_y) = sb.as_ref().map(|b| b.point("wingtip_l").px()).unwrap_or((cx - 20, 26));
    let (wtr_x, wtr_y) = sb.as_ref().map(|b| b.point("wingtip_r").px()).unwrap_or((cx + 20, 26));
    let (_, _tail_y) = sb.as_ref().map(|b| b.point("tail").px()).unwrap_or((cx, 34));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 5, 54));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 5, 54));

    // Legs — from body down to soft body foot positions
    let leg_top = body_y + body_r - 2;
    let ll_h = (fl_y - leg_top).max(4);
    let lr_h = (fr_y - leg_top).max(4);
    fill_rect(img, fl_x - 2, leg_top, 5, ll_h, p.mouth);
    fill_rect(img, fr_x - 2, leg_top, 5, lr_h, p.mouth);
    // Knee joints
    fill_rect(img, fl_x - 3, leg_top + ll_h / 2, 7, 4, p.mouth);
    fill_rect(img, fr_x - 3, leg_top + lr_h / 2, 7, 4, p.mouth);
    // Feet
    fill_rect(img, fl_x - 4, fl_y - 1, 6, 4, p.mouth);
    fill_rect(img, fr_x - 3, fr_y - 1, 6, 4, p.mouth);
    // Claws
    for dx in [-2, 0, 2, 4] {
        put(img, fl_x - 4 + dx, fl_y + 3, CLAW);
        put(img, fr_x - 3 + dx, fr_y + 3, CLAW);
    }

    // Wings — full adult feathers expressed as TaperedTails radiating from
    // the body to the wingtip soft-body anchors. Inner span (body→wing root)
    // is the heavy proximal feather mass (base 4); outer span (wing root→
    // wingtip) is the long finer flight feather (base 3, longer reach).
    TaperedTail::new((cx, wl_y), (wl_x, wl_y), 4, Palette::Orange)
        .paint_with(img, p.accent, None);
    TaperedTail::new((cx, wr_y), (wr_x, wr_y), 4, Palette::Orange)
        .paint_with(img, p.accent, None);
    TaperedTail::new((wl_x, wl_y), (wtl_x, wtl_y), 3, Palette::Orange)
        .paint_with(img, p.accent, None);
    TaperedTail::new((wr_x, wr_y), (wtr_x, wtr_y), 3, Palette::Orange)
        .paint_with(img, p.accent, None);
    // Wingtip feather streaks — fading splays at the very end of each wing.
    for i in 0..4 {
        put(img, wtl_x, wtl_y + i * 2, fade(p.accent, 0.3));
        put(img, wtr_x, wtr_y + i * 2, fade(p.accent, 0.3));
    }

    // Body — BumpyDome closes Pylum's lifecycle. Lowest bumpiness in the
    // curve (0.20) so the adult silhouette is solid and imposing.
    BumpyDome::new(cx, body_y, body_r as u32, Palette::Orange)
        .with_bumpiness(0.20)
        .with_bumps(11)
        .with_seed(53)
        .paint_with(img, p.body, None);
    for &(dx, dy) in &[(-5,-4), (4,-2), (-2,3), (6,1), (-7,0), (3,-6)] {
        put(img, cx + dx, body_y + dy, p.body_light);
    }
    fill_circle(img, cx, body_y + 3, 8, p.body_light);

    // Neck (from head to body)
    let neck_cx = (hx + cx) / 2;
    let neck_top = hy.min(body_y);
    let neck_h = (body_y - hy).max(1);
    fill_rect(img, neck_cx - 4, neck_top, 9, neck_h, p.body);
    fill_rect(img, neck_cx - 3, neck_top + 1, 7, neck_h.max(1) - 1, p.accent);

    // Head (on top of neck) — BumpyDome with very low bumpiness; adult
    // Pylum's head is sharp-featured (casque, sharp beak) so the silhouette
    // stays tight.
    BumpyDome::new(hx, hy, hr as u32, Palette::Orange)
        .with_bumpiness(0.15)
        .with_bumps(8)
        .with_seed(67)
        .paint_with(img, p.body, None);

    // Casque crown (moves with head)
    fill_rect(img, hx - 2, hy - hr - 4, 5, 7, p.body);
    fill_rect(img, hx - 1, hy - hr - 6, 3, 3, p.accent);
    put(img, hx, hy - hr - 7, fade(p.accent, 0.3));
    put(img, hx - 3, hy - hr - 3, p.accent);
    put(img, hx + 3, hy - hr - 3, p.accent);
    put(img, hx - 4, hy - hr - 1, p.accent);
    put(img, hx + 4, hy - hr - 1, p.accent);

    // Kokoro-sac
    fill_circle(img, cx, body_y + 2, 5, RESONANCE_BRIGHT);
    fill_circle(img, cx, body_y + 2, 3, RESONANCE);
    put(img, cx - body_r - 1, body_y, RESONANCE);
    put(img, cx + body_r, body_y, RESONANCE);

    // Eyes (on head)
    draw_eyes(img, hx, hy + 1, 5, 3, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, hx - 4, hy + 1, HIGHLIGHT);
        put(img, hx + 2, hy + 1, HIGHLIGHT);
    }

    // Beak (on head)
    fill_rect(img, hx - 3, hy + 5, 7, 3, NEAR_BLACK_PX);
    fill_rect(img, hx - 2, hy + 8, 5, 2, NEAR_BLACK_PX);
    fill_rect(img, hx - 1, hy + 10, 3, 2, NEAR_BLACK_PX);

    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        put(img, cx - 5, body_y + 1, p.body_light);
        put(img, cx + 5, body_y + 1, p.body_light);
    }
}

// ===================================================================
// ELDER — own silhouette per docs/aesthetic-targets.md §4d
// ===================================================================
// kawaii_factor 0.2 (palette palette §4d.3, 6 colors): DeepBrown outline,
// OrangeDark body, BrownDark shadow, GoldDark wing edge fading, Cream belly,
// NearBlack slit eye. Casque collapses to a stub, wings fold close to body,
// legs sit short and grounded.

pub fn draw_elder(img: &mut RgbaImage, cx: i32, _mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    let body: Rgba<u8>        = Palette::OrangeDark.into();
    let body_shadow: Rgba<u8> = Palette::BrownDark.into();
    let belly: Rgba<u8>       = Palette::Cream.into();
    let wing_edge: Rgba<u8>   = Palette::GoldDark.into();
    let eye: Rgba<u8>         = Palette::NearBlack.into();

    // Hunched anatomy — head sits lower (14 vs 10 adult), body slightly lower.
    let (hx, hy)         = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 14));
    let (_, body_y)      = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 26));
    let (wl_x, wl_y)     = sb.as_ref().map(|b| b.point("wing_l").px()).unwrap_or((cx - 12, 24));
    let (wr_x, wr_y)     = sb.as_ref().map(|b| b.point("wing_r").px()).unwrap_or((cx + 12, 24));
    let (fl_x, fl_y)     = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 4, 50));
    let (fr_x, fr_y)     = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 4, 50));
    let body_r = 12;
    let hr = 8;

    // Folded wings — stubs that hug the body. TaperedTail goes from the body
    // edge outward to the soft-body wing root only (no wingtip extension).
    // Painted in OrangeDark first, then a single GoldDark trim pixel on top
    // captures the §4d.3 "wing edge fading" detail without adding a brush.
    TaperedTail::new((cx - body_r / 2, body_y - 1), (wl_x, wl_y), 3, Palette::OrangeDark)
        .paint_with(img, body, None);
    TaperedTail::new((cx + body_r / 2, body_y - 1), (wr_x, wr_y), 3, Palette::OrangeDark)
        .paint_with(img, body, None);
    put(img, wl_x, wl_y, wing_edge);
    put(img, wl_x - 1, wl_y + 1, wing_edge);
    put(img, wr_x, wr_y, wing_edge);
    put(img, wr_x + 1, wr_y + 1, wing_edge);

    // Short grounded legs — straight from body to feet, no knee bulge.
    let leg_top = body_y + body_r - 2;
    let ll_h = (fl_y - leg_top).max(3);
    let lr_h = (fr_y - leg_top).max(3);
    fill_rect(img, fl_x - 1, leg_top, 3, ll_h, body_shadow);
    fill_rect(img, fr_x - 1, leg_top, 3, lr_h, body_shadow);
    fill_rect(img, fl_x - 2, fl_y, 5, 2, body_shadow);
    fill_rect(img, fr_x - 2, fr_y, 5, 2, body_shadow);

    // Body — plump, low bumpiness so the plumage reads as settled.
    BumpyDome::new(cx, body_y, body_r as u32, Palette::OrangeDark)
        .with_bumpiness(0.12)
        .with_bumps(11)
        .with_seed(79)
        .paint_with(img, body, None);

    // Quiet cream belly.
    fill_circle(img, cx, body_y + 3, 6, belly);

    // Short thick neck — head sits directly on body.
    let neck_cx = (hx + cx) / 2;
    let neck_top = hy.min(body_y);
    let neck_h = (body_y - hy).max(1);
    fill_rect(img, neck_cx - 4, neck_top, 9, neck_h, body);

    // Small head — same OrangeDark mass.
    BumpyDome::new(hx, hy, hr as u32, Palette::OrangeDark)
        .with_bumpiness(0.13)
        .with_bumps(8)
        .with_seed(97)
        .paint_with(img, body, None);

    // Casque collapses to a tiny stub (no accent crown).
    fill_rect(img, hx - 1, hy - hr - 2, 3, 3, body_shadow);

    // Slit eyes — 1px tall lines, no glint.
    fill_rect(img, hx - 5, hy + 1, 4, 1, eye);
    fill_rect(img, hx + 1, hy + 1, 4, 1, eye);

    // Short beak — DeepBrown leans into body shadow.
    fill_rect(img, hx - 2, hy + 5, 5, 2, body_shadow);
    put(img, hx, hy + 7, body_shadow);
}
