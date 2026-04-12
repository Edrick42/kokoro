//! Runtime pixel art creature renderer.
//!
//! Draws creatures directly to a 64x64 pixel buffer and displays
//! as a scaled-up sprite. No pre-made PNGs — the creature IS the pixels.
//!
//! Species-specific drawing lives in submodules (one per species).

pub mod moluun;
pub mod pylum;
pub mod skael;
pub mod nyxal;

use bevy::prelude::*;
use bevy::image::ImageSampler;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{RgbaImage, Rgba};

type BevyImage = Image;

use crate::game::state::AppState;
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
        app.add_systems(
            Update,
            (attach_pixel_creature, update_pixel_creature)
                .chain()
                .run_if(in_state(AppState::Gameplay)),
        );
    }
}

// ===================================================================
// SYSTEMS
// ===================================================================

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

pub struct Palette {
    pub body: Rgba<u8>,
    pub body_light: Rgba<u8>,
    pub eye: Rgba<u8>,
    pub mouth: Rgba<u8>,
    pub accent: Rgba<u8>,
    pub egg: Rgba<u8>,
    pub egg_spot: Rgba<u8>,
}

pub const NEAR_BLACK_PX: Rgba<u8> = Rgba([27, 19, 13, 255]);
#[allow(dead_code)]
pub const CREAM_PX: Rgba<u8> = Rgba([217, 199, 174, 255]);

pub fn palette(species: &Species) -> Palette {
    match species {
        Species::Moluun => Palette {
            body:       Rgba([140, 180, 200, 255]),
            body_light: Rgba([210, 200, 185, 255]),
            eye:        NEAR_BLACK_PX,
            mouth:      Rgba([60, 45, 40, 255]),
            accent:     Rgba([120, 155, 175, 255]),
            egg:        Rgba([220, 200, 180, 255]),
            egg_spot:   Rgba([160, 190, 205, 255]),
        },
        Species::Pylum => Palette {
            body:       Rgba([235, 190, 80, 255]),
            body_light: Rgba([245, 220, 150, 255]),
            eye:        Rgba([30, 25, 20, 255]),
            mouth:      Rgba([230, 120, 30, 255]),
            accent:     Rgba([200, 155, 55, 255]),
            egg:        Rgba([235, 225, 200, 255]),
            egg_spot:   Rgba([180, 140, 80, 255]),
        },
        Species::Skael => Palette {
            body:       Rgba([45, 120, 85, 255]),
            body_light: Rgba([90, 160, 120, 255]),
            eye:        Rgba([190, 155, 40, 255]),
            mouth:      Rgba([35, 65, 50, 255]),
            accent:     Rgba([100, 75, 55, 255]),
            egg:        Rgba([70, 130, 95, 255]),
            egg_spot:   Rgba([40, 80, 60, 255]),
        },
        Species::Nyxal => Palette {
            body:       Rgba([80, 45, 110, 255]),
            body_light: Rgba([60, 35, 80, 255]),
            eye:        Rgba([40, 180, 200, 255]),
            mouth:      NEAR_BLACK_PX,
            accent:     Rgba([65, 40, 95, 255]),
            egg:        Rgba([70, 45, 100, 180]),
            egg_spot:   Rgba([50, 180, 200, 200]),
        },
    }
}

fn elder_palette(species: &Species) -> Palette {
    let base = palette(species);
    Palette {
        body:       fade(base.body, 0.3),
        body_light: fade(base.body_light, 0.3),
        eye:        base.eye,
        mouth:      fade(base.mouth, 0.2),
        accent:     fade(base.accent, 0.3),
        egg:        base.egg,
        egg_spot:   base.egg_spot,
    }
}

/// Fades a color toward gray by `amount` (0.0 = unchanged, 1.0 = full gray).
pub fn fade(c: Rgba<u8>, amount: f32) -> Rgba<u8> {
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

fn draw_cub(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32) {
    let p = palette(species);
    match species {
        Species::Moluun => moluun::draw_cub(img, &p, cx, mood),
        Species::Pylum  => pylum::draw_cub(img, &p, cx, mood),
        Species::Skael  => skael::draw_cub(img, &p, cx, mood),
        Species::Nyxal  => nyxal::draw_cub(img, &p, cx, mood),
    }
}

fn draw_young(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32) {
    let p = palette(species);
    match species {
        Species::Moluun => moluun::draw_young(img, &p, cx, mood),
        Species::Pylum  => pylum::draw_young(img, &p, cx, mood),
        Species::Skael  => skael::draw_young(img, &p, cx, mood),
        Species::Nyxal  => nyxal::draw_young(img, &p, cx, mood),
    }
}

fn draw_adult(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32) {
    let p = palette(species);
    match species {
        Species::Moluun => moluun::draw_adult(img, &p, cx, mood),
        Species::Pylum  => pylum::draw_adult(img, &p, cx, mood),
        Species::Skael  => skael::draw_adult(img, &p, cx, mood),
        Species::Nyxal  => nyxal::draw_adult(img, &p, cx, mood),
    }
}

fn draw_elder(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32) {
    let p = elder_palette(species);
    let base_p = palette(species);

    match species {
        Species::Moluun => {
            moluun::draw_adult(img, &p, cx, mood);
            moluun::draw_elder_details(img, cx);
        }
        Species::Pylum => {
            pylum::draw_adult(img, &p, cx, mood);
            pylum::draw_elder_details(img, &base_p, cx);
        }
        Species::Skael => {
            skael::draw_adult(img, &p, cx, mood);
            skael::draw_elder_details(img, &base_p, cx);
        }
        Species::Nyxal => {
            nyxal::draw_adult(img, &p, cx, mood);
            nyxal::draw_elder_details(img, cx);
        }
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
        Species::Moluun => moluun::draw_egg(img, &p, cx),
        Species::Pylum  => pylum::draw_egg(img, &p, cx),
        Species::Skael  => skael::draw_egg(img, &p, cx),
        Species::Nyxal  => nyxal::draw_egg(img, &p, cx),
    }
}

// ===================================================================
// PRIMITIVES (shared by all species)
// ===================================================================

pub fn put(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && x < img.width() as i32 && y >= 0 && y < img.height() as i32 {
        img.put_pixel(x as u32, y as u32, color);
    }
}

pub fn fill_circle(img: &mut RgbaImage, cx: i32, cy: i32, r: i32, color: Rgba<u8>) {
    let r_sq = r * r;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy < r_sq {
                put(img, cx + dx, cy + dy, color);
            }
        }
    }
}

pub fn fill_rect(img: &mut RgbaImage, x: i32, y: i32, w: i32, h: i32, color: Rgba<u8>) {
    for dy in 0..h {
        for dx in 0..w {
            put(img, x + dx, y + dy, color);
        }
    }
}

pub fn fill_ellipse(img: &mut RgbaImage, cx: i32, cy: i32, rx: i32, ry: i32, color: Rgba<u8>) {
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

pub fn draw_eyes(img: &mut RgbaImage, cx: i32, ey: i32, size: i32, gap: i32, mood: &MoodState, color: Rgba<u8>) {
    if *mood == MoodState::Sleeping {
        fill_rect(img, cx - gap - size, ey + size / 3, size, 2, color);
        fill_rect(img, cx + gap, ey + size / 3, size, 2, color);
    } else {
        fill_rect(img, cx - gap - size, ey, size, size, color);
        fill_rect(img, cx + gap, ey, size, size, color);
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
