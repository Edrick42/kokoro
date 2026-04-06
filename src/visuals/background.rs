//! Biome background — each species has a distinct environment behind it.
//!
//! Procedural gradient backgrounds that change when the player switches species.
//! Colors reflect each biome's atmosphere from the lore:
//! - Verdance (Moluun): deep greens, bioluminescent spore glow
//! - Highlands (Pylum): warm amber sky, rocky mesas
//! - Shallows (Skael): dark teal caves, crystal shimmer
//! - Depths (Nyxal): deep indigo ocean, bioluminescent specks

use bevy::prelude::*;

use crate::genome::{Genome, Species};

/// Marker for the background entity.
#[derive(Component)]
struct BiomeBackground;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_background)
           .add_systems(Update, update_background);
    }
}

/// Returns the background color for a species' biome.
fn biome_color(species: &Species) -> Color {
    match species {
        // Verdance: deep forest, bioluminescent
        Species::Moluun => Color::srgb(0.06, 0.12, 0.08),
        // Highlands: warm amber sky at dusk
        Species::Pylum  => Color::srgb(0.14, 0.10, 0.06),
        // Shallows: dark teal cave
        Species::Skael  => Color::srgb(0.05, 0.10, 0.12),
        // Depths: deep indigo ocean
        Species::Nyxal  => Color::srgb(0.04, 0.04, 0.10),
    }
}

fn spawn_background(mut commands: Commands, genome: Res<Genome>) {
    commands.spawn((
        BiomeBackground,
        Sprite {
            color: biome_color(&genome.species),
            custom_size: Some(Vec2::new(800.0, 1400.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
}

/// Updates background color when species changes.
fn update_background(
    genome: Res<Genome>,
    mut bg_q: Query<&mut Sprite, With<BiomeBackground>>,
) {
    if !genome.is_changed() { return; }

    let target = biome_color(&genome.species);
    for mut sprite in bg_q.iter_mut() {
        sprite.color = target;
    }
}
