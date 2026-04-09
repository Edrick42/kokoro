//! Runtime pixel art creature renderer.
//!
//! Draws creatures directly to a small pixel buffer (64x64) every frame,
//! then displays as a scaled-up sprite. No pre-made PNGs — the creature
//! IS the pixels, generated from its anatomy in real time.
//!
//! Design principles (Kindchenschema — what humans find cute):
//! - Large head (40-50% of total height)
//! - Large eyes placed LOW on the face
//! - Round, compact body
//! - Short, thick limbs
//! - Big forehead (space above eyes)
//! - Small mouth placed low

use bevy::prelude::*;
use bevy::image::{ImageSampler};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{RgbaImage, Rgba};

// Bevy's Image type (re-exported from prelude as Image)
type BevyImage = Image;

use crate::creature::species::CreatureRoot;
use crate::genome::{Genome, Species};
use crate::mind::{Mind, MoodState};

/// Canvas size for the pixel creature (small = chunky pixels when scaled up).
const CANVAS_W: u32 = 64;
const CANVAS_H: u32 = 64;

/// Display scale (how big each pixel appears on screen).
const DISPLAY_SCALE: f32 = 5.0;

/// Marker component for the pixel creature sprite (on the child entity).
#[derive(Component)]
pub struct PixelCreature;

/// Marker on the root entity to indicate it already has a pixel creature attached.
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
) {
    for entity in query.iter() {
        // Create the pixel buffer image
        let handle = create_pixel_creature_image(&mut images);

        // Draw initial frame
        if let Some(image) = images.get_mut(&handle) {
            let mut buf = RgbaImage::new(CANVAS_W, CANVAS_H);
            draw_creature(&mut buf, &genome.species, &mind.mood);
            if let Some(ref mut data) = image.data {
                data.copy_from_slice(buf.as_raw());
            }
        }

        // Mark root so we don't attach again
        info!("Attaching pixel creature to entity {:?}", entity);
        commands.entity(entity).insert(HasPixelCreature);

        // Spawn pixel sprite as child of the creature root
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
            Transform::from_xyz(0.0, 0.0, 5.0), // in front of everything else
        ));
    }
}

/// Color palette for each species.
struct Palette {
    body: Rgba<u8>,
    body_light: Rgba<u8>,
    eye: Rgba<u8>,
    mouth: Rgba<u8>,
    accent: Rgba<u8>,     // ears, wings, crests, tentacles
    #[allow(dead_code)]
    outline: Rgba<u8>,
}

fn palette(species: &Species) -> Palette {
    match species {
        Species::Moluun => Palette {
            body:       Rgba([140, 215, 240, 255]),
            body_light: Rgba([180, 235, 250, 255]),
            eye:        Rgba([30, 30, 40, 255]),
            mouth:      Rgba([30, 30, 40, 255]),
            accent:     Rgba([120, 195, 225, 255]),
            outline:    Rgba([50, 70, 85, 255]),
        },
        Species::Pylum => Palette {
            body:       Rgba([245, 210, 130, 255]),
            body_light: Rgba([255, 235, 175, 255]),
            eye:        Rgba([30, 30, 45, 255]),
            mouth:      Rgba([240, 150, 55, 255]),
            accent:     Rgba([230, 190, 110, 255]),
            outline:    Rgba([75, 55, 35, 255]),
        },
        Species::Skael => Palette {
            body:       Rgba([110, 170, 120, 255]),
            body_light: Rgba([145, 200, 150, 255]),
            eye:        Rgba([190, 155, 45, 255]),
            mouth:      Rgba([75, 95, 75, 255]),
            accent:     Rgba([150, 95, 75, 255]),
            outline:    Rgba([40, 55, 45, 255]),
        },
        Species::Nyxal => Palette {
            body:       Rgba([90, 55, 125, 255]),
            body_light: Rgba([115, 80, 150, 255]),
            eye:        Rgba([40, 175, 195, 255]),
            mouth:      Rgba([15, 15, 30, 255]),
            accent:     Rgba([95, 60, 135, 255]),
            outline:    Rgba([30, 20, 45, 255]),
        },
    }
}

/// Draws the creature onto a pixel buffer based on species and mood.
fn draw_creature(img: &mut RgbaImage, species: &Species, mood: &MoodState) {
    let p = palette(species);
    let w = img.width() as i32;
    let _h = img.height() as i32;
    let cx = w / 2;

    // Clear canvas
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }

    match species {
        Species::Moluun => draw_moluun(img, &p, cx, mood),
        Species::Pylum  => draw_pylum(img, &p, cx, mood),
        Species::Skael  => draw_skael(img, &p, cx, mood),
        Species::Nyxal  => draw_nyxal(img, &p, cx, mood),
    }
}

/// Safe pixel setter.
fn put(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && x < img.width() as i32 && y >= 0 && y < img.height() as i32 {
        img.put_pixel(x as u32, y as u32, color);
    }
}

/// Filled circle (trimmed corners to avoid stray pixels).
fn fill_circle(img: &mut RgbaImage, cx: i32, cy: i32, r: i32, color: Rgba<u8>) {
    let r_sq = r * r;
    for dy in -r..=r {
        for dx in -r..=r {
            let dist = dx * dx + dy * dy;
            // Slightly tighter than r² to cut diagonal corner pixels
            if dist < r_sq {
                put(img, cx + dx, cy + dy, color);
            }
        }
    }
}

/// Filled rectangle.
fn fill_rect(img: &mut RgbaImage, x: i32, y: i32, w: i32, h: i32, color: Rgba<u8>) {
    for dy in 0..h {
        for dx in 0..w {
            put(img, x + dx, y + dy, color);
        }
    }
}

// ===================================================================
// MOLUUN — round mammal, big ears on top
// ===================================================================

fn draw_moluun(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24; // body center Y
    let br = 18; // body radius

    // Ears (overlap body top edge — no gap)
    fill_circle(img, cx - 11, by - 14, 6, p.accent);
    fill_circle(img, cx + 11, by - 14, 6, p.accent);

    // Body
    fill_circle(img, cx, by, br, p.body);

    // Belly
    fill_circle(img, cx, by + 5, 12, p.body_light);

    // Feet (overlap body bottom — no gap)
    fill_rect(img, cx - 10, by + br - 4, 8, 7, p.body);
    fill_rect(img, cx + 3, by + br - 4, 8, 7, p.body);

    // Eyes — BIG (5x5), placed low on face
    let ey = by + 2;
    if *mood == MoodState::Sleeping {
        fill_rect(img, cx - 8, ey + 1, 5, 2, p.eye);
        fill_rect(img, cx + 4, ey + 1, 5, 2, p.eye);
    } else {
        fill_rect(img, cx - 9, ey, 5, 5, p.eye);
        fill_rect(img, cx + 5, ey, 5, 5, p.eye);
    }

    // Mouth
    if *mood != MoodState::Sleeping {
        fill_rect(img, cx - 2, by + 10, 4, 2, p.mouth);
    }
}

// ===================================================================
// PYLUM — round bird, small wings, beak
// ===================================================================

fn draw_pylum(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 16;

    // Wings (overlap body sides — no gap)
    fill_rect(img, cx - br + 1, by - 3, 6, 12, p.accent);
    fill_rect(img, cx + br - 6, by - 3, 6, 12, p.accent);

    // Body
    fill_circle(img, cx, by, br, p.body);

    // Belly
    fill_circle(img, cx, by + 4, 10, p.body_light);

    // Tuft on top (overlaps body)
    fill_rect(img, cx - 2, by - br - 2, 4, 5, p.body);

    // Feet (overlap body bottom)
    fill_rect(img, cx - 7, by + br - 3, 6, 6, p.mouth);
    fill_rect(img, cx + 2, by + br - 3, 6, 6, p.mouth);

    // Eyes — BIG
    let ey = by + 1;
    if *mood == MoodState::Sleeping {
        fill_rect(img, cx - 8, ey + 1, 5, 2, p.eye);
        fill_rect(img, cx + 4, ey + 1, 5, 2, p.eye);
    } else {
        fill_rect(img, cx - 9, ey, 5, 5, p.eye);
        fill_rect(img, cx + 5, ey, 5, 5, p.eye);
    }

    // Beak (simple block below eyes)
    fill_rect(img, cx - 3, by + 8, 6, 4, p.mouth);
}

// ===================================================================
// SKAEL — round reptile, crests on top
// ===================================================================

fn draw_skael(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 24;
    let br = 16;

    // Crests (overlap body top — no gap)
    fill_rect(img, cx - 10, by - br + 1, 5, 8, p.accent);
    fill_rect(img, cx + 6, by - br + 1, 5, 8, p.accent);

    // Body
    fill_circle(img, cx, by, br, p.body);

    // Belly
    fill_circle(img, cx, by + 4, 10, p.body_light);

    // Tail (overlaps body bottom)
    fill_rect(img, cx - 3, by + br - 3, 6, 10, p.body);

    // Feet (overlap body bottom)
    fill_rect(img, cx - 10, by + br - 4, 7, 7, p.body);
    fill_rect(img, cx + 4, by + br - 4, 7, 7, p.body);

    // Eyes — BIG golden
    let ey = by + 1;
    if *mood == MoodState::Sleeping {
        fill_rect(img, cx - 8, ey + 1, 5, 2, p.eye);
        fill_rect(img, cx + 4, ey + 1, 5, 2, p.eye);
    } else {
        fill_rect(img, cx - 9, ey, 5, 5, p.eye);
        fill_rect(img, cx + 5, ey, 5, 5, p.eye);
    }

    // Snout
    fill_rect(img, cx - 3, by + 9, 6, 3, p.mouth);
}

// ===================================================================
// NYXAL — round squid, tentacles hanging down
// ===================================================================

fn draw_nyxal(img: &mut RgbaImage, p: &Palette, cx: i32, mood: &MoodState) {
    let by = 18;
    let br = 14;

    // Mantle (overlaps body top)
    fill_circle(img, cx, by - 5, 10, p.accent);

    // Tentacles (overlap body bottom — drawn BEFORE body so body covers the join)
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

    // Body (on top of tentacles)
    fill_circle(img, cx, by, br, p.body);

    // Belly
    fill_circle(img, cx, by + 3, 9, p.body_light);

    // Eyes — BIG cyan
    let ey = by + 1;
    if *mood == MoodState::Sleeping {
        fill_rect(img, cx - 8, ey + 1, 5, 2, p.eye);
        fill_rect(img, cx + 4, ey + 1, 5, 2, p.eye);
    } else {
        fill_rect(img, cx - 9, ey, 5, 5, p.eye);
        fill_rect(img, cx + 5, ey, 5, 5, p.eye);
    }
}

// ===================================================================
// SYSTEM — draws and updates the pixel creature each frame
// ===================================================================

/// Updates the pixel creature texture based on current genome and mood.
fn update_pixel_creature(
    genome: Res<Genome>,
    mind: Res<Mind>,
    mut images: ResMut<Assets<BevyImage>>,
    creature_q: Query<&Sprite, With<PixelCreature>>,
    mut pixel_buf: Local<Option<RgbaImage>>,
) {
    // Initialize buffer on first run
    if pixel_buf.is_none() {
        *pixel_buf = Some(RgbaImage::new(CANVAS_W, CANVAS_H));
    }
    let buf = pixel_buf.as_mut().unwrap();

    // Only redraw when mood changes (optimization)
    if !mind.is_changed() && !genome.is_changed() {
        return;
    }

    // Draw the creature
    draw_creature(buf, &genome.species, &mind.mood);

    // Update the Bevy image asset
    for sprite in creature_q.iter() {
        if let Some(image) = images.get_mut(&sprite.image) {
            if let Some(ref mut data) = image.data {
                data.copy_from_slice(buf.as_raw());
            }
        }
    }
}

/// Creates a blank pixel creature image and returns the handle.
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

    // Nearest-neighbor filtering for crisp pixel art (no blurring)
    image.sampler = ImageSampler::nearest();

    images.add(image)
}
