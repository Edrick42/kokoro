//! Moluun skin — warm forest mammal (koala/wombat/red panda).
//!
//! Structural evolution (pokemon-style body plan change):
//! - Cub:   almost ALL head, tiny body underneath, no arms, stubs for feet
//! - Young: head shrinks relative, body elongates, short arms appear, ears grow out
//! - Adult: body dominates, defined limbs, broad shoulders, full features
//! - Elder: slightly hunched, thinner, fading resonance

use image::{RgbaImage, Rgba};
use crate::mind::MoodState;
use super::{Palette, fill_circle, fill_rect, fill_ellipse, put, draw_eyes, fade};

const HIGHLIGHT: Rgba<u8> = Rgba([255, 255, 255, 200]);
const NOSE_COLOR: Rgba<u8> = Rgba([50, 35, 30, 255]);
const BLUSH: Rgba<u8> = Rgba([220, 150, 140, 255]);
const EAR_INNER: Rgba<u8> = Rgba([200, 160, 155, 255]);
const RESONANCE: Rgba<u8> = Rgba([160, 210, 230, 80]);
const RESONANCE_BRIGHT: Rgba<u8> = Rgba([170, 220, 240, 120]);
const EAR_GLOW: Rgba<u8> = Rgba([140, 200, 220, 100]);

// ===================================================================
// EGG
// ===================================================================

pub fn draw_egg(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    fill_ellipse(img, cx, cy, 13, 17, p.egg);
    fill_ellipse(img, cx, cy + 8, 10, 5, fade(p.egg, 0.15));
    fill_circle(img, cx - 5, cy - 6, 3, p.egg_spot);
    fill_circle(img, cx + 6, cy - 2, 3, p.egg_spot);
    fill_circle(img, cx - 2, cy + 5, 2, p.egg_spot);
    fill_circle(img, cx + 3, cy + 8, 2, p.egg_spot);
    put(img, cx + 1, cy - 12, fade(p.egg, 0.3));
    put(img, cx + 2, cy - 11, fade(p.egg, 0.3));
}

// ===================================================================
// CUB — 80% head, tiny body, stubs, no arms
// ===================================================================
// Think Togepi / baby Kirby — a round head with a small body attached below.
// Head radius ~16px, body just a small bump underneath (~8px).
// Ears tiny. Feet are round stubs. No arms at all.

pub fn draw_cub(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    // HEAD is the dominant shape — centered high
    let hy = 20;   // head center (high up)
    let hr = 16;   // head radius (BIG)

    // Tiny ears (nubs on top of the big head)
    fill_circle(img, cx - 10, hy - 12, 4, p.accent);
    fill_circle(img, cx + 10, hy - 12, 4, p.accent);
    fill_circle(img, cx - 10, hy - 12, 2, EAR_INNER);
    fill_circle(img, cx + 10, hy - 12, 2, EAR_INNER);

    // BIG round head
    fill_circle(img, cx, hy, hr, p.body);

    // Soft fur on head (sparse)
    for &(dx, dy) in &[(-7,-5), (7,-4), (-3,-10), (6,7)] {
        put(img, cx + dx, hy + dy, p.accent);
    }

    // Small body underneath the head (like a pouch/bump)
    let body_y = hy + hr + 4;  // below the head
    fill_ellipse(img, cx, body_y, 10, 7, p.body);
    // Belly on the small body
    fill_ellipse(img, cx, body_y + 1, 7, 5, p.body_light);

    // Kokoro-sac glow (faint — just forming)
    fill_circle(img, cx, body_y, 3, RESONANCE);

    // Tiny stub feet poking out under the body
    fill_circle(img, cx - 5, body_y + 6, 3, p.body);
    fill_circle(img, cx + 5, body_y + 6, 3, p.body);

    // NO arms

    // HUGE eyes on the big head (6x6)
    draw_eyes(img, cx, hy + 2, 6, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 6, hy + 2, HIGHLIGHT);
        put(img, cx - 5, hy + 2, HIGHLIGHT);
        put(img, cx + 2, hy + 2, HIGHLIGHT);
        put(img, cx + 3, hy + 2, HIGHLIGHT);
    }

    // Big blush (baby cheeks)
    fill_rect(img, cx - 12, hy + 6, 3, 2, BLUSH);
    fill_rect(img, cx + 10, hy + 6, 3, 2, BLUSH);

    // Tiny nose (just a dot)
    put(img, cx, hy + 10, NOSE_COLOR);
    put(img, cx + 1, hy + 10, NOSE_COLOR);

    // Tiny mouth
    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        put(img, cx - 1, hy + 12, p.mouth);
        put(img, cx + 1, hy + 12, p.mouth);
    } else if *mood != MoodState::Sleeping {
        put(img, cx, hy + 12, p.mouth);
    }
}

// ===================================================================
// YOUNG — head shrinks relative, body grows, arms appear
// ===================================================================
// Head and body are now ~equal size. The creature is elongating.
// Short arms sprout. Ears grow noticeably. Feet get bigger.
// Think Charmeleon — awkward middle stage, clearly transitioning.

pub fn draw_young(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    // Head and body roughly equal now
    let hy = 18;    // head center
    let hr = 13;    // head radius (shrunk from 16)
    let body_y = 32; // body center (lower, separated)
    let body_r = 12; // body radius (grown from 7-height ellipse)

    // Ears growing out (bigger than cub, clearly visible)
    fill_circle(img, cx - 10, hy - 10, 5, p.accent);
    fill_circle(img, cx + 10, hy - 10, 5, p.accent);
    fill_circle(img, cx - 10, hy - 10, 3, EAR_INNER);
    fill_circle(img, cx + 10, hy - 10, 3, EAR_INNER);
    // Faint ear glow starting
    put(img, cx - 12, hy - 13, EAR_GLOW);
    put(img, cx + 12, hy - 13, EAR_GLOW);

    // Head (smaller proportionally)
    fill_circle(img, cx, hy, hr, p.body);
    // Fur on head
    for &(dx, dy) in &[(-6,-4), (5,-3), (-3,-8)] {
        put(img, cx + dx, hy + dy, p.accent);
    }

    // Neck connecting head to body (new! didn't exist in cub)
    fill_rect(img, cx - 5, hy + hr - 2, 11, 5, p.body);

    // Body (round, growing)
    fill_circle(img, cx, body_y, body_r, p.body);
    // Fur texture on body
    for &(dx, dy) in &[(-5,-3), (4,2), (-3,5), (6,-1), (-7,1)] {
        put(img, cx + dx, body_y + dy, p.accent);
    }
    // Belly
    fill_circle(img, cx, body_y + 2, 8, p.body_light);

    // Kokoro-sac glow (brighter)
    fill_circle(img, cx, body_y + 1, 4, RESONANCE);
    // Resonance lines starting on flanks
    put(img, cx - 9, body_y - 2, RESONANCE);
    put(img, cx + 9, body_y - 2, RESONANCE);

    // Short arms! (new feature — stubby but visible)
    fill_rect(img, cx - body_r - 1, body_y - 3, 4, 6, p.body);
    fill_rect(img, cx + body_r - 2, body_y - 3, 4, 6, p.body);

    // Feet (bigger than cub, pads starting)
    fill_circle(img, cx - 6, body_y + body_r - 1, 4, p.body);
    fill_circle(img, cx + 6, body_y + body_r - 1, 4, p.body);
    for dx in [-1, 1] {
        put(img, cx - 6 + dx, body_y + body_r + 2, p.accent);
        put(img, cx + 6 + dx, body_y + body_r + 2, p.accent);
    }

    // Eyes — 5x5 on head (maturing)
    draw_eyes(img, cx, hy + 2, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 5, hy + 2, HIGHLIGHT);
        put(img, cx + 2, hy + 2, HIGHLIGHT);
    }

    // Blush
    fill_rect(img, cx - 10, hy + 5, 2, 2, BLUSH);
    fill_rect(img, cx + 9, hy + 5, 2, 2, BLUSH);

    // Nose forming triangle shape
    fill_rect(img, cx - 1, hy + 8, 3, 2, NOSE_COLOR);
    put(img, cx, hy + 10, NOSE_COLOR);

    // Mouth
    if *mood != MoodState::Sleeping {
        put(img, cx - 1, hy + 12, p.mouth);
        put(img, cx + 1, hy + 12, p.mouth);
    }
}

// ===================================================================
// ADULT — body dominates, full limbs, broad build
// ===================================================================
// Body is now clearly larger than head. Defined neck/shoulders.
// Full arms with paws. Thick legs with toe pads. Dense fur everywhere.
// Think Charizard-level transformation — powerful, mature, complete.

pub fn draw_adult(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    // Body-dominant proportions
    let hy = 14;    // head center (high up, smaller)
    let hr = 11;    // head radius (much smaller than cub's 16!)
    let body_y = 30; // body center
    let body_r = 16; // body radius (dominant)

    // Full-sized ears with detailed pink inner
    fill_circle(img, cx - 10, hy - 8, 7, p.accent);
    fill_circle(img, cx + 10, hy - 8, 7, p.accent);
    fill_circle(img, cx - 10, hy - 8, 4, EAR_INNER);
    fill_circle(img, cx + 10, hy - 8, 4, EAR_INNER);
    // Bioluminescent ear tips
    put(img, cx - 12, hy - 13, EAR_GLOW);
    put(img, cx - 11, hy - 14, EAR_GLOW);
    put(img, cx + 12, hy - 13, EAR_GLOW);
    put(img, cx + 11, hy - 14, EAR_GLOW);

    // Head
    fill_circle(img, cx, hy, hr, p.body);
    for &(dx, dy) in &[(-5,-4), (4,-3), (-2,-7), (3,5)] {
        put(img, cx + dx, hy + dy, p.accent);
    }

    // Thick neck / shoulders connecting to body
    fill_rect(img, cx - 7, hy + hr - 2, 15, 6, p.body);

    // Large body
    fill_circle(img, cx, body_y, body_r, p.body);

    // Dense fur texture
    for &(dx, dy) in &[(-8,-6), (7,-4), (-5,3), (9,1), (-3,-10), (6,7), (-10,4), (4,-7), (-6,9)] {
        put(img, cx + dx, body_y + dy, p.accent);
    }

    // Warm belly with fur detail
    fill_circle(img, cx, body_y + 3, 11, p.body_light);
    for &(dx, dy) in &[(-3, 2), (4, 3), (-1, 5), (2, 7)] {
        put(img, cx + dx, body_y + 3 + dy, fade(p.body_light, 0.1));
    }

    // Kokoro-sac glow — bright core
    fill_circle(img, cx, body_y + 2, 5, RESONANCE_BRIGHT);
    fill_circle(img, cx, body_y + 2, 3, RESONANCE);

    // Resonance lines along flanks
    for &(dx, dy) in &[(-12,-2), (-13,0), (-14,2), (12,-2), (13,0), (14,2)] {
        put(img, cx + dx, body_y + dy, RESONANCE);
    }

    // Full arms with paws
    fill_rect(img, cx - body_r - 1, body_y - 4, 5, 10, p.body);
    fill_rect(img, cx + body_r - 3, body_y - 4, 5, 10, p.body);
    fill_circle(img, cx - body_r, body_y + 6, 3, p.accent);
    fill_circle(img, cx + body_r, body_y + 6, 3, p.accent);

    // Sturdy feet with toe pads
    fill_circle(img, cx - 8, body_y + body_r - 1, 6, p.body);
    fill_circle(img, cx + 8, body_y + body_r - 1, 6, p.body);
    for dx in [-3, -1, 1, 3] {
        put(img, cx - 8 + dx, body_y + body_r + 4, p.accent);
        put(img, cx + 8 + dx, body_y + body_r + 4, p.accent);
    }

    // Eyes on head — 5x5 with glow aura
    draw_eyes(img, cx, hy + 2, 5, 3, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 4, hy + 2, HIGHLIGHT);
        put(img, cx + 2, hy + 2, HIGHLIGHT);
        // Eye glow aura
        put(img, cx - 7, hy + 3, RESONANCE);
        put(img, cx + 7, hy + 3, RESONANCE);
    }

    // Subtle blush
    fill_rect(img, cx - 8, hy + 5, 2, 2, BLUSH);
    fill_rect(img, cx + 7, hy + 5, 2, 2, BLUSH);

    // Broad triangular nose
    fill_rect(img, cx - 2, hy + 8, 5, 2, NOSE_COLOR);
    fill_rect(img, cx - 1, hy + 10, 3, 1, NOSE_COLOR);
    put(img, cx, hy + 11, NOSE_COLOR);

    // Mood mouth
    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        fill_rect(img, cx - 2, hy + 13, 5, 1, p.mouth);
        put(img, cx - 3, hy + 12, p.mouth);
        put(img, cx + 4, hy + 12, p.mouth);
    } else if *mood == MoodState::Hungry {
        fill_rect(img, cx - 1, hy + 13, 3, 2, p.mouth);
    } else if *mood != MoodState::Sleeping {
        fill_rect(img, cx - 1, hy + 13, 3, 1, p.mouth);
    }
}

// ===================================================================
// ELDER overlay — fading, hunched, wise
// ===================================================================

pub fn draw_elder_details(img: &mut RgbaImage, cx: i32) {
    let hy = 14; // same as adult head
    let body_y = 30;
    let white = Rgba([230, 225, 215, 255]);
    let thin = Rgba([200, 170, 165, 150]);
    let dim_res = Rgba([140, 180, 200, 50]);

    // Gray ear tips
    fill_circle(img, cx - 12, hy - 13, 3, white);
    fill_circle(img, cx + 12, hy - 13, 3, white);

    // Wisdom marks on forehead
    put(img, cx - 2, hy - 5, white);
    put(img, cx + 1, hy - 6, white);
    put(img, cx + 3, hy - 4, white);

    // White whisker dots
    put(img, cx - 8, hy + 4, white);
    put(img, cx - 9, hy + 6, white);
    put(img, cx + 8, hy + 4, white);
    put(img, cx + 9, hy + 6, white);

    // Thinning fur patches
    put(img, cx - 7, body_y - 5, thin);
    put(img, cx + 6, body_y - 3, thin);
    put(img, cx - 10, body_y + 3, thin);
    put(img, cx + 9, body_y + 5, thin);

    // Fading kokoro-sac
    fill_circle(img, cx, body_y + 2, 4, dim_res);
    put(img, cx - 13, body_y, dim_res);
    put(img, cx + 13, body_y, dim_res);
}
