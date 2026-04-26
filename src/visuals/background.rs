//! Biome background with day/night cycle — pixel art drawn in code.
//!
//! Each species has a unique biome background drawn to a 64×64 pixel buffer
//! (same pipeline as creature skins). Scaled to fill the screen.
//!
//! - Moluun → Verdance (bioluminescent forest)
//! - Pylum  → Highlands (rocky cliffs, clouds)
//! - Skael  → Shallows (crystal caves, water shimmer)
//! - Nyxal  → Depths (deep ocean, bioluminescence)

use bevy::prelude::*;
use bevy::image::ImageSampler;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{RgbaImage, Rgba};

use crate::game::state::{AppState, GameplayEntity};
use crate::config::ui::palette;
use crate::genome::{Genome, Species};
use crate::world::daycycle::{DayCycle, TimeOfDay};

const BG_W: u32 = 64;
const BG_H: u32 = 64;
const BG_SCALE: f32 = 11.0; // fills 704×704 area

#[derive(Component)]
struct BiomeBackground;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Gameplay), spawn_background)
           .add_systems(Update, update_background.run_if(in_state(AppState::Gameplay)));
    }
}

// ===================================================================
// SYSTEMS
// ===================================================================

fn spawn_background(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    genome: Res<Genome>,
    cycle: Res<DayCycle>,
) {
    let mut buf = RgbaImage::new(BG_W, BG_H);
    draw_biome(&mut buf, &genome.species, &cycle.time_of_day);

    let handle = create_bg_texture(&mut images, &buf);

    // ClearColor matches the dominant background color
    let bg_color = background_tint(&genome.species, &cycle.time_of_day);
    commands.insert_resource(ClearColor(bg_color));

    commands.spawn((
        GameplayEntity,
        BiomeBackground,
        Sprite {
            image: handle,
            custom_size: Some(Vec2::new(BG_W as f32 * BG_SCALE, BG_H as f32 * BG_SCALE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
}

fn update_background(
    genome: Res<Genome>,
    cycle: Res<DayCycle>,
    mut clear: ResMut<ClearColor>,
    mut images: ResMut<Assets<Image>>,
    bg_q: Query<&Sprite, With<BiomeBackground>>,
    mut pixel_buf: Local<Option<RgbaImage>>,
) {
    if !genome.is_changed() && !cycle.is_changed() { return; }

    if pixel_buf.is_none() {
        *pixel_buf = Some(RgbaImage::new(BG_W, BG_H));
    }
    let buf = pixel_buf.as_mut().unwrap();

    draw_biome(buf, &genome.species, &cycle.time_of_day);

    let bg_color = background_tint(&genome.species, &cycle.time_of_day);
    clear.0 = bg_color;

    for sprite in bg_q.iter() {
        if let Some(image) = images.get_mut(&sprite.image) {
            if let Some(ref mut data) = image.data {
                data.copy_from_slice(buf.as_raw());
            }
        }
    }
}

fn create_bg_texture(images: &mut Assets<Image>, buf: &RgbaImage) -> Handle<Image> {
    let mut image = Image::new_fill(
        Extent3d {
            width: BG_W,
            height: BG_H,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );
    if let Some(ref mut data) = image.data {
        data.copy_from_slice(buf.as_raw());
    }
    image.sampler = ImageSampler::nearest();
    images.add(image)
}

// ===================================================================
// COLOR HELPERS
// ===================================================================

fn background_tint(species: &Species, time: &TimeOfDay) -> Color {
    let tint = match species {
        Species::Moluun => palette::GOLD,
        Species::Pylum  => palette::ORANGE,
        Species::Skael  => palette::TEAL,
        Species::Nyxal  => palette::RED,
    };

    let time_base = match time {
        TimeOfDay::Morning   => blend(palette::CREAM, Color::srgb(0.95, 0.92, 0.88), 0.3),
        TimeOfDay::Afternoon => palette::CREAM,
        TimeOfDay::Sunset    => blend(palette::CREAM, palette::ORANGE, 0.25),
        TimeOfDay::Night     => blend(palette::CREAM, palette::NEAR_BLACK, 0.65),
    };

    let tint_amount = if matches!(time, TimeOfDay::Night) { 0.20 } else { 0.12 };
    blend(time_base, tint, tint_amount)
}

fn blend(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgb(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
    )
}

/// Blend two pixel colors.
fn px_blend(a: Rgba<u8>, b: Rgba<u8>, t: f32) -> Rgba<u8> {
    Rgba([
        (a[0] as f32 * (1.0 - t) + b[0] as f32 * t) as u8,
        (a[1] as f32 * (1.0 - t) + b[1] as f32 * t) as u8,
        (a[2] as f32 * (1.0 - t) + b[2] as f32 * t) as u8,
        255,
    ])
}

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

// ===================================================================
// BIOME DRAWING
// ===================================================================

fn draw_biome(img: &mut RgbaImage, species: &Species, time: &TimeOfDay) {
    match species {
        Species::Moluun => draw_verdance(img, time),
        Species::Pylum  => draw_highlands(img, time),
        Species::Skael  => draw_shallows(img, time),
        Species::Nyxal  => draw_depths(img, time),
    }
}

/// Night darkening factor for pixel colors.
fn night_tint(color: Rgba<u8>, time: &TimeOfDay) -> Rgba<u8> {
    let dark = Rgba([15, 12, 20, 255]);
    match time {
        TimeOfDay::Night   => px_blend(color, dark, 0.55),
        TimeOfDay::Sunset  => px_blend(color, Rgba([60, 40, 30, 255]), 0.20),
        TimeOfDay::Morning => px_blend(color, Rgba([240, 230, 215, 255]), 0.10),
        TimeOfDay::Afternoon => color,
    }
}

// -------------------------------------------------------------------
// VERDANCE — Moluun's bioluminescent forest
// -------------------------------------------------------------------

fn draw_verdance(img: &mut RgbaImage, time: &TimeOfDay) {
    let sky       = night_tint(Rgba([170, 200, 160, 255]), time);
    let sky_light = night_tint(Rgba([195, 215, 180, 255]), time);
    let trunk     = night_tint(Rgba([80, 60, 45, 255]), time);
    let trunk_dk  = night_tint(Rgba([55, 40, 30, 255]), time);
    let leaf      = night_tint(Rgba([65, 130, 70, 255]), time);
    let leaf_lt   = night_tint(Rgba([90, 160, 95, 255]), time);
    let moss      = night_tint(Rgba([50, 100, 55, 255]), time);
    let ground    = night_tint(Rgba([75, 65, 50, 255]), time);
    let ground_lt = night_tint(Rgba([95, 80, 60, 255]), time);
    let firefly   = Rgba([200, 220, 120, 180]); // glows regardless of time

    // Sky gradient
    for y in 0..30 {
        let t = y as f32 / 30.0;
        let color = px_blend(sky_light, sky, t);
        fill_rect(img, 0, y, 64, 1, color);
    }

    // Canopy (leaf layer at top)
    for &(x, y, r) in &[(10,8,8), (30,5,10), (50,9,7), (20,12,6), (42,7,9)] {
        for dy in -(r as i32)..=(r as i32) {
            for dx in -(r as i32)..=(r as i32) {
                if dx*dx + dy*dy < r*r {
                    let c = if (dx + dy) % 3 == 0 { leaf_lt } else { leaf };
                    put(img, x + dx, y + dy, c);
                }
            }
        }
    }

    // Tree trunks (left and right — the center is reserved for the creature)
    fill_rect(img, 8, 14, 4, 40, trunk);
    fill_rect(img, 9, 14, 2, 40, trunk_dk);
    fill_rect(img, 48, 15, 4, 39, trunk);
    fill_rect(img, 49, 15, 2, 39, trunk_dk);

    // Ground
    fill_rect(img, 0, 52, 64, 12, ground);
    // Ground texture
    for &(x, y) in &[(5,53), (15,54), (25,53), (38,55), (50,54), (12,56), (42,53), (55,55)] {
        put(img, x, y, ground_lt);
    }

    // Moss patches
    for &(x, y, w) in &[(0,51,12), (18,52,8), (35,51,10), (52,52,12)] {
        fill_rect(img, x, y, w, 2, moss);
    }

    // Fireflies (small glowing dots — visible even at night)
    if matches!(time, TimeOfDay::Sunset | TimeOfDay::Night) {
        for &(x, y) in &[(14,22), (35,18), (52,25), (22,30), (45,35), (8,28), (58,20)] {
            put(img, x, y, firefly);
        }
    }
}

// -------------------------------------------------------------------
// HIGHLANDS — Pylum's windswept cliffs
// -------------------------------------------------------------------

fn draw_highlands(img: &mut RgbaImage, time: &TimeOfDay) {
    let sky       = night_tint(Rgba([160, 185, 210, 255]), time);
    let sky_light = night_tint(Rgba([200, 215, 235, 255]), time);
    let cloud     = night_tint(Rgba([230, 225, 220, 200]), time);
    let rock      = night_tint(Rgba([120, 105, 90, 255]), time);
    let rock_lt   = night_tint(Rgba([150, 135, 115, 255]), time);
    let rock_dk   = night_tint(Rgba([85, 75, 65, 255]), time);
    let peak      = night_tint(Rgba([170, 160, 150, 255]), time);
    let peak_snow = night_tint(Rgba([220, 215, 210, 255]), time);
    let _ground    = night_tint(Rgba([130, 115, 85, 255]), time);
    let grass     = night_tint(Rgba([110, 140, 80, 255]), time);
    let wind      = Rgba([200, 210, 220, 100]);

    // Sky gradient
    for y in 0..35 {
        let t = y as f32 / 35.0;
        let color = px_blend(sky_light, sky, t);
        fill_rect(img, 0, y, 64, 1, color);
    }

    // Distant mountain peaks
    for &(cx, base_y, h) in &[(12, 20, 12), (35, 18, 16), (55, 22, 10)] {
        for dy in 0..h {
            let w = (h - dy) * 2;
            let x = cx - w / 2;
            let c = if dy < 3 { peak_snow } else { peak };
            fill_rect(img, x, base_y + dy, w, 1, c);
        }
    }

    // Clouds
    for &(x, y, w) in &[(5, 10, 14), (30, 6, 18), (50, 12, 10)] {
        fill_rect(img, x, y, w, 3, cloud);
        fill_rect(img, x + 2, y - 1, w - 4, 1, cloud);
    }

    // Rocky cliff platform
    fill_rect(img, 0, 35, 64, 29, rock);
    // Cliff face texture
    for &(x, y) in &[(5,37), (15,39), (28,36), (40,38), (52,37), (10,42), (35,40), (48,43)] {
        fill_rect(img, x, y, 3, 2, rock_lt);
    }
    for &(x, y) in &[(8,44), (22,41), (45,45), (55,42), (18,46), (38,44)] {
        put(img, x, y, rock_dk);
    }

    // Cliff edge (top of platform)
    fill_rect(img, 0, 34, 64, 2, rock_lt);

    // Grass tufts on cliff edge
    for &(x, w) in &[(3,4), (12,6), (25,5), (38,4), (50,7), (58,3)] {
        fill_rect(img, x, 33, w, 2, grass);
    }

    // Wind streaks (subtle)
    for &(x, y) in &[(8,15), (25,20), (45,13), (55,18)] {
        fill_rect(img, x, y, 6, 1, wind);
    }
}

// -------------------------------------------------------------------
// SHALLOWS — Skael's crystal caves
// -------------------------------------------------------------------

fn draw_shallows(img: &mut RgbaImage, time: &TimeOfDay) {
    let cave_bg   = night_tint(Rgba([40, 50, 55, 255]), time);
    let cave_lt   = night_tint(Rgba([55, 65, 70, 255]), time);
    let ceiling   = night_tint(Rgba([30, 35, 40, 255]), time);
    let crystal   = Rgba([80, 190, 160, 220]);       // teal glow
    let crystal_b = Rgba([60, 160, 140, 180]);        // dimmer crystal
    let water     = night_tint(Rgba([50, 90, 110, 255]), time);
    let water_lt  = Rgba([70, 120, 140, 200]);         // shimmer
    let mineral   = Rgba([130, 180, 100, 160]);        // glowing minerals
    let stalac    = night_tint(Rgba([65, 55, 50, 255]), time);
    let ground    = night_tint(Rgba([60, 55, 50, 255]), time);

    // Cave background (dark)
    fill_rect(img, 0, 0, 64, 64, cave_bg);

    // Cave ceiling (darker band at top)
    fill_rect(img, 0, 0, 64, 10, ceiling);
    // Stalactites
    for &(x, h) in &[(8,8), (18,5), (30,10), (42,6), (54,7)] {
        fill_rect(img, x, 0, 3, h, stalac);
        put(img, x + 1, h, stalac);
    }

    // Cave wall texture
    for &(x, y) in &[(5,15), (20,20), (40,18), (55,22), (10,25), (35,28), (50,14)] {
        fill_rect(img, x, y, 4, 3, cave_lt);
    }

    // Crystal formations (the star feature)
    for &(x, y, h) in &[(12,28,8), (38,25,10), (55,30,6)] {
        // Crystal shaft
        fill_rect(img, x, y, 2, h, crystal_b);
        fill_rect(img, x + 1, y, 1, h, crystal);
        // Crystal tip (bright)
        put(img, x, y - 1, crystal);
        put(img, x + 1, y - 1, crystal);
    }
    // Smaller crystals
    for &(x, y) in &[(22,32), (45,35), (8,38), (30,40)] {
        fill_rect(img, x, y, 1, 4, crystal_b);
        put(img, x, y - 1, crystal);
    }

    // Water pool at bottom
    fill_rect(img, 0, 48, 64, 16, water);
    // Water shimmer
    for &(x, y) in &[(8,50), (20,49), (35,51), (48,50), (58,49), (15,52), (42,51)] {
        fill_rect(img, x, y, 4, 1, water_lt);
    }

    // Glowing minerals on walls
    for &(x, y) in &[(3,20), (48,16), (25,24), (58,28), (15,35)] {
        put(img, x, y, mineral);
    }

    // Ground (cave floor above water)
    fill_rect(img, 0, 44, 64, 4, ground);
}

// -------------------------------------------------------------------
// DEPTHS — Nyxal's abyssal ocean
// -------------------------------------------------------------------

fn draw_depths(img: &mut RgbaImage, _time: &TimeOfDay) {
    // The deep ocean looks the same day or night — eternal darkness
    let deep_top  = Rgba([15, 20, 45, 255]);    // dark blue-purple
    let deep_mid  = Rgba([10, 12, 30, 255]);     // darker
    let deep_bot  = Rgba([5, 5, 15, 255]);       // near black
    let particle  = Rgba([30, 50, 80, 150]);      // floating debris
    let bio_dim   = Rgba([40, 120, 140, 120]);    // dim bioluminescence
    let bio_bright = Rgba([60, 180, 200, 180]);   // bright bioluminescence
    let orb       = Rgba([80, 200, 220, 200]);    // glowing orbs
    let pressure  = Rgba([20, 25, 50, 100]);      // pressure distortion lines

    // Deep ocean gradient (always dark)
    for y in 0..64 {
        let t = y as f32 / 64.0;
        let color = if t < 0.4 {
            px_blend(deep_top, deep_mid, t / 0.4)
        } else {
            px_blend(deep_mid, deep_bot, (t - 0.4) / 0.6)
        };
        fill_rect(img, 0, y as i32, 64, 1, color);
    }

    // Pressure distortion lines (horizontal streaks)
    for &(y, w) in &[(8,20), (18,30), (32,25), (45,35), (55,20)] {
        fill_rect(img, 10, y, w, 1, pressure);
    }

    // Floating particles (marine snow)
    for &(x, y) in &[(5,10), (15,22), (28,8), (40,30), (52,18), (8,42),
                       (35,50), (55,38), (20,55), (45,12), (12,35), (48,48)] {
        put(img, x, y, particle);
    }

    // Bioluminescent orbs (the defining feature)
    for &(x, y, bright) in &[(10,15,false), (35,8,true), (55,25,false),
                              (20,40,true), (45,50,false), (8,55,true)] {
        let color = if bright { orb } else { bio_dim };
        put(img, x, y, color);
        put(img, x + 1, y, color);
        put(img, x, y + 1, color);
        put(img, x + 1, y + 1, color);
        // Glow halo
        let halo = if bright { bio_bright } else { bio_dim };
        put(img, x - 1, y, halo);
        put(img, x + 2, y, halo);
        put(img, x, y - 1, halo);
        put(img, x + 1, y + 2, halo);
    }

    // Distant bioluminescent streaks (whale-like trails)
    for &(x, y, len) in &[(5, 28, 12), (40, 42, 15)] {
        for i in 0..len {
            put(img, x + i, y, bio_dim);
            if i % 3 == 0 { put(img, x + i, y + 1, bio_dim); }
        }
    }
}
