//! Moluun pixel art — warm forest mammal with round ears, soft fur, pink cheeks.

use image::{RgbaImage, Rgba};
use crate::mind::MoodState;
use super::{Palette, fill_circle, fill_rect, fill_ellipse, put, draw_eyes, fade};

// --- Egg ---

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
    put(img, cx + 3, cy - 12, fade(p.egg, 0.3));
}

// --- Cub ---

pub fn draw_cub(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 18;
    let blush = Rgba([220, 150, 140, 255]);
    let ear_inner = Rgba([200, 160, 155, 255]);
    let highlight = Rgba([255, 255, 255, 200]);
    let nose = Rgba([50, 35, 30, 255]);

    // Round ears with pink inner
    fill_circle(img, cx - 11, by - 14, 6, p.accent);
    fill_circle(img, cx + 11, by - 14, 6, p.accent);
    fill_circle(img, cx - 11, by - 14, 3, ear_inner);
    fill_circle(img, cx + 11, by - 14, 3, ear_inner);

    // Body
    fill_circle(img, cx, by, br, p.body);

    // Fur texture
    for &(dx, dy) in &[(-8,-6), (7,-4), (-5,3), (9,1), (-3,-10), (6,7), (-9,5)] {
        put(img, cx + dx, by + dy, p.accent);
    }

    // Belly
    fill_circle(img, cx, by + 6, 13, p.body_light);

    // Round stub feet with pads
    fill_circle(img, cx - 7, by + br - 2, 5, p.body);
    fill_circle(img, cx + 7, by + br - 2, 5, p.body);
    for dx in [-2, 0, 2] {
        put(img, cx - 7 + dx, by + br + 2, p.accent);
        put(img, cx + 7 + dx, by + br + 2, p.accent);
    }

    // HUGE eyes (6x6)
    draw_eyes(img, cx, by, 6, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 6, by, highlight);
        put(img, cx - 5, by, highlight);
        put(img, cx + 2, by, highlight);
        put(img, cx + 3, by, highlight);
    }

    // Blush marks
    fill_rect(img, cx - 11, by + 5, 3, 2, blush);
    fill_rect(img, cx + 9, by + 5, 3, 2, blush);

    // Nose — inverted triangle
    fill_rect(img, cx - 1, by + 7, 3, 2, nose);
    put(img, cx, by + 9, nose);

    // Mouth
    if *mood != MoodState::Sleeping {
        put(img, cx - 1, by + 11, p.mouth);
        put(img, cx + 1, by + 11, p.mouth);
    }
    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        put(img, cx - 2, by + 11, p.mouth);
        put(img, cx + 2, by + 11, p.mouth);
        put(img, cx - 1, by + 12, p.mouth);
        put(img, cx + 1, by + 12, p.mouth);
    }
}

// --- Young ---

pub fn draw_young(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 17;
    let blush = Rgba([220, 150, 140, 255]);
    let ear_inner = Rgba([200, 160, 155, 255]);
    let highlight = Rgba([255, 255, 255, 200]);
    let nose = Rgba([50, 35, 30, 255]);

    // Growing ears
    fill_circle(img, cx - 11, by - 13, 6, p.accent);
    fill_circle(img, cx + 11, by - 13, 6, p.accent);
    fill_circle(img, cx - 11, by - 13, 3, ear_inner);
    fill_circle(img, cx + 11, by - 13, 3, ear_inner);

    // Body
    fill_circle(img, cx, by, br, p.body);
    for &(dx, dy) in &[(-7,-5), (6,-3), (-4,4), (8,2), (-2,-9), (5,6)] {
        put(img, cx + dx, by + dy, p.accent);
    }
    fill_circle(img, cx, by + 5, 12, p.body_light);

    // Feet with pads
    fill_circle(img, cx - 7, by + br - 1, 5, p.body);
    fill_circle(img, cx + 7, by + br - 1, 5, p.body);
    for dx in [-2, 0, 2] {
        put(img, cx - 7 + dx, by + br + 3, p.accent);
        put(img, cx + 7 + dx, by + br + 3, p.accent);
    }

    // Short arms emerging
    fill_rect(img, cx - br + 1, by + 1, 3, 5, p.body);
    fill_rect(img, cx + br - 3, by + 1, 3, 5, p.body);

    // Eyes — 5x5
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 5, by + 1, highlight);
        put(img, cx + 2, by + 1, highlight);
    }

    // Blush
    fill_rect(img, cx - 10, by + 5, 2, 2, blush);
    fill_rect(img, cx + 9, by + 5, 2, 2, blush);

    // Nose
    fill_rect(img, cx - 1, by + 7, 3, 2, nose);
    put(img, cx, by + 9, nose);

    // Mouth
    if *mood != MoodState::Sleeping {
        put(img, cx - 1, by + 11, p.mouth);
        put(img, cx + 1, by + 11, p.mouth);
    }
    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        put(img, cx - 2, by + 11, p.mouth);
        put(img, cx + 2, by + 11, p.mouth);
        put(img, cx - 1, by + 12, p.mouth);
        put(img, cx + 1, by + 12, p.mouth);
    }
}

// --- Adult ---

pub fn draw_adult(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 18;
    let blush = Rgba([210, 145, 135, 255]);
    let ear_inner = Rgba([200, 160, 155, 255]);
    let highlight = Rgba([255, 255, 255, 200]);
    let nose = Rgba([50, 35, 30, 255]);

    // Full-sized ears
    fill_circle(img, cx - 12, by - 14, 7, p.accent);
    fill_circle(img, cx + 12, by - 14, 7, p.accent);
    fill_circle(img, cx - 12, by - 14, 4, ear_inner);
    fill_circle(img, cx + 12, by - 14, 4, ear_inner);

    // Body
    fill_circle(img, cx, by, br, p.body);
    for &(dx, dy) in &[(-8,-7), (7,-5), (-5,3), (9,1), (-3,-11), (6,7), (-10,5), (4,-8), (-6,9)] {
        put(img, cx + dx, by + dy, p.accent);
    }

    // Belly
    fill_circle(img, cx, by + 5, 13, p.body_light);
    for &(dx, dy) in &[(-3, 2), (4, 3), (-1, 6), (2, 8)] {
        put(img, cx + dx, by + 5 + dy, fade(p.body_light, 0.1));
    }

    // Arms
    fill_rect(img, cx - br + 1, by - 1, 4, 8, p.body);
    fill_rect(img, cx + br - 4, by - 1, 4, 8, p.body);
    fill_circle(img, cx - br + 2, by + 7, 2, p.accent);
    fill_circle(img, cx + br - 2, by + 7, 2, p.accent);

    // Feet
    fill_circle(img, cx - 8, by + br - 1, 6, p.body);
    fill_circle(img, cx + 8, by + br - 1, 6, p.body);
    for dx in [-3, -1, 1, 3] {
        put(img, cx - 8 + dx, by + br + 4, p.accent);
        put(img, cx + 8 + dx, by + br + 4, p.accent);
    }

    // Eyes — 5x5
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
    if *mood != MoodState::Sleeping {
        put(img, cx - 5, by + 1, highlight);
        put(img, cx + 2, by + 1, highlight);
    }

    // Blush
    fill_rect(img, cx - 10, by + 5, 2, 2, blush);
    fill_rect(img, cx + 9, by + 5, 2, 2, blush);

    // Nose — inverted triangle (wider)
    fill_rect(img, cx - 2, by + 7, 5, 2, nose);
    fill_rect(img, cx - 1, by + 9, 3, 1, nose);
    put(img, cx, by + 10, nose);

    // Mouth — mood-expressive
    if *mood == MoodState::Happy || *mood == MoodState::Playful {
        fill_rect(img, cx - 2, by + 12, 5, 1, p.mouth);
        put(img, cx - 3, by + 11, p.mouth);
        put(img, cx + 4, by + 11, p.mouth);
    } else if *mood == MoodState::Hungry {
        fill_rect(img, cx - 1, by + 12, 3, 2, p.mouth);
    } else if *mood != MoodState::Sleeping {
        fill_rect(img, cx - 1, by + 12, 3, 1, p.mouth);
    }
}

// --- Elder overlay (called after draw_adult with faded palette) ---

pub fn draw_elder_details(img: &mut RgbaImage, cx: i32) {
    let by = 24;
    let white = Rgba([230, 225, 215, 255]);
    // Gray/white ear tips
    fill_circle(img, cx - 12, by - 18, 3, white);
    fill_circle(img, cx + 12, by - 18, 3, white);
    // Wisdom marks on forehead
    put(img, cx - 2, 13, white);
    put(img, cx + 1, 12, white);
    put(img, cx + 3, 14, white);
    // White whisker dots
    put(img, cx - 10, by + 3, white);
    put(img, cx - 11, by + 5, white);
    put(img, cx + 10, by + 3, white);
    put(img, cx + 11, by + 5, white);
}
