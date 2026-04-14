//! Nyxal skin — deep-sea cephalopod (cuttlefish / nautilus).
//!
//! Structural evolution:
//! - Cub:   almost ALL mantle dome, tiny tentacle stubs, translucent, minimal glow
//! - Young: tentacles growing FAST (allometric), dome shrinking proportionally, glow appearing
//! - Adult: tentacles dominate, dome proportionally small, rich bioluminescence, side fins
//! - Elder: dimmer glow, wisdom rings on mantle, thinner tentacles

use image::{RgbaImage, Rgba};
use bevy::prelude::Res;
use crate::creature::interaction::soft_body::SoftBody;
use crate::mind::MoodState;
use super::{Palette, fill_circle, fill_rect, put, draw_eyes, fade};

const GLOW_BRIGHT: Rgba<u8> = Rgba([80, 220, 240, 200]);
const GLOW_DIM: Rgba<u8> = Rgba([50, 140, 160, 150]);
const GLOW_FAINT: Rgba<u8> = Rgba([40, 100, 120, 80]);
const SPOT: Rgba<u8> = Rgba([100, 60, 140, 255]);

// ===================================================================
// EGG
// ===================================================================

pub fn draw_egg(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    fill_circle(img, cx, cy, 14, p.egg);
    fill_circle(img, cx, cy, 10, fade(p.egg, 0.15));
    fill_circle(img, cx, cy, 6, p.body_light);
    fill_circle(img, cx - 4, cy - 4, 2, p.egg_spot);
    fill_circle(img, cx + 5, cy + 2, 2, p.egg_spot);
    fill_circle(img, cx - 1, cy + 6, 2, p.egg_spot);
    fill_circle(img, cx, cy, 3, p.body);
    put(img, cx, cy - 1, p.eye);
}

// ===================================================================
// CUB — almost all dome, tiny stubs, translucent
// ===================================================================
// Planktonic larva: huge mantle dome with tiny tentacle nubs underneath.
// Nearly transparent. Almost no glow. Alien blob.

pub fn draw_cub(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, _sb: &Option<Res<SoftBody>>) {
    // HUGE mantle dome (this IS the creature)
    let my = 18;   // mantle center
    let mr = 16;   // mantle radius — dominates everything

    // Tiny tentacle stubs underneath (barely there)
    let tent_y = my + mr - 2;
    fill_rect(img, cx - 10, tent_y, 3, 6, p.accent);
    fill_rect(img, cx - 4,  tent_y, 3, 8, p.accent);
    fill_rect(img, cx + 2,  tent_y, 3, 8, p.accent);
    fill_rect(img, cx + 8,  tent_y, 3, 6, p.accent);

    // Big dome
    fill_circle(img, cx, my, mr, p.body);

    // Inner lighter area (translucent — can see through)
    fill_circle(img, cx, my + 2, 10, p.body_light);

    // Body spots (sparse — barely visible at this age)
    put(img, cx - 4, my - 3, SPOT);
    put(img, cx + 3, my - 1, SPOT);

    // Faint kokoro-sac glow
    fill_circle(img, cx, my + 1, 3, GLOW_FAINT);

    // Big cyan eyes
    draw_eyes(img, cx, my + 2, 6, 4, mood, p.eye);
}

// ===================================================================
// YOUNG — tentacles explode in growth, dome shrinks proportionally
// ===================================================================
// Allometric growth: tentacles grow MUCH faster than the dome.
// The creature is transitioning from blob to cephalopod.

pub fn draw_young(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, _sb: &Option<Res<SoftBody>>) {
    let my = 16;    // mantle center (smaller)
    let mr = 12;    // mantle radius (shrank!)

    // Growing mantle cap
    fill_circle(img, cx, my - 3, 8, p.accent);
    // Glow rim on mantle
    for dx in -5..=5 {
        put(img, cx + dx, my - 10, GLOW_FAINT);
    }

    // LONG tentacles (the big change!)
    let tent_y = my + mr - 3;
    let tent_len = 22;  // much longer than cub's 6-8
    fill_rect(img, cx - 12, tent_y, 3, tent_len - 4, p.accent);
    fill_rect(img, cx - 5,  tent_y, 3, tent_len, p.accent);
    fill_rect(img, cx + 3,  tent_y, 3, tent_len, p.accent);
    fill_rect(img, cx + 10, tent_y, 3, tent_len - 4, p.accent);
    // Glow tips appearing!
    fill_rect(img, cx - 12, tent_y + tent_len - 5, 3, 2, GLOW_DIM);
    fill_rect(img, cx - 5,  tent_y + tent_len - 2, 3, 2, GLOW_DIM);
    fill_rect(img, cx + 3,  tent_y + tent_len - 2, 3, 2, GLOW_DIM);
    fill_rect(img, cx + 10, tent_y + tent_len - 5, 3, 2, GLOW_DIM);

    // Body (dome)
    fill_circle(img, cx, my, mr, p.body);
    for &(dx, dy) in &[(-4,-3), (3,-1), (-1,2), (5,0)] {
        put(img, cx + dx, my + dy, SPOT);
    }
    fill_circle(img, cx, my + 3, 7, p.body_light);

    // Kokoro-sac (brighter)
    fill_circle(img, cx, my + 2, 4, GLOW_DIM);

    // Eyes
    draw_eyes(img, cx, my + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 8, my + 3, GLOW_FAINT);
        put(img, cx + 7, my + 3, GLOW_FAINT);
    }
}

// ===================================================================
// ADULT — tentacle-dominant, rich bioluminescence, side fins
// ===================================================================
// Complete transformation: tentacles are now the defining feature.
// Dome is proportionally small. Rich chromatophore patterns. Side fins.

pub fn draw_adult(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState, _sb: &Option<Res<SoftBody>>) {
    let my = 14;    // mantle center (small, high up)
    let mr = 11;    // mantle radius (proportionally small now!)

    // Full mantle cap with glow rim
    fill_circle(img, cx, my - 4, 10, p.accent);
    for dx in -8..=8 {
        put(img, cx + dx, my - 13, GLOW_BRIGHT);
    }
    // Mantle chromatophore spots
    put(img, cx - 4, my - 8, GLOW_DIM);
    put(img, cx + 3, my - 6, GLOW_DIM);
    put(img, cx - 1, my - 10, GLOW_DIM);

    // LONG flowing tentacles (dominant feature!)
    let tent_y = my + mr - 3;
    let tent_len = 30;  // very long
    fill_rect(img, cx - 14, tent_y, 4, tent_len - 6, p.accent);
    fill_rect(img, cx - 5,  tent_y, 4, tent_len, p.accent);
    fill_rect(img, cx + 2,  tent_y, 4, tent_len, p.accent);
    fill_rect(img, cx + 11, tent_y, 4, tent_len - 6, p.accent);
    // Curled outer tips
    put(img, cx - 15, tent_y + tent_len - 7, p.accent);
    put(img, cx + 14, tent_y + tent_len - 7, p.accent);
    // Bright glow tips!
    fill_rect(img, cx - 14, tent_y + tent_len - 8, 4, 3, GLOW_BRIGHT);
    fill_rect(img, cx - 5,  tent_y + tent_len - 2, 4, 3, GLOW_BRIGHT);
    fill_rect(img, cx + 2,  tent_y + tent_len - 2, 4, 3, GLOW_BRIGHT);
    fill_rect(img, cx + 11, tent_y + tent_len - 8, 4, 3, GLOW_BRIGHT);
    // Sucker dots along inner tentacles
    for i in 0..5 {
        put(img, cx - 4, tent_y + 3 + i * 4, SPOT);
        put(img, cx + 4, tent_y + 3 + i * 4, SPOT);
    }

    // Side fins (new adult feature!)
    for i in 0..4 {
        put(img, cx - mr - 1, my - 2 + i * 2, fade(p.accent, 0.3));
        put(img, cx + mr + 1, my - 2 + i * 2, fade(p.accent, 0.3));
    }

    // Body (dome — proportionally small now)
    fill_circle(img, cx, my, mr, p.body);
    for &(dx, dy) in &[(-5,-4), (4,-2), (-2,3), (6,1), (-7,0), (3,-6), (-4,5), (7,-3)] {
        put(img, cx + dx, my + dy, SPOT);
    }
    fill_circle(img, cx, my + 3, 7, p.body_light);

    // Kokoro-sac (bright with double layer)
    fill_circle(img, cx, my + 2, 5, GLOW_BRIGHT);
    fill_circle(img, cx, my + 2, 3, GLOW_DIM);

    // Eyes — luminous cyan with glow aura
    draw_eyes(img, cx, my + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 9, my + 2, GLOW_DIM);
        put(img, cx - 9, my + 3, GLOW_DIM);
        put(img, cx + 8, my + 2, GLOW_DIM);
        put(img, cx + 8, my + 3, GLOW_DIM);
    }

    // Happy: pulsing glow
    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        put(img, cx - 3, my - 2, GLOW_BRIGHT);
        put(img, cx + 2, my + 1, GLOW_BRIGHT);
        put(img, cx, my + 5, GLOW_BRIGHT);
    }
}

// ===================================================================
// ELDER overlay
// ===================================================================

pub fn draw_elder_details(img: &mut RgbaImage, cx: i32) {
    let my = 14;
    let dim = Rgba([140, 130, 115, 200]);
    let ring = Rgba([120, 100, 150, 255]);

    // Dimmer glow tips (overwrite bright with faded)
    let tent_y = my + 8;
    fill_rect(img, cx - 14, tent_y + 16, 4, 3, dim);
    fill_rect(img, cx - 5,  tent_y + 26, 4, 3, dim);
    fill_rect(img, cx + 2,  tent_y + 26, 4, 3, dim);
    fill_rect(img, cx + 11, tent_y + 16, 4, 3, dim);
    // Faded mantle glow
    for dx in -6..=6 {
        put(img, cx + dx, my - 13, dim);
    }
    // Wisdom rings on mantle (concentric age marks)
    put(img, cx - 2, my - 7, ring);
    put(img, cx + 1, my - 5, ring);
    put(img, cx, my - 9, ring);
    put(img, cx - 3, my - 3, ring);
    put(img, cx + 3, my - 4, ring);
}
