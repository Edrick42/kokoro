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
// Kept by Young/Adult/Elder rendering paths — not all are used by the
// rewritten cub blit, but pruning would create a cascade of churn.
#[allow(unused_imports)]
use kokoro_art_palette::dsl::{Brush, BumpyDome, RingedTail, TaperedTail, outline_silhouette};
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
// CUB — STATIC SPRITE BLIT FROM REFERENCE TRANSCRIPTION
// ===================================================================
// `assets/sprites/moluun_cub.png` is the user's chosen Pinterest pixel-art
// reference, transcribed via the `ref_transcribe` test below (4-color
// palette quantised + 5×5 majority sample per cell). That sprite is the
// authoritative cub look — drawing procedurally with brushes never got
// close. We just blit the bytes here.
//
// The rig + brush system stays available for future ANIMATION on top of
// this sprite (mood-driven eye blinks, breathing scale, etc.); the rest
// pose itself is no longer procedural.

const CUB_SPRITE_BYTES: &[u8] = include_bytes!("../../../assets/sprites/moluun_cub.png");

pub fn draw_cub(img: &mut RgbaImage, _p: &SpeciesSkin, cx: i32, _mood: &MoodState, _sb: &Option<Res<SoftBody>>) {
    // Decode once per call. PNG decoding for a 64×64 image is ~10 µs and
    // dwarfed by Bevy's per-frame work; not worth caching at this stage.
    let sprite = image::load_from_memory(CUB_SPRITE_BYTES)
        .expect("moluun_cub.png missing or corrupt — re-run transcribe_moluun_cub_ref")
        .to_rgba8();
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);

    // Sprite is authored centred at canvas (32, 32). Re-centre on `cx`
    // so callers that move the creature around still work.
    let off_x = cx - 32;
    let dst_w = img.width() as i32;
    let dst_h = img.height() as i32;

    for y in 0..sh {
        for x in 0..sw {
            let src_pixel = *sprite.get_pixel(x as u32, y as u32);
            if src_pixel.0[3] == 0 {
                continue; // transparent — let the caller's background show
            }
            let dx = x + off_x;
            let dy = y;
            if dx >= 0 && dy >= 0 && dx < dst_w && dy < dst_h {
                img.put_pixel(dx as u32, dy as u32, src_pixel);
            }
        }
    }
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

// ===================================================================
// REF TRANSCRIBER — converts the Pinterest cross-stitch JPG into a
// quantised sprite PNG. Run on demand:
//
//     cargo test transcribe_moluun_cub_ref -- --ignored --nocapture
//
// Reads the user-saved JPG, samples each grid cell's centre, snaps the
// colour to one of {transparent, NearBlack, OffWhite, OrangeDark, Orange},
// centres the result in a 64×64 RGBA buffer, and writes
// assets/sprites/moluun_cub.png. After generating, draw_cub blits that
// PNG instead of trying to recreate the look procedurally.
// ===================================================================
#[cfg(test)]
mod ref_transcribe {
    use image::{GenericImageView, Rgba, RgbaImage};
    use kokoro_art_palette::Palette;
    use std::path::PathBuf;

    /// Snap a sampled pixel to the closest of our four sprite colours,
    /// or to transparent. Includes the cross-stitch grid's two greens
    /// (cell-fill sage and darker grid line) as transparent candidates so
    /// noisy edge samples and grid lines drop out cleanly.
    fn quantise(r: u8, g: u8, b: u8) -> Option<Rgba<u8>> {
        // Two opaque candidate sets. Greens are "transparent" matches —
        // when one wins the nearest-distance contest, return None.
        const TRANSPARENT_GREENS: [(u8, u8, u8); 2] = [
            (200, 222, 188), // sage cell fill
            (160, 196, 154), // darker grid-line green
        ];
        let opaque: [Rgba<u8>; 4] = [
            Rgba(Palette::NearBlack.rgba(255)),
            Rgba(Palette::OffWhite.rgba(255)),
            Rgba(Palette::OrangeDark.rgba(255)),
            Rgba(Palette::Orange.rgba(255)),
        ];
        let dist_sq = |cr: u8, cg: u8, cb: u8| -> i32 {
            let dr = cr as i32 - r as i32;
            let dg = cg as i32 - g as i32;
            let db = cb as i32 - b as i32;
            dr * dr + dg * dg + db * db
        };
        let (mut best_d, mut best_color): (i32, Option<Rgba<u8>>) = (i32::MAX, None);
        for (cr, cg, cb) in TRANSPARENT_GREENS {
            let d = dist_sq(cr, cg, cb);
            if d < best_d {
                best_d = d;
                best_color = None; // transparent wins
            }
        }
        for c in opaque {
            let d = dist_sq(c.0[0], c.0[1], c.0[2]);
            if d < best_d {
                best_d = d;
                best_color = Some(c);
            }
        }
        best_color
    }

    /// 5×5 majority-vote sampling around `(px, py)`. Wide enough to ride
    /// over grid lines and JPEG noise; the float-grid centre keeps the
    /// window biased toward the cell's interior.
    fn sample_cell(src: &image::DynamicImage, px: u32, py: u32) -> Option<Rgba<u8>> {
        use image::GenericImageView;
        let (w, h) = src.dimensions();
        let mut votes: std::collections::HashMap<Option<[u8; 4]>, u32> =
            std::collections::HashMap::new();
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let nx = px as i32 + dx;
                let ny = py as i32 + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let p = src.get_pixel(nx as u32, ny as u32);
                let q = quantise(p[0], p[1], p[2]);
                let key = q.map(|r| r.0);
                *votes.entry(key).or_insert(0) += 1;
            }
        }
        votes
            .into_iter()
            .max_by_key(|(_, n)| *n)
            .and_then(|(k, _)| k)
            .map(Rgba)
    }

    /// Source path is hard-coded — this is a one-off transcription for a
    /// reference the user explicitly chose; if we change refs in the
    /// future, point this at the new file. Path is a PNG (the `image`
    /// crate is built without the jpeg feature in this project, so the
    /// original JPG must be pre-converted with `sips`).
    const SOURCE_JPG: &str = "/tmp/moluun_ref.png";

    #[test]
    #[ignore]
    fn transcribe_moluun_cub_ref() {
        let src = image::open(SOURCE_JPG)
            .expect("ref JPG missing — see SOURCE_JPG path");
        let (sw, sh) = src.dimensions();

        // The cross-stitch source is 331×300 with a 33×30 grid → cells
        // are ~10.03 × 10.0 pixels. Using integer 10 accumulates ≈1px of
        // drift across the row and starts catching grid lines instead of
        // cell centres. Float cell size + floored centre keeps the sample
        // point inside each cell's interior every time.
        let cols = 33u32;
        let rows = 30u32;
        let cell_w = sw as f32 / cols as f32;
        let cell_h = sh as f32 / rows as f32;
        eprintln!(
            "source {sw}×{sh}, grid {cols}×{rows}, cell {cell_w:.2}×{cell_h:.2}px"
        );

        let mut quantised: Vec<Vec<Option<Rgba<u8>>>> =
            vec![vec![None; cols as usize]; rows as usize];
        for cy in 0..rows {
            for cx in 0..cols {
                let px = ((cx as f32 + 0.5) * cell_w).floor() as u32;
                let py = ((cy as f32 + 0.5) * cell_h).floor() as u32;
                quantised[cy as usize][cx as usize] = sample_cell(&src, px, py);
            }
        }

        // Iterative denoise: drops opaque cells with fewer than 2
        // same-coloured 8-neighbours. Two passes converge on a clean
        // silhouette.
        //
        // EXCEPT: NearBlack pixels are spared. Eyes and noses in pixel
        // art are typically 1-2 black pixels sitting alone inside a
        // face mask of a different colour. Denoising them off the sprite
        // strips the face of its key features. Other colours (orange,
        // cream) always appear in larger clusters, so they obey the rule.
        let near_black: [u8; 4] = Palette::NearBlack.rgba(255);
        for _pass in 0..2 {
            let snapshot = quantised.clone();
            for cy in 0..rows as usize {
                for cx in 0..cols as usize {
                    let cur = snapshot[cy][cx];
                    let Some(c) = cur else { continue };
                    if c.0 == near_black {
                        continue;
                    }
                    let mut same = 0u32;
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = cx as i32 + dx;
                            let ny = cy as i32 + dy;
                            if nx < 0 || ny < 0 || nx >= cols as i32 || ny >= rows as i32 {
                                continue;
                            }
                            if snapshot[ny as usize][nx as usize] == cur {
                                same += 1;
                            }
                        }
                    }
                    if same < 2 {
                        quantised[cy][cx] = None;
                    }
                }
            }
        }

        // Bounding box of opaque cells so we can centre the sprite.
        let (mut min_x, mut min_y, mut max_x, mut max_y) =
            (cols as i32, rows as i32, -1i32, -1i32);
        for cy in 0..rows as i32 {
            for cx in 0..cols as i32 {
                if quantised[cy as usize][cx as usize].is_some() {
                    if cx < min_x { min_x = cx; }
                    if cy < min_y { min_y = cy; }
                    if cx > max_x { max_x = cx; }
                    if cy > max_y { max_y = cy; }
                }
            }
        }
        let sprite_w = (max_x - min_x + 1).max(1);
        let sprite_h = (max_y - min_y + 1).max(1);
        eprintln!("sprite bounding box {sprite_w}×{sprite_h} cells");

        // Centre in a 64×64 canvas. If the sprite is bigger than 64 in
        // any axis, clip from the edges (shouldn't happen with this ref).
        let canvas = 64i32;
        let off_x = (canvas - sprite_w) / 2 - min_x;
        let off_y = (canvas - sprite_h) / 2 - min_y;
        let mut out = RgbaImage::new(64, 64);
        for cy in min_y..=max_y {
            for cx in min_x..=max_x {
                if let Some(c) = quantised[cy as usize][cx as usize] {
                    let dx = cx + off_x;
                    let dy = cy + off_y;
                    if dx >= 0 && dy >= 0 && dx < canvas && dy < canvas {
                        out.put_pixel(dx as u32, dy as u32, c);
                    }
                }
            }
        }

        let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("sprites");
        std::fs::create_dir_all(&out_dir).unwrap();
        out.save(out_dir.join("moluun_cub.png")).unwrap();

        // Save a 4× upscale alongside for easy review.
        let mut up = RgbaImage::new(256, 256);
        for y in 0..64u32 {
            for x in 0..64u32 {
                let p = *out.get_pixel(x, y);
                for dy in 0..4u32 {
                    for dx in 0..4u32 {
                        up.put_pixel(x * 4 + dx, y * 4 + dy, p);
                    }
                }
            }
        }
        let snap_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("sprite-snapshots");
        std::fs::create_dir_all(&snap_dir).unwrap();
        up.save(snap_dir.join("moluun_cub_from_ref@4x.png")).unwrap();

        eprintln!(
            "wrote {} and target/sprite-snapshots/moluun_cub_from_ref@4x.png",
            out_dir.join("moluun_cub.png").display()
        );
    }
}

