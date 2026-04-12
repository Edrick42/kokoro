//! Creature skin renderer — the visible surface of every Kobara.
//!
//! Draws creatures to a 64x64 pixel buffer, scaled 5x for display.
//! Species-specific drawing lives in submodules (one per species).
//! Anatomy data (skeleton, muscles, fat, skin) drives visual parameters.

pub mod moluun;
pub mod nyxal;
pub mod params;
pub mod pylum;
pub mod skael;

use bevy::prelude::*;
use bevy::image::ImageSampler;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{RgbaImage, Rgba};

type BevyImage = Image;

use crate::game::state::AppState;
use crate::creature::anatomy::AnatomyState;
use crate::creature::pose::PoseState;
use crate::creature::reactions::ExpressionOverride;
use crate::creature::species::CreatureRoot;
use crate::genome::{Genome, Species};
use crate::mind::{Mind, MoodState};
use crate::visuals::evolution::{GrowthState, GrowthStage};

use params::SkinParams;

const CANVAS_W: u32 = 64;
const CANVAS_H: u32 = 64;
const DISPLAY_SCALE: f32 = 5.0;

#[derive(Component)]
pub struct CreatureSkin;

#[derive(Component)]
pub struct HasCreatureSkin;

pub struct SkinPlugin;

impl Plugin for SkinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (attach_skin, update_skin)
                .chain()
                .run_if(in_state(AppState::Gameplay)),
        );
    }
}

// ===================================================================
// SYSTEMS
// ===================================================================

fn attach_skin(
    mut commands: Commands,
    mut images: ResMut<Assets<BevyImage>>,
    query: Query<Entity, (With<CreatureRoot>, Without<HasCreatureSkin>)>,
    genome: Res<Genome>,
    mind: Res<Mind>,
    growth: Res<GrowthState>,
    anatomy: Option<Res<AnatomyState>>,
    pose: Res<PoseState>,
    expression: Res<ExpressionOverride>,
) {
    for entity in query.iter() {
        let handle = create_skin_texture(&mut images);

        if let Some(image) = images.get_mut(&handle) {
            let mut buf = RgbaImage::new(CANVAS_W, CANVAS_H);
            let sp = anatomy.as_ref()
                .map(|a| SkinParams::from_anatomy(a, &genome.species, &growth.stage))
                .unwrap_or_else(SkinParams::healthy_default);
            draw_creature(&mut buf, &genome.species, &mind.mood, &growth.stage, &sp, &pose, &expression);
            if let Some(ref mut data) = image.data {
                data.copy_from_slice(buf.as_raw());
            }
        }

        info!("Attaching skin to entity {:?}", entity);
        commands.entity(entity).insert(HasCreatureSkin);

        commands.entity(entity).with_child((
            CreatureSkin,
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

fn update_skin(
    genome: Res<Genome>,
    mind: Res<Mind>,
    growth: Res<GrowthState>,
    anatomy: Option<Res<AnatomyState>>,
    pose: Res<PoseState>,
    expression: Res<ExpressionOverride>,
    mut images: ResMut<Assets<BevyImage>>,
    creature_q: Query<&Sprite, With<CreatureSkin>>,
    mut pixel_buf: Local<Option<RgbaImage>>,
) {
    if pixel_buf.is_none() {
        *pixel_buf = Some(RgbaImage::new(CANVAS_W, CANVAS_H));
    }
    let buf = pixel_buf.as_mut().unwrap();

    let anatomy_changed = anatomy.as_ref().map_or(false, |a| a.is_changed());
    if !mind.is_changed() && !genome.is_changed() && !growth.is_changed()
        && !anatomy_changed && !pose.is_changed() && !expression.is_changed() {
        return;
    }

    let sp = anatomy.as_ref()
        .map(|a| SkinParams::from_anatomy(a, &genome.species, &growth.stage))
        .unwrap_or_else(SkinParams::healthy_default);
    draw_creature(buf, &genome.species, &mind.mood, &growth.stage, &sp, &pose, &expression);

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

/// Computed pixel offsets from skeletal pose — passed to draw functions.
#[allow(dead_code)]
pub struct PoseOffsets {
    /// Head horizontal shift (from neck angle).
    pub head_dx: i32,
    /// Head vertical dip (from neck angle).
    pub head_dy: i32,
    /// Tail horizontal swing.
    pub tail_dx: i32,
}

fn draw_creature(img: &mut RgbaImage, species: &Species, mood: &MoodState, stage: &GrowthStage, sp: &SkinParams, pose: &PoseState, expr: &ExpressionOverride) {
    let cx = img.width() as i32 / 2;

    // Clear canvas
    for pixel in img.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }

    // Compute pixel offsets from pose joint angles
    let neck = pose.angle("neck");
    let tail = pose.angle("tail");
    let offsets = PoseOffsets {
        head_dx: (neck.to_radians().sin() * 4.0) as i32,
        head_dy: (neck.to_radians().sin().abs() * 2.0) as i32,
        tail_dx: (tail.to_radians().sin() * 3.0) as i32,
    };

    // Override mood for expression if active
    let effective_mood = if expr.is_active() {
        // Map expression override to a temporary mood for eye/mouth rendering
        match (expr.eyes, expr.mouth) {
            (2, _)  => MoodState::Sleeping,  // half-closed eyes → sleeping look
            (1, 2)  => MoodState::Playful,   // wide eyes + big smile → playful
            (-1, _) => MoodState::Sick,      // squint → sick look
            (_, 1)  => MoodState::Hungry,    // mouth open → hungry look
            _       => mood.clone(),
        }
    } else {
        mood.clone()
    };

    // Body shifts opposite to head (natural counterbalance)
    let body_dx = -offsets.head_dx / 2;

    match stage {
        GrowthStage::Egg   => draw_egg(img, species),
        GrowthStage::Cub   => draw_cub(img, species, &effective_mood, cx + offsets.head_dx, sp),
        GrowthStage::Young => draw_young(img, species, &effective_mood, cx + offsets.head_dx + body_dx, sp),
        GrowthStage::Adult => draw_adult(img, species, &effective_mood, cx + offsets.head_dx + body_dx, sp),
        GrowthStage::Elder => draw_elder(img, species, &effective_mood, cx + offsets.head_dx + body_dx, sp),
    }
}

fn draw_cub(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32, sp: &SkinParams) {
    let p = palette(species);
    // Cubs: fat affects belly size slightly (baby fat)
    let belly_mod = 0.8 + sp.belly * 0.4; // 0.8 to 1.2
    match species {
        Species::Moluun => moluun::draw_cub(img, &p, cx, mood),
        Species::Pylum  => pylum::draw_cub(img, &p, cx, mood),
        Species::Skael  => skael::draw_cub(img, &p, cx, mood),
        Species::Nyxal  => nyxal::draw_cub(img, &p, cx, mood),
    }
    // Overlay: belly modulated by fat (draw extra belly pixels if fat)
    if belly_mod > 1.05 {
        let extra = ((belly_mod - 1.0) * 6.0) as i32;
        let belly_color = fade(p.body_light, 0.05);
        for dx in -extra..=extra {
            put(img, cx + dx, 38, belly_color);
        }
    }
}

fn draw_young(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32, sp: &SkinParams) {
    let p = palette(species);
    match species {
        Species::Moluun => moluun::draw_young(img, &p, cx, mood),
        Species::Pylum  => pylum::draw_young(img, &p, cx, mood),
        Species::Skael  => skael::draw_young(img, &p, cx, mood),
        Species::Nyxal  => nyxal::draw_young(img, &p, cx, mood),
    }
    // Fat overlay: wider belly when well-fed
    if sp.belly > 0.6 {
        let extra = ((sp.belly - 0.5) * 8.0) as i32;
        let belly_color = fade(p.body_light, 0.05);
        for dx in -extra..=extra {
            put(img, cx + dx, 38, belly_color);
            put(img, cx + dx, 39, belly_color);
        }
    }
}

fn draw_adult(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32, sp: &SkinParams) {
    let p = palette(species);
    match species {
        Species::Moluun => moluun::draw_adult(img, &p, cx, mood),
        Species::Pylum  => pylum::draw_adult(img, &p, cx, mood),
        Species::Skael  => skael::draw_adult(img, &p, cx, mood),
        Species::Nyxal  => nyxal::draw_adult(img, &p, cx, mood),
    }
    // Fat affects adult body visibly: wider body when bulk > 1.0, thinner when < 0.9
    let bulk_extra = ((sp.bulk - 1.0) * 10.0) as i32; // -3 to +3 px
    if bulk_extra != 0 {
        let color = if bulk_extra > 0 { fade(p.body, 0.05) } else { Rgba([0, 0, 0, 0]) };
        let body_y = 30; // approximate adult body center
        for dy in -5..=5 {
            if bulk_extra > 0 {
                // Fat: add extra body pixels on both sides
                for i in 0..bulk_extra {
                    put(img, cx - 17 - i, body_y + dy, color);
                    put(img, cx + 17 + i, body_y + dy, color);
                }
            }
            // Thin: handled by vitality color fade (already in SkinParams)
        }
    }
    // Belly size from fat
    if sp.belly > 0.6 {
        let belly_extra = ((sp.belly - 0.5) * 6.0) as i32;
        let belly_color = fade(p.body_light, 0.08);
        for dx in -belly_extra..=belly_extra {
            put(img, cx + dx, 42, belly_color);
            put(img, cx + dx, 43, belly_color);
        }
    }
}

fn draw_elder(img: &mut RgbaImage, species: &Species, mood: &MoodState, cx: i32, _sp: &SkinParams) {
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

pub fn create_skin_texture(images: &mut Assets<BevyImage>) -> Handle<BevyImage> {
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
