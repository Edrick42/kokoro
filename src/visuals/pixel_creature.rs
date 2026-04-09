//! Runtime pixel art creature renderer.
//!
//! Draws creatures directly to a small pixel buffer (64x64) and displays
//! as a scaled-up sprite. No pre-made PNGs — the creature IS the pixels.
//!
//! ## Growth stages
//!
//! Each species has 5 visual stages, each with distinct proportions:
//!
//! | Stage | Proportions | Features |
//! |-------|-------------|----------|
//! | Egg   | Oval, no face | Species-colored with pattern |
//! | Cub   | 70:30 head:body, huge eyes | No appendages (no ears/wings/crests) |
//! | Young | 55:45, eyes shrink slightly | Appendages appear small |
//! | Adult | 45:55, balanced proportions | Full features, all appendages |
//! | Elder | 45:55, slightly faded colors | Wisdom marks, paler palette |
//!
//! Kindchenschema (baby schema) drives the Cub stage — big head, big eyes
//! low on face, round body. As the creature grows, proportions shift toward
//! a more mature body plan.

use bevy::prelude::*;
use bevy::image::ImageSampler;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{RgbaImage, Rgba};

type BevyImage = Image;

use crate::creature::species::CreatureRoot;
use crate::genome::{Genome, Species};
use crate::mind::{Mind, MoodState};
use crate::visuals::evolution::{GrowthState, GrowthStage};

const CANVAS_W: u32 = 64;
const CANVAS_H: u32 = 64;
const DISPLAY_SCALE: f32 = 5.0;

#[derive(Component)]
pub struct PixelCreature;

#[derive(Component)]
pub struct HasPixelCreature;

pub struct PixelCreaturePlugin;

impl Plugin for PixelCreaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (attach_pixel_creature, update_pixel_creature).chain());
    }
}

/// Attaches a pixel creature sprite to any CreatureRoot that doesn't have one yet.
fn attach_pixel_creature(
    mut commands: Commands,
    mut images: ResMut<Assets<BevyImage>>,
    query: Query<Entity, (With<CreatureRoot>, Without<HasPixelCreature>)>,
    genome: Res<Genome>,
    mind: Res<Mind>,
    growth: Res<GrowthState>,
) {
    for entity in query.iter() {
        let handle = create_pixel_creature_image(&mut images);

        if let Some(image) = images.get_mut(&handle) {
            let mut buf = RgbaImage::new(CANVAS_W, CANVAS_H);
            draw_creature(&mut buf, &genome.species, &mind.mood, &growth.stage);
            if let Some(ref mut data) = image.data {
                data.copy_from_slice(buf.as_raw());
            }
        }

        info!("Attaching pixel creature to entity {:?}", entity);
        commands.entity(entity).insert(HasPixelCreature);

        commands.entity(entity).with_child((
            PixelCreature,
            Sprite {
                image: handle,
                custom_size: Some(Vec2::new(
                    CANVAS_W as f32 * DISPLAY_SCALE,
                    CANVAS_H as f32 * DISPLAY_SCALE,
                )),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 5.0),
        ));
    }
}

/// Updates the pixel creature texture when mood, genome, or growth stage changes.
fn update_pixel_creature(
    genome: Res<Genome>,
    mind: Res<Mind>,
    growth: Res<GrowthState>,
    mut images: ResMut<Assets<BevyImage>>,
    creature_q: Query<&Sprite, With<PixelCreature>>,
    mut pixel_buf: Local<Option<RgbaImage>>,
) {
    if pixel_buf.is_none() {
        *pixel_buf = Some(RgbaImage::new(CANVAS_W, CANVAS_H));
    }
    let buf = pixel_buf.as_mut().unwrap();

    if !mind.is_changed() && !genome.is_changed() && !growth.is_changed() {
        return;
    }

    draw_creature(buf, &genome.species, &mind.mood, &growth.stage);

    for sprite in creature_q.iter() {
        if let Some(image) = images.get_mut(&sprite.image) {
            if let Some(ref mut data) = image.data {
                data.copy_from_slice(buf.as_raw());
            }
        }
    }
}

// ===================================================================
// COLOR PALETTES
// ===================================================================

struct Palette {
    body: Rgba<u8>,
    body_light: Rgba<u8>,
    eye: Rgba<u8>,
    mouth: Rgba<u8>,
    accent: Rgba<u8>,
    egg: Rgba<u8>,       // egg shell / membrane color
    egg_spot: Rgba<u8>,  // egg pattern spots
}

fn palette(species: &Species) -> Palette {
    match species {
        Species::Moluun => Palette {
            body:       Rgba([140, 215, 240, 255]),
            body_light: Rgba([180, 235, 250, 255]),
            eye:        Rgba([15, 15, 20, 255]),
            mouth:      Rgba([30, 30, 40, 255]),
            accent:     Rgba([120, 195, 225, 255]),
            egg:        Rgba([220, 200, 180, 255]),
            egg_spot:   Rgba([170, 220, 235, 255]),
        },
        Species::Pylum => Palette {
            body:       Rgba([245, 210, 130, 255]),
            body_light: Rgba([255, 235, 175, 255]),
            eye:        Rgba([15, 15, 20, 255]),
            mouth:      Rgba([240, 150, 55, 255]),
            accent:     Rgba([230, 190, 110, 255]),
            egg:        Rgba([235, 225, 200, 255]),
            egg_spot:   Rgba([180, 140, 80, 255]),
        },
        Species::Skael => Palette {
            body:       Rgba([110, 170, 120, 255]),
            body_light: Rgba([145, 200, 150, 255]),
            eye:        Rgba([15, 15, 20, 255]),
            mouth:      Rgba([75, 95, 75, 255]),
            accent:     Rgba([150, 95, 75, 255]),
            egg:        Rgba([160, 180, 160, 255]),
            egg_spot:   Rgba([100, 140, 110, 255]),
        },
        Species::Nyxal => Palette {
            body:       Rgba([90, 55, 125, 255]),
            body_light: Rgba([115, 80, 150, 255]),
            eye:        Rgba([15, 15, 20, 255]),
            mouth:      Rgba([15, 15, 30, 255]),
            accent:     Rgba([95, 60, 135, 255]),
            egg:        Rgba([70, 50, 100, 180]),
            egg_spot:   Rgba([50, 170, 190, 200]),
        },
    }
}

/// Returns a faded (elder) version of a palette — 30% desaturated toward gray.
fn elder_palette(species: &Species) -> Palette {
    let base = palette(species);
    Palette {
        body:       fade(base.body, 0.3),
        body_light: fade(base.body_light, 0.3),
        eye:        base.eye,   // eyes always black, all species
        mouth:      fade(base.mouth, 0.2),
        accent:     fade(base.accent, 0.3),
        egg:        base.egg,
        egg_spot:   base.egg_spot,
    }
}

/// Fades a color toward gray by `amount` (0.0 = unchanged, 1.0 = full gray).
fn fade(c: Rgba<u8>, amount: f32) -> Rgba<u8> {
    let gray = (c[0] as f32 * 0.3 + c[1] as f32 * 0.59 + c[2] as f32 * 0.11) as u8;
    let a = amount;
    Rgba([
        (c[0] as f32 * (1.0 - a) + gray as f32 * a) as u8,
        (c[1] as f32 * (1.0 - a) + gray as f32 * a) as u8,
        (c[2] as f32 * (1.0 - a) + gray as f32 * a) as u8,
        c[3],
    ])
}

// ===================================================================
// MAIN DRAW DISPATCH
// ===================================================================

fn draw_creature(img: &mut RgbaImage, species: &Species, mood: &MoodState, stage: &GrowthStage) {
    let cx = img.width() as i32 / 2;

    // Clear canvas
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }

    match stage {
        GrowthStage::Cub   => draw_cub(img, species, mood, cx),
        GrowthStage::Young => draw_young(img, species, mood, cx),
        GrowthStage::Adult => draw_adult(img, species, mood, cx),
        GrowthStage::Elder => draw_elder(img, species, mood, cx),
    }
}

/// Draws an egg (called from outside the normal creature dispatch).
pub fn draw_egg(img: &mut RgbaImage, species: &Species) {
    let p = palette(species);
    let cx = img.width() as i32 / 2;

    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }

    match species {
        Species::Moluun => draw_egg_moluun(img, &p, cx),
        Species::Pylum  => draw_egg_pylum(img, &p, cx),
        Species::Skael  => draw_egg_skael(img, &p, cx),
        Species::Nyxal  => draw_egg_nyxal(img, &p, cx),
    }
}

// ===================================================================
// PRIMITIVES
// ===================================================================

fn put(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && x < img.width() as i32 && y >= 0 && y < img.height() as i32 {
        img.put_pixel(x as u32, y as u32, color);
    }
}

fn fill_circle(img: &mut RgbaImage, cx: i32, cy: i32, r: i32, color: Rgba<u8>) {
    let r_sq = r * r;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy < r_sq {
                put(img, cx + dx, cy + dy, color);
            }
        }
    }
}

fn fill_rect(img: &mut RgbaImage, x: i32, y: i32, w: i32, h: i32, color: Rgba<u8>) {
    for dy in 0..h {
        for dx in 0..w {
            put(img, x + dx, y + dy, color);
        }
    }
}

/// Filled ellipse for eggs (wider than tall, or taller than wide).
fn fill_ellipse(img: &mut RgbaImage, cx: i32, cy: i32, rx: i32, ry: i32, color: Rgba<u8>) {
    for dy in -ry..=ry {
        for dx in -rx..=rx {
            let nx = dx as f32 / rx as f32;
            let ny = dy as f32 / ry as f32;
            if nx * nx + ny * ny < 1.0 {
                put(img, cx + dx, cy + dy, color);
            }
        }
    }
}

/// Simple eyes helper — draws a pair of rectangular eyes.
fn draw_eyes(img: &mut RgbaImage, cx: i32, ey: i32, size: i32, gap: i32, mood: &MoodState, color: Rgba<u8>) {
    if *mood == MoodState::Sleeping {
        // Closed eyes — horizontal slits
        fill_rect(img, cx - gap - size, ey + size / 3, size, 2, color);
        fill_rect(img, cx + gap, ey + size / 3, size, 2, color);
    } else {
        fill_rect(img, cx - gap - size, ey, size, size, color);
        fill_rect(img, cx + gap, ey, size, size, color);
    }
}

// ===================================================================
// EGG — species-specific oval with pattern
// ===================================================================

fn draw_egg_moluun(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    // Warm cream egg, slightly taller than wide
    fill_ellipse(img, cx, cy, 12, 16, p.egg);
    // Blue spots (hint of species color)
    fill_circle(img, cx - 4, cy - 5, 3, p.egg_spot);
    fill_circle(img, cx + 5, cy + 3, 2, p.egg_spot);
    fill_circle(img, cx - 2, cy + 7, 2, p.egg_spot);
}

fn draw_egg_pylum(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    // Speckled bird egg
    fill_ellipse(img, cx, cy, 10, 14, p.egg);
    // Brown speckles scattered
    for &(dx, dy) in &[(-3,-6), (4,-3), (-5,2), (2,5), (5,-7), (-1,8), (3,1), (-4,-2)] {
        put(img, cx + dx, cy + dy, p.egg_spot);
        put(img, cx + dx + 1, cy + dy, p.egg_spot);
    }
}

fn draw_egg_skael(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    // Crystalline green egg
    fill_ellipse(img, cx, cy, 11, 15, p.egg);
    // Crystal vein lines (diagonal marks)
    for i in 0..5 {
        let y = cy - 8 + i * 4;
        put(img, cx - 3 + i, y, p.egg_spot);
        put(img, cx - 2 + i, y, p.egg_spot);
        put(img, cx + 2 - i, y + 1, p.egg_spot);
    }
}

fn draw_egg_nyxal(img: &mut RgbaImage, p: &Palette, cx: i32) {
    let cy = 30;
    // Translucent gelatinous sphere (rounder than other eggs)
    fill_circle(img, cx, cy, 13, p.egg);
    // Bioluminescent spots (glow)
    fill_circle(img, cx - 3, cy - 3, 2, p.egg_spot);
    fill_circle(img, cx + 4, cy + 2, 2, p.egg_spot);
    fill_circle(img, cx, cy + 5, 1, p.egg_spot);
    // Inner lighter area (translucent effect)
    fill_circle(img, cx, cy, 7, p.body_light);
}

// ===================================================================
// CUB — Kindchenschema: 70:30 head:body, huge eyes, no appendages
// ===================================================================
// This is the ORIGINAL design — maximum cuteness, pure round blob + eyes.

fn draw_cub(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32) {
    let p = palette(species);
    let by = 24; // body center Y
    let br = 18; // body radius — body IS the head

    // Body (no appendages for cubs — just a round blob)
    fill_circle(img, cx, by, br, p.body);

    // Belly
    fill_circle(img, cx, by + 5, 12, p.body_light);

    // Tiny stub feet (minimal, just resting bumps)
    fill_rect(img, cx - 8, by + br - 3, 6, 5, p.body);
    fill_rect(img, cx + 3, by + br - 3, 6, 5, p.body);

    // HUGE eyes (6x6) placed low — peak Kindchenschema
    draw_eyes(img, cx, by + 1, 6, 4, mood, p.eye);

    // Tiny mouth
    if *mood != MoodState::Sleeping {
        fill_rect(img, cx - 1, by + 10, 3, 2, p.mouth);
    }
}

// ===================================================================
// YOUNG — emerging features: small appendages appear, body elongates
// ===================================================================
// Head:body ~55:45. Appendages appear as small stubs.
// Eyes still large (5x5) but not as dominant.

fn draw_young(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32) {
    let p = palette(species);

    match species {
        Species::Moluun => draw_young_moluun(img, &p, cx, mood),
        Species::Pylum  => draw_young_pylum(img, &p, cx, mood),
        Species::Skael  => draw_young_skael(img, &p, cx, mood),
        Species::Nyxal  => draw_young_nyxal(img, &p, cx, mood),
    }
}

fn draw_young_moluun(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 17;

    // Small ear stubs (just emerging)
    fill_circle(img, cx - 9, by - 12, 4, p.accent);
    fill_circle(img, cx + 9, by - 12, 4, p.accent);

    // Body
    fill_circle(img, cx, by, br, p.body);
    fill_circle(img, cx, by + 5, 11, p.body_light);

    // Feet (slightly bigger than cub)
    fill_rect(img, cx - 9, by + br - 4, 7, 6, p.body);
    fill_rect(img, cx + 3, by + br - 4, 7, 6, p.body);

    // Eyes — 5x5 (slightly smaller than cub)
    draw_eyes(img, cx, by + 2, 5, 4, mood, p.eye);

    // Mouth
    if *mood != MoodState::Sleeping {
        fill_rect(img, cx - 2, by + 10, 4, 2, p.mouth);
    }
}

fn draw_young_pylum(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 15;

    // Wing stubs (small rectangles)
    fill_rect(img, cx - br, by - 1, 4, 8, p.accent);
    fill_rect(img, cx + br - 3, by - 1, 4, 8, p.accent);

    // Body
    fill_circle(img, cx, by, br, p.body);
    fill_circle(img, cx, by + 4, 9, p.body_light);

    // Tiny tuft
    fill_rect(img, cx - 1, by - br - 1, 3, 3, p.body);

    // Feet
    fill_rect(img, cx - 6, by + br - 3, 5, 5, p.mouth);
    fill_rect(img, cx + 2, by + br - 3, 5, 5, p.mouth);

    // Eyes
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);

    // Small beak
    fill_rect(img, cx - 2, by + 8, 5, 3, p.mouth);
}

fn draw_young_skael(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 15;

    // Small crest stubs
    fill_rect(img, cx - 8, by - br + 2, 4, 5, p.accent);
    fill_rect(img, cx + 5, by - br + 2, 4, 5, p.accent);

    // Body
    fill_circle(img, cx, by, br, p.body);
    fill_circle(img, cx, by + 4, 9, p.body_light);

    // Short tail stub
    fill_rect(img, cx - 2, by + br - 2, 5, 6, p.body);

    // Feet
    fill_rect(img, cx - 8, by + br - 3, 6, 6, p.body);
    fill_rect(img, cx + 3, by + br - 3, 6, 6, p.body);

    // Eyes
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);

    // Small snout
    fill_rect(img, cx - 2, by + 8, 5, 3, p.mouth);
}

fn draw_young_nyxal(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 20;
    let br = 13;

    // Small mantle (just emerging)
    fill_circle(img, cx, by - 4, 7, p.accent);

    // 2 short tentacle stubs (front pair only)
    fill_rect(img, cx - 6, by + br - 2, 4, 10, p.accent);
    fill_rect(img, cx + 3, by + br - 2, 4, 10, p.accent);

    // Body
    fill_circle(img, cx, by, br, p.body);
    fill_circle(img, cx, by + 3, 8, p.body_light);

    // Eyes
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
}

// ===================================================================
// ADULT — full features, balanced proportions (45:55 head:body)
// ===================================================================
// This is the mature form — all appendages present and full-sized.
// Body is slightly larger relative to head than Cub.

fn draw_adult(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32) {
    let p = palette(species);

    match species {
        Species::Moluun => draw_adult_moluun(img, &p, cx, mood),
        Species::Pylum  => draw_adult_pylum(img, &p, cx, mood),
        Species::Skael  => draw_adult_skael(img, &p, cx, mood),
        Species::Nyxal  => draw_adult_nyxal(img, &p, cx, mood),
    }
}

fn draw_adult_moluun(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 18;

    // Full-sized ears
    fill_circle(img, cx - 11, by - 14, 6, p.accent);
    fill_circle(img, cx + 11, by - 14, 6, p.accent);

    // Body
    fill_circle(img, cx, by, br, p.body);
    fill_circle(img, cx, by + 5, 12, p.body_light);

    // Sturdy feet
    fill_rect(img, cx - 10, by + br - 4, 8, 7, p.body);
    fill_rect(img, cx + 3, by + br - 4, 8, 7, p.body);

    // Eyes — 5x5 (mature, not as huge as cub)
    draw_eyes(img, cx, by + 2, 5, 4, mood, p.eye);

    // Mouth
    if *mood != MoodState::Sleeping {
        fill_rect(img, cx - 2, by + 10, 4, 2, p.mouth);
    }
}

fn draw_adult_pylum(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 16;

    // Full wings
    fill_rect(img, cx - br + 1, by - 3, 6, 12, p.accent);
    fill_rect(img, cx + br - 6, by - 3, 6, 12, p.accent);

    // Body
    fill_circle(img, cx, by, br, p.body);
    fill_circle(img, cx, by + 4, 10, p.body_light);

    // Tuft
    fill_rect(img, cx - 2, by - br - 2, 4, 5, p.body);

    // Feet
    fill_rect(img, cx - 7, by + br - 3, 6, 6, p.mouth);
    fill_rect(img, cx + 2, by + br - 3, 6, 6, p.mouth);

    // Eyes
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);

    // Beak
    fill_rect(img, cx - 3, by + 8, 6, 4, p.mouth);
}

fn draw_adult_skael(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 16;

    // Full crests
    fill_rect(img, cx - 10, by - br + 1, 5, 8, p.accent);
    fill_rect(img, cx + 6, by - br + 1, 5, 8, p.accent);

    // Body
    fill_circle(img, cx, by, br, p.body);
    fill_circle(img, cx, by + 4, 10, p.body_light);

    // Full tail
    fill_rect(img, cx - 3, by + br - 3, 6, 10, p.body);

    // Feet
    fill_rect(img, cx - 10, by + br - 4, 7, 7, p.body);
    fill_rect(img, cx + 4, by + br - 4, 7, 7, p.body);

    // Eyes
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);

    // Snout
    fill_rect(img, cx - 3, by + 9, 6, 3, p.mouth);
}

fn draw_adult_nyxal(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 18;
    let br = 14;

    // Full mantle
    fill_circle(img, cx, by - 5, 10, p.accent);

    // 4 full tentacles
    fill_rect(img, cx - 10, by + br - 4, 4, 18, p.accent);
    fill_rect(img, cx - 4, by + br - 3, 4, 20, p.accent);
    fill_rect(img, cx + 1, by + br - 3, 4, 20, p.accent);
    fill_rect(img, cx + 7, by + br - 4, 4, 18, p.accent);

    // Glow tips
    let glow = Rgba([50, 180, 200, 255]);
    fill_rect(img, cx - 10, by + br + 12, 4, 3, glow);
    fill_rect(img, cx - 4, by + br + 15, 4, 3, glow);
    fill_rect(img, cx + 1, by + br + 15, 4, 3, glow);
    fill_rect(img, cx + 7, by + br + 12, 4, 3, glow);

    // Body
    fill_circle(img, cx, by, br, p.body);
    fill_circle(img, cx, by + 3, 9, p.body_light);

    // Eyes
    draw_eyes(img, cx, by + 1, 5, 4, mood, p.eye);
}

// ===================================================================
// ELDER — faded colors, wisdom marks, slightly hunched
// ===================================================================
// Same structure as Adult but with desaturated palette and visual details.

fn draw_elder(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32) {
    let p = elder_palette(species);
    let base_p = palette(species);

    match species {
        Species::Moluun => {
            draw_adult_moluun(img, &p, cx, mood);
            // Wisdom marks — small light spots on forehead
            put(img, cx - 3, 14, base_p.body_light);
            put(img, cx + 2, 13, base_p.body_light);
            put(img, cx, 12, base_p.body_light);
        }
        Species::Pylum => {
            draw_adult_pylum(img, &p, cx, mood);
            // Faded feather tips — lighter accents at wing edges
            put(img, cx - 15, 23, base_p.body_light);
            put(img, cx - 15, 25, base_p.body_light);
            put(img, cx + 16, 23, base_p.body_light);
            put(img, cx + 16, 25, base_p.body_light);
        }
        Species::Skael => {
            draw_adult_skael(img, &p, cx, mood);
            // Worn scale marks — lighter patches on body
            put(img, cx - 6, 20, base_p.body_light);
            put(img, cx + 5, 22, base_p.body_light);
            put(img, cx - 4, 28, base_p.body_light);
            put(img, cx + 3, 30, base_p.body_light);
        }
        Species::Nyxal => {
            draw_adult_nyxal(img, &p, cx, mood);
            // Dimmer glow tips — overwrite with softer color
            let dim_glow = Rgba([35, 130, 150, 255]);
            fill_rect(img, cx - 10, 24, 4, 3, dim_glow);
            fill_rect(img, cx - 4, 27, 4, 3, dim_glow);
            fill_rect(img, cx + 1, 27, 4, 3, dim_glow);
            fill_rect(img, cx + 7, 24, 4, 3, dim_glow);
        }
    }
}

// ===================================================================
// IMAGE CREATION
// ===================================================================

pub fn create_pixel_creature_image(images: &mut Assets<BevyImage>) -> Handle<BevyImage> {
    let mut image = BevyImage::new_fill(
        Extent3d {
            width: CANVAS_W,
            height: CANVAS_H,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );

    image.sampler = ImageSampler::nearest();
    images.add(image)
}
