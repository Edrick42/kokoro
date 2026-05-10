//! Moluun skin — warm forest mammal (koala/wombat/red panda).
//!
//! Structural evolution (pokemon-style body plan change):
//! - Cub:   almost ALL head, tiny body underneath, no arms, stubs for feet
//! - Young: head shrinks relative, body elongates, short arms appear, ears grow out
//! - Adult: body dominates, defined limbs, broad shoulders, full features
//! - Elder: slightly hunched, thinner, fading resonance

use bevy::prelude::Res;
use image::{RgbaImage, Rgba};
use kokoro_art_palette::Palette;
use kokoro_art_palette::dsl::{Brush, BumpyDome, RingedTail, TaperedTail, outline_silhouette};
use kokoro_rig::Vec2;
use crate::creature::interaction::soft_body::SoftBody;
use crate::mind::MoodState;
use super::{SpeciesSkin, fill_circle, fill_rect, fill_ellipse, put, draw_eyes, fade};

// Master palette anchors (docs/aesthetic-targets.md §3 + §4.1)
const HIGHLIGHT:        Rgba<u8> = Rgba(Palette::OffWhite.rgba(200));
const NOSE_COLOR:       Rgba<u8> = Rgba(Palette::DeepBrown.rgba(255));
// Blush follows kawaii_factor (§4b): full at Cub (now inlined in
// draw_cub's rig path), subtle at Young, absent past that.
const BLUSH_SUBTLE:     Rgba<u8> = Rgba(Palette::CoralPink.rgba(128));
const EAR_INNER:        Rgba<u8> = Rgba(Palette::Tan.rgba(255));
const RESONANCE:        Rgba<u8> = Rgba(Palette::CyanBright.rgba(80));
const RESONANCE_BRIGHT: Rgba<u8> = Rgba(Palette::CyanBright.rgba(120));
const EAR_GLOW:         Rgba<u8> = Rgba(Palette::CyanBright.rgba(100));

// ===================================================================
// EGG
// ===================================================================

pub fn draw_egg(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32) {
    let cy = 30;
    // Egg silhouette via elliptical BumpyDome — very low bumpiness so the
    // shell still reads as smooth, but the silhouette no longer feels like
    // a perfect mathematical ellipse.
    BumpyDome::new(cx, cy, 13, Palette::Gold)
        .with_height(17)
        .with_bumpiness(0.08)
        .with_bumps(8)
        .with_seed(101)
        .paint_with(img, p.egg, None);
    fill_ellipse(img, cx, cy + 8, 10, 5, fade(p.egg, 0.15));
    fill_circle(img, cx - 5, cy - 6, 3, p.egg_spot);
    fill_circle(img, cx + 6, cy - 2, 3, p.egg_spot);
    fill_circle(img, cx - 2, cy + 5, 2, p.egg_spot);
    fill_circle(img, cx + 3, cy + 8, 2, p.egg_spot);
    put(img, cx + 1, cy - 12, fade(p.egg, 0.3));
    put(img, cx + 2, cy - 11, fade(p.egg, 0.3));
}

// ===================================================================
// CUB — quadrupedal side view, ref-faithful red panda palette
// ===================================================================
// Pose: head-right, tail-left curled down, 2 visible legs hanging.
// 4-color palette per the chosen reference (cross-stitch / pixel-art
// pattern):
//   - NearBlack      → outline, eye slit, snout, tear streak
//   - OffWhite       → face mask (dominates the face), inner ear, paw tips
//   - OrangeDark     → top of body, head crown, dark tail bands, ear outer
//   - Orange         → bottom of body, light tail bands
//
// Body is BICOLOR HARD: top half OrangeDark, bottom half Orange. Head's
// face mask covers nearly the entire face leaving only the crown dark.
// Tear streaks (the red panda hallmark) drop from below each eye.

pub fn draw_cub(img: &mut RgbaImage, _p: &SpeciesSkin, cx: i32, mood: &MoodState, _sb: &Option<Res<SoftBody>>) {
    // Place the rig at the canvas. The skeleton's default root sits at
    // (22, 36) so the cub silhouette spans roughly x=8..58 — already
    // centred horizontally for cx=32. We just shift by (cx-32, 0) so a
    // caller-provided cx still works.
    let mut skel = crate::visuals::rigs::moluun::cub_skeleton();
    skel.set_root(Vec2::new(22.0 + (cx as f32 - 32.0), 36.0));
    skel.forward();

    // Helpers — turn world Vec2s into the (i32, i32) tuples the brushes
    // expect. Saves repeating the cast in every call below.
    let pt = |v: Vec2| (v.x.round() as i32, v.y.round() as i32);
    let world_base = |name: &str| skel.id_of(name).map(|id| pt(skel.world_base(id)));
    let world_tip = |name: &str| skel.id_of(name).map(|id| pt(skel.world_tip(id)));
    let midpoint = |name: &str| {
        skel.id_of(name).map(|id| {
            let b = skel.world_base(id);
            let t = skel.world_tip(id);
            ((b.x + t.x) * 0.5, (b.y + t.y) * 0.5)
        })
    };

    // ====== Z-ORDERED PAINTING (back-to-front) ======

    // ---- BACK LEG (z=-1) — paint first, body covers the attach point.
    if let (Some(h), Some(p)) = (world_base("hip_back"), world_tip("paw_back")) {
        TaperedTail::new(h, p, 2, Palette::OrangeDark).paint(img);
        // White paw tip
        fill_rect(img, p.0 - 1, p.1 - 1, 3, 2, Rgba(Palette::OffWhite.rgba(255)));
    }

    // ---- BACK EAR (z=0, behind head) — only the tip pokes out.
    if let (Some(b), Some(t)) = (world_base("ear_back"), world_tip("ear_back")) {
        TaperedTail::new(b, t, 2, Palette::OrangeDark).paint(img);
    }

    // ---- TAIL (z=0, behind body but in front of nothing relevant) ----
    // Banded: alternating OrangeDark / Orange along arc length.
    let mut tail_joints: Vec<(i32, i32)> = ["tail_1", "tail_2", "tail_3", "tail_4", "tail_5"]
        .iter()
        .filter_map(|name| world_base(name))
        .collect();
    if let Some((tx, ty)) = world_tip("tail_5") {
        tail_joints.push((tx, ty));
    }
    if tail_joints.len() >= 2 {
        RingedTail::new(tail_joints, 3, Palette::OrangeDark, Palette::Orange)
            .with_rings(3, 2)
            .paint(img);
    }

    // ---- BODY (z=0) — bicolor: top half OrangeDark, bottom half Orange ----
    // The body is a horizontal mass running from pelvis → shoulders.
    // Paint as two stacked elliptical BumpyDomes (top dark, bottom light).
    if let (Some(pelvis), Some(shoulder)) =
        (world_base("pelvis"), world_base("shoulder_front"))
    {
        let body_cx = (pelvis.0 + shoulder.0) / 2;
        let body_cy = (pelvis.1 + shoulder.1) / 2;
        let body_half_len = ((shoulder.0 - pelvis.0).abs() / 2 + 4) as u32; // a bit wider than spine
        // Top half (dark) — sits 2px above body centre.
        BumpyDome::new(body_cx, body_cy - 2, body_half_len, Palette::OrangeDark)
            .with_height(5)
            .with_bumpiness(0.15)
            .with_bumps(10)
            .with_seed(11)
            .paint(img);
        // Bottom half (light) — sits 3px below body centre, slightly
        // narrower so it tucks under the dark band.
        BumpyDome::new(body_cx, body_cy + 3, body_half_len - 1, Palette::Orange)
            .with_height(4)
            .with_bumpiness(0.15)
            .with_bumps(10)
            .with_seed(13)
            .paint(img);
    }

    // ---- HEAD (z=+1) — dark crown + white face mask ----
    if let Some((hx, hy)) = midpoint("head") {
        // Crown — OrangeDark dome, top of head only.
        BumpyDome::new(hx as i32, hy as i32 - 1, 7, Palette::OrangeDark)
            .with_bumpiness(0.20)
            .with_bumps(9)
            .with_seed(3)
            .paint(img);
        // FACE MASK — OffWhite covering basically the whole face. Sits
        // slightly down from the head centre so the crown stays dark on
        // top.
        BumpyDome::new(hx as i32, hy as i32 + 2, 6, Palette::OffWhite)
            .with_bumpiness(0.10)
            .with_bumps(8)
            .with_seed(23)
            .paint(img);
    }

    // ---- FRONT EAR (z=+2, on top of head) ----
    if let (Some(b), Some(t)) = (world_base("ear_front"), world_tip("ear_front")) {
        TaperedTail::new(b, t, 2, Palette::OrangeDark).paint(img);
        // Inner-ear tiny white at the tip
        fill_rect(img, t.0, t.1, 1, 2, Rgba(Palette::OffWhite.rgba(255)));
    }

    // ---- EYE (z=+2) — single black slit (red panda half-closed look) ----
    let _ = mood; // mood-driven eye variants land in a follow-up
    if let Some((ex, ey)) = world_base("eye") {
        // 3-pixel horizontal slit
        fill_rect(img, ex - 1, ey, 3, 1, Rgba(Palette::NearBlack.rgba(255)));
        // TEAR STREAK — vertical 2-3 px line dropping straight down from
        // below the eye. THE red-panda identity feature.
        put(img, ex, ey + 2, Rgba(Palette::NearBlack.rgba(255)));
        put(img, ex, ey + 3, Rgba(Palette::NearBlack.rgba(255)));
    }

    // ---- SNOUT (z=+2) — small black wedge at the front of the face ----
    if let Some((nx, ny)) = world_base("snout") {
        fill_rect(img, nx - 1, ny, 2, 1, Rgba(Palette::NearBlack.rgba(255)));
    }

    // ---- FRONT LEG (z=+1) — paints after body so it appears in front ----
    if let (Some(s), Some(p)) = (world_base("shoulder_front"), world_tip("paw_front")) {
        TaperedTail::new(s, p, 2, Palette::OrangeDark).paint(img);
        fill_rect(img, p.0 - 1, p.1 - 1, 3, 2, Rgba(Palette::OffWhite.rgba(255)));
    }

    // ---- OUTLINE PASS — NearBlack 1px border (matches ref's hard outline) ----
    outline_silhouette(img, Rgba(Palette::NearBlack.rgba(255)));
}

// ===================================================================
// YOUNG — head shrinks relative, body grows, arms appear
// ===================================================================
// Head and body are now ~equal size. The creature is elongating.
// Short arms sprout. Ears grow noticeably. Feet get bigger.
// Think Charmeleon — awkward middle stage, clearly transitioning.

pub fn draw_young(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // Soft body positions
    let (hx, hy) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 18));
    let (bx, by) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 32));
    let (sl_x, sl_y) = sb.as_ref().map(|b| b.point("shoulder_l").px()).unwrap_or((cx - 13, 29));
    let (sr_x, sr_y) = sb.as_ref().map(|b| b.point("shoulder_r").px()).unwrap_or((cx + 13, 29));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 6, 43));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 6, 43));
    let hr = 13;
    let body_r = 12;

    // Ears (on head, soft body position)
    fill_circle(img, hx - 10, hy - 10, 5, p.accent);
    fill_circle(img, hx + 10, hy - 10, 5, p.accent);
    fill_circle(img, hx - 10, hy - 10, 3, EAR_INNER);
    fill_circle(img, hx + 10, hy - 10, 3, EAR_INNER);
    put(img, hx - 12, hy - 13, EAR_GLOW);
    put(img, hx + 12, hy - 13, EAR_GLOW);

    // Head (soft body position) — BumpyDome with low bumpiness; head still
    // dominates but no longer reads as a perfect circle. Different seed than
    // the cub head so the silhouette evolves visibly across stages.
    BumpyDome::new(hx, hy, hr as u32, Palette::Gold)
        .with_bumpiness(0.22)
        .with_bumps(9)
        .with_seed(11)
        .paint_with(img, p.body, None);
    for &(dx, dy) in &[(-6,-4), (5,-3), (-3,-8)] {
        put(img, hx + dx, hy + dy, p.accent);
    }

    // Neck — filled zone between head and body centers (NEVER gaps)
    let neck_cx = (hx + bx) / 2;
    let neck_top = hy.min(by);
    let neck_bottom = hy.max(by);
    let neck_height = (neck_bottom - neck_top).max(1);
    fill_rect(img, neck_cx - 5, neck_top, 11, neck_height, p.body);

    // Body (soft body position) — BumpyDome too, slightly more bumpy than
    // the head so the silhouette suggests fluffier torso fur.
    BumpyDome::new(bx, by, body_r as u32, Palette::Gold)
        .with_bumpiness(0.30)
        .with_bumps(10)
        .with_seed(19)
        .paint_with(img, p.body, None);
    for &(dx, dy) in &[(-5,-3), (4,2), (-3,5), (6,-1), (-7,1)] {
        put(img, bx + dx, by + dy, p.accent);
    }
    fill_circle(img, bx, by + 2, 8, p.body_light);

    // Kokoro-sac
    fill_circle(img, bx, by + 1, 4, RESONANCE);

    // Arms (soft body positions)
    fill_rect(img, sl_x, sl_y, 4, 6, p.body);
    fill_rect(img, sr_x - 3, sr_y, 4, 6, p.body);

    // Feet (soft body positions)
    fill_circle(img, fl_x, fl_y, 4, p.body);
    fill_circle(img, fr_x, fr_y, 4, p.body);
    for dx in [-1, 1] {
        put(img, fl_x + dx, fl_y + 3, p.accent);
        put(img, fr_x + dx, fr_y + 3, p.accent);
    }

    // Face (on head)
    draw_eyes(img, hx, hy + 2, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, hx - 5, hy + 2, HIGHLIGHT);
        put(img, hx + 2, hy + 2, HIGHLIGHT);
    }

    fill_rect(img, hx - 10, hy + 5, 2, 2, BLUSH_SUBTLE);
    fill_rect(img, hx + 9, hy + 5, 2, 2, BLUSH_SUBTLE);

    fill_rect(img, hx - 1, hy + 8, 3, 2, NOSE_COLOR);
    put(img, hx, hy + 10, NOSE_COLOR);

    // Mouth (mood-reactive)
}

// ===================================================================
// ADULT — body dominates, full limbs, broad build
// ===================================================================
// Body is now clearly larger than head. Defined neck/shoulders.
// Full arms with paws. Thick legs with toe pads. Dense fur everywhere.
// Think Charizard-level transformation — powerful, mature, complete.

pub fn draw_adult(img: &mut RgbaImage, p: &SpeciesSkin, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // === SOFT BODY POSITIONS ===
    // All positions come from physics simulation. Fallback to defaults if no soft body.
    let (hx, hy) = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 14));
    let (bx, by) = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 30));
    let (belly_x, belly_y) = sb.as_ref().map(|b| b.point("belly").px()).unwrap_or((cx, 33));
    let (sl_x, sl_y) = sb.as_ref().map(|b| b.point("shoulder_l").px()).unwrap_or((cx - 17, 26));
    let (pl_x, pl_y) = sb.as_ref().map(|b| b.point("paw_l").px()).unwrap_or((cx - 17, 36));
    let (sr_x, sr_y) = sb.as_ref().map(|b| b.point("shoulder_r").px()).unwrap_or((cx + 17, 26));
    let (pr_x, pr_y) = sb.as_ref().map(|b| b.point("paw_r").px()).unwrap_or((cx + 17, 36));
    let (fl_x, fl_y) = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 8, 46));
    let (fr_x, fr_y) = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 8, 46));
    let (ear_x, ear_y) = sb.as_ref().map(|b| b.point("ear_anchor").px()).unwrap_or((cx, 6));

    let body_r = 13;
    let hr = 11;

    // === DRAW ORDER: body first (bottom), then neck, then head on top ===

    // Body FIRST (pinned anchor — always in position) — BumpyDome with the
    // tightest bumpiness in the lifecycle (0.18) so the adult silhouette
    // reads as solid, mature fur rather than fluffy down.
    BumpyDome::new(bx, by, body_r as u32, Palette::Gold)
        .with_bumpiness(0.18)
        .with_bumps(11)
        .with_seed(31)
        .paint_with(img, p.body, None);

    // Neck — thick rectangle connecting body center to head center.
    // Drawn OVER body, UNDER head. ALWAYS visible, NEVER a gap.
    let neck_cx = (hx + bx) / 2;
    let neck_top = hy;               // from head center
    let neck_bottom = by;            // to body center
    let neck_y = neck_top.min(neck_bottom);
    let neck_h = (neck_top - neck_bottom).unsigned_abs() as i32 + 1;
    fill_rect(img, neck_cx - 7, neck_y, 15, neck_h, p.body);

    // Head ON TOP of neck (so it covers the joint) — BumpyDome continuing
    // the curve. Slightly more bumpy than body so head fur catches more
    // light than torso fur.
    BumpyDome::new(hx, hy, hr as u32, Palette::Gold)
        .with_bumpiness(0.20)
        .with_bumps(10)
        .with_seed(43)
        .paint_with(img, p.body, None);
    for &(dx, dy) in &[(-5,-4), (4,-3), (-2,-7), (3,5)] {
        put(img, hx + dx, hy + dy, p.accent);
    }

    // Ears (on head)
    fill_circle(img, ear_x - 10, ear_y, 7, p.accent);
    fill_circle(img, ear_x + 10, ear_y, 7, p.accent);
    fill_circle(img, ear_x - 10, ear_y, 4, EAR_INNER);
    fill_circle(img, ear_x + 10, ear_y, 4, EAR_INNER);
    put(img, ear_x - 12, ear_y - 5, EAR_GLOW);
    put(img, ear_x + 12, ear_y - 5, EAR_GLOW);
    // Body fur dots — only those clearly OUTSIDE the head circle, so the head
    // never gets accent-color pixels splashed on top of it.
    for &(dx, dy) in &[(-8,-6), (7,-4), (-5,3), (9,1), (-3,-10), (6,7), (-10,4), (4,-7), (-6,9)] {
        let px = bx + dx;
        let py = by + dy;
        let dist_sq = (px - hx).pow(2) + (py - hy).pow(2);
        if dist_sq > hr * hr {
            put(img, px, py, p.accent);
        }
    }

    // Belly (oscillates with breathing via soft body)
    fill_circle(img, belly_x, belly_y, 9, p.body_light);
    for &(dx, dy) in &[(-3, 2), (4, 3), (-1, 5), (2, 7)] {
        put(img, belly_x + dx, belly_y + dy, fade(p.body_light, 0.1));
    }

    // Kokoro-sac glow (on belly)
    fill_circle(img, belly_x, belly_y - 1, 5, RESONANCE_BRIGHT);
    fill_circle(img, belly_x, belly_y - 1, 3, RESONANCE);

    // Left arm (shoulder → paw, soft body positions)
    let al_h = (pl_y - sl_y).max(3);
    fill_rect(img, sl_x, sl_y, 5, al_h, p.body);
    fill_circle(img, pl_x, pl_y, 3, p.accent);

    // Right arm
    let ar_h = (pr_y - sr_y).max(3);
    fill_rect(img, sr_x - 4, sr_y, 5, ar_h, p.body);
    fill_circle(img, pr_x, pr_y, 3, p.accent);

    // Left foot (soft body position)
    fill_circle(img, fl_x, fl_y, 6, p.body);
    for dx in [-3, -1, 1, 3] {
        put(img, fl_x + dx, fl_y + 5, p.accent);
    }

    // Right foot
    fill_circle(img, fr_x, fr_y, 6, p.body);
    for dx in [-3, -1, 1, 3] {
        put(img, cx + 8 + dx, fr_y + 5, p.accent);
    }

    // === FACE (on head, moves with head) ===

    // Eyes
    draw_eyes(img, hx, hy + 2, 5, 3, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, hx - 4, hy + 2, HIGHLIGHT);
        put(img, hx + 2, hy + 2, HIGHLIGHT);
        put(img, hx - 7, hy + 3, RESONANCE);
        put(img, hx + 7, hy + 3, RESONANCE);
    }

    // No blush at adult stage — kawaii_factor reaches 0.0 (docs §4c).

    // Nose (on head)
    fill_rect(img, hx - 2, hy + 8, 5, 2, NOSE_COLOR);
    fill_rect(img, hx - 1, hy + 10, 3, 1, NOSE_COLOR);
    put(img, hx, hy + 11, NOSE_COLOR);

    // Mouth drawn centrally in draw_creature
}

// ===================================================================
// ELDER — own silhouette per docs/aesthetic-targets.md §4d
// ===================================================================
// Reference 14 (urso marrom em pé): plump, grounded, slit eyes, neutral pose.
// Elder is NOT "adult dessaturated". It has its own anatomy:
//   - kawaii_factor 0.2 → roundness 0.55, head/body 0.35
//   - hunched: head sits lower, neck shorter, limbs tucked close
//   - palette reduced to 5 colors (Brown ramp + Cream + NearBlack)
//   - slit eyes (1px tall), no glint, no blush, no accent cyan
//   - reads as "ancião marrom" — Gold/accent fades into Brown for the silhouette

pub fn draw_elder(img: &mut RgbaImage, cx: i32, mood: &MoodState, sb: &Option<Res<SoftBody>>) {
    // Elder palette per §4d.3 — Moluun's 5 colors only.
    let body: Rgba<u8>        = Palette::Brown.into();
    let body_shadow: Rgba<u8> = Palette::BrownDark.into();
    let belly: Rgba<u8>       = Palette::Cream.into();
    let eye: Rgba<u8>         = Palette::NearBlack.into();

    // Hunched anatomy — head sits lower (18 vs 14 adult), body sits lower
    // and slightly larger (32 / r14 vs 30 / r13 adult). Limbs tuck inward.
    let (hx, hy)         = sb.as_ref().map(|b| b.point("head").px()).unwrap_or((cx, 18));
    let (bx, by)         = sb.as_ref().map(|b| b.point("body").px()).unwrap_or((cx, 32));
    let (belly_x, belly_y) = sb.as_ref().map(|b| b.point("belly").px()).unwrap_or((cx, 35));
    let (sl_x, sl_y)     = sb.as_ref().map(|b| b.point("shoulder_l").px()).unwrap_or((cx - 14, 30));
    let (pl_x, pl_y)     = sb.as_ref().map(|b| b.point("paw_l").px()).unwrap_or((cx - 14, 36));
    let (sr_x, sr_y)     = sb.as_ref().map(|b| b.point("shoulder_r").px()).unwrap_or((cx + 14, 30));
    let (pr_x, pr_y)     = sb.as_ref().map(|b| b.point("paw_r").px()).unwrap_or((cx + 14, 36));
    let (fl_x, fl_y)     = sb.as_ref().map(|b| b.point("foot_l").px()).unwrap_or((cx - 6, 46));
    let (fr_x, fr_y)     = sb.as_ref().map(|b| b.point("foot_r").px()).unwrap_or((cx + 6, 46));

    // Body — plump, low bumpiness so the fur reads as "settled" rather than
    // fluffy. Brown ramp (not Gold) is the elder signature.
    BumpyDome::new(bx, by, 14, Palette::Brown)
        .with_bumpiness(0.12)
        .with_bumps(11)
        .with_seed(73)
        .paint_with(img, body, None);

    // Short, thick neck — drawn between head and body so it never gaps.
    let neck_cx = (hx + bx) / 2;
    let neck_top = hy.min(by);
    let neck_h = (hy - by).unsigned_abs() as i32 + 1;
    fill_rect(img, neck_cx - 6, neck_top, 13, neck_h, body);

    // Head — slightly smaller than adult head (10 vs 11), same Brown so the
    // creature reads as a single mass.
    BumpyDome::new(hx, hy, 10, Palette::Brown)
        .with_bumpiness(0.15)
        .with_bumps(9)
        .with_seed(89)
        .paint_with(img, body, None);

    // Drooped ears — BrownDark, no inner cyan glow.
    fill_circle(img, hx - 9, hy - 8, 4, body_shadow);
    fill_circle(img, hx + 9, hy - 8, 4, body_shadow);

    // Belly — quiet cream patch, smaller than adult.
    fill_circle(img, belly_x, belly_y, 7, belly);

    // Arms — tucked close (limb/body 0.30). TaperedTail follows the soft-body
    // shoulder→paw vector so a hunched arm stays connected when the body
    // shifts. base_width=2 keeps the arm thin (elder is not muscular).
    TaperedTail::new((sl_x, sl_y), (pl_x, pl_y), 2, Palette::Brown)
        .paint_with(img, body, None);
    TaperedTail::new((sr_x, sr_y), (pr_x, pr_y), 2, Palette::Brown)
        .paint_with(img, body, None);

    // Feet — grounded, plump.
    fill_circle(img, fl_x, fl_y, 5, body);
    fill_circle(img, fr_x, fr_y, 5, body);

    // Slit eyes — single 1px-tall horizontal line per eye, no glint. Sleeping
    // mood already collapses to identical silhouette so we let it through.
    let _ = mood;
    fill_rect(img, hx - 6, hy + 2, 4, 1, eye);
    fill_rect(img, hx + 2, hy + 2, 4, 1, eye);
}

