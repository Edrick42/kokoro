//! Biome background with day/night cycle — retro palette.
//!
//! Combines two layers:
//! 1. **Species biome** — each species tints the background toward its palette color
//! 2. **Time of day** — morning is bright cream, sunset is warm orange,
//!    night is deep near-black, afternoon is the neutral base
//!
//! All colors derived from the 6-color retro palette with brightness shifts.
//!
//! ## Color flow
//!
//! ```text
//! Morning   → bright cream (CREAM lightened)
//! Afternoon → base cream + species tint
//! Sunset    → warm (CREAM → ORANGE blend)
//! Night     → dark (CREAM → NEAR_BLACK blend) + species tint
//! ```

use bevy::prelude::*;

use crate::game::state::{AppState, GameplayEntity};
use crate::config::ui::palette;
use crate::genome::{Genome, Species};
use crate::world::daycycle::{DayCycle, TimeOfDay};

#[derive(Component)]
struct BiomeBackground;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Gameplay), spawn_background)
           .add_systems(Update, update_background.run_if(in_state(AppState::Gameplay)));
    }
}

/// Species tint color from palette.
fn species_tint(species: &Species) -> Color {
    match species {
        Species::Moluun => palette::GOLD,
        Species::Pylum  => palette::ORANGE,
        Species::Skael  => palette::TEAL,
        Species::Nyxal  => palette::RED,
    }
}

/// Computes the background color from species + time of day.
///
/// The base is always CREAM, shifted by time of day (brightness)
/// and tinted by species (hue).
fn background_color(species: &Species, time: &TimeOfDay) -> Color {
    let tint = species_tint(species);

    // Time-of-day base: how light or dark the scene is
    let time_base = match time {
        // Morning: bright — cream lightened (blend toward white)
        TimeOfDay::Morning => blend(palette::CREAM, Color::srgb(0.95, 0.92, 0.88), 0.3),
        // Afternoon: neutral cream — the default
        TimeOfDay::Afternoon => palette::CREAM,
        // Sunset: warm — cream blended toward orange
        TimeOfDay::Sunset => blend(palette::CREAM, palette::ORANGE, 0.25),
        // Night: dark — cream blended heavily toward near-black
        TimeOfDay::Night => blend(palette::CREAM, palette::NEAR_BLACK, 0.65),
    };

    // Apply species tint on top of time base
    let tint_amount = match time {
        TimeOfDay::Night => 0.20, // stronger tint at night (species glow)
        _ => 0.12,                // subtle during day
    };

    blend(time_base, tint, tint_amount)
}

fn blend(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgb(
        a.red   + (b.red   - a.red)   * t,
        a.green + (b.green - a.green) * t,
        a.blue  + (b.blue  - a.blue)  * t,
    )
}

fn spawn_background(
    mut commands: Commands,
    genome: Res<Genome>,
    cycle: Res<DayCycle>,
) {
    let color = background_color(&genome.species, &cycle.time_of_day);

    // Also set ClearColor so window edges match
    commands.insert_resource(ClearColor(color));

    commands.spawn((
        GameplayEntity,
        BiomeBackground,
        Sprite {
            color,
            custom_size: Some(Vec2::new(800.0, 1400.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
}

/// Updates background when species or time of day changes.
fn update_background(
    genome: Res<Genome>,
    cycle: Res<DayCycle>,
    mut clear: ResMut<ClearColor>,
    mut bg_q: Query<&mut Sprite, With<BiomeBackground>>,
) {
    if !genome.is_changed() && !cycle.is_changed() { return; }

    let color = background_color(&genome.species, &cycle.time_of_day);
    clear.0 = color;

    for mut sprite in bg_q.iter_mut() {
        sprite.color = color;
    }
}
