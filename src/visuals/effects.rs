//! Mood-driven visual effects — runtime pixel art.
//!
//! Spawns floating pixel art effects above the creature based on mood:
//! - **Sleeping** → chunky ZZZ letters
//! - **Playful** → pixel hearts
//! - **Lonely** → pixel rain drops
//! - **Sick** → pixel dizzy stars
//!
//! All effects are drawn at runtime to small pixel buffers (16×16),
//! displayed at 4× scale with nearest-neighbor — matching the retro style.

use bevy::prelude::*;
use bevy::image::ImageSampler;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{RgbaImage, Rgba};

use crate::mind::{Mind, MoodState};
use crate::creature::species::CreatureRoot;

const EFFECT_SIZE: u32 = 16;
const EFFECT_SCALE: f32 = 4.0;

// Palette colors as pixel values
const NEAR_BLACK_PX: Rgba<u8> = Rgba([27, 19, 13, 255]);
const RED_PX: Rgba<u8> = Rgba([217, 13, 67, 255]);
const TEAL_PX: Rgba<u8> = Rgba([1, 105, 112, 255]);
const GOLD_PX: Rgba<u8> = Rgba([217, 164, 4, 255]);

#[derive(Component)]
pub struct MoodEffect;

#[derive(Resource)]
struct CurrentEffectMood(Option<MoodState>);

#[derive(Component)]
pub struct EffectAnimation {
    elapsed: f32,
    base_y: f32,
    sway_amp: f32,
    float_speed: f32,
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentEffectMood(None))
           .add_systems(Update, (spawn_mood_effects, animate_effects).chain());
    }
}

fn spawn_mood_effects(
    mut commands: Commands,
    mind: Res<Mind>,
    mut images: ResMut<Assets<Image>>,
    mut current: ResMut<CurrentEffectMood>,
    root_q: Query<Entity, With<CreatureRoot>>,
    effect_q: Query<Entity, With<MoodEffect>>,
) {
    if !mind.is_changed() { return; }
    if current.0.as_ref() == Some(&mind.mood) { return; }
    current.0 = Some(mind.mood.clone());

    for entity in effect_q.iter() {
        commands.entity(entity).despawn();
    }

    let draw_fn: Option<fn(&mut RgbaImage)> = match &mind.mood {
        MoodState::Sleeping => Some(draw_zzz),
        MoodState::Playful  => Some(draw_heart),
        MoodState::Lonely   => Some(draw_rain),
        MoodState::Sick     => Some(draw_star),
        _ => None,
    };

    let Some(draw) = draw_fn else { return };
    let Ok(root) = root_q.single() else { return };

    // Create pixel art effect
    let mut buf = RgbaImage::new(EFFECT_SIZE, EFFECT_SIZE);
    draw(&mut buf);

    let mut image = Image::new_fill(
        Extent3d { width: EFFECT_SIZE, height: EFFECT_SIZE, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );
    image.sampler = ImageSampler::nearest();
    if let Some(ref mut data) = image.data {
        data.copy_from_slice(buf.as_raw());
    }
    let handle = images.add(image);

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: handle,
                custom_size: Some(Vec2::splat(EFFECT_SIZE as f32 * EFFECT_SCALE)),
                ..default()
            },
            Transform::from_xyz(40.0, 90.0, 6.0),
            MoodEffect,
            EffectAnimation {
                elapsed: 0.0,
                base_y: 90.0,
                sway_amp: 5.0,
                float_speed: 10.0,
            },
        ));
    });
}

fn animate_effects(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut EffectAnimation), With<MoodEffect>>,
) {
    for (mut transform, mut anim) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        let float_offset = (anim.elapsed * anim.float_speed * 0.3).sin() * 8.0;
        transform.translation.y = anim.base_y + float_offset;

        let sway = (anim.elapsed * 1.5).sin() * anim.sway_amp;
        transform.translation.x = 40.0 + sway;
    }
}

// ===================================================================
// PIXEL ART EFFECT DRAWING — 16×16 buffers
// ===================================================================

fn put(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && x < img.width() as i32 && y >= 0 && y < img.height() as i32 {
        img.put_pixel(x as u32, y as u32, color);
    }
}

fn fill_rect(img: &mut RgbaImage, x: i32, y: i32, w: i32, h: i32, color: Rgba<u8>) {
    for dy in 0..h {
        for dx in 0..w {
            put(img, x + dx, y + dy, color);
        }
    }
}

/// ZZZ — bold pixel "Z" letters, stacked and offset
fn draw_zzz(img: &mut RgbaImage) {
    let c = NEAR_BLACK_PX;
    // Big Z (bottom-left)
    fill_rect(img, 1, 9, 5, 1, c);   // top bar
    put(img, 4, 10, c);               // diagonal
    put(img, 3, 11, c);
    put(img, 2, 12, c);
    fill_rect(img, 1, 13, 5, 1, c);  // bottom bar

    // Medium Z (middle)
    fill_rect(img, 5, 5, 4, 1, c);
    put(img, 7, 6, c);
    put(img, 6, 7, c);
    fill_rect(img, 5, 8, 4, 1, c);

    // Small Z (top-right)
    fill_rect(img, 9, 2, 3, 1, c);
    put(img, 10, 3, c);
    fill_rect(img, 9, 4, 3, 1, c);
}

/// Heart — pixel art heart shape in RED
fn draw_heart(img: &mut RgbaImage) {
    let c = RED_PX;
    // Top bumps
    fill_rect(img, 3, 4, 3, 2, c);
    fill_rect(img, 8, 4, 3, 2, c);
    // Middle fill
    fill_rect(img, 2, 5, 10, 3, c);
    // Narrowing rows
    fill_rect(img, 3, 8, 8, 1, c);
    fill_rect(img, 4, 9, 6, 1, c);
    fill_rect(img, 5, 10, 4, 1, c);
    fill_rect(img, 6, 11, 2, 1, c);
    put(img, 7, 12, c); // tip (the extra pixel for symmetry doesn't matter at this scale)
}

/// Rain — pixel rain drops falling down in TEAL
fn draw_rain(img: &mut RgbaImage) {
    let c = TEAL_PX;
    // Drop 1 (left)
    fill_rect(img, 3, 3, 2, 4, c);
    put(img, 3, 7, c);
    // Drop 2 (center, lower)
    fill_rect(img, 7, 6, 2, 4, c);
    put(img, 7, 10, c);
    // Drop 3 (right, higher)
    fill_rect(img, 11, 1, 2, 4, c);
    put(img, 11, 5, c);
}

/// Star — dizzy star shape in GOLD
fn draw_star(img: &mut RgbaImage) {
    let c = GOLD_PX;
    // Center cross
    fill_rect(img, 6, 3, 2, 10, c);  // vertical bar
    fill_rect(img, 3, 6, 10, 2, c);  // horizontal bar
    // Diagonal points
    put(img, 4, 4, c); put(img, 5, 5, c);    // top-left
    put(img, 9, 4, c); put(img, 8, 5, c);    // top-right (fixed: 8 not 10)
    put(img, 4, 9, c); put(img, 5, 8, c);    // bottom-left
    put(img, 9, 9, c); put(img, 8, 8, c);    // bottom-right (fixed: 8 not 10)
}
