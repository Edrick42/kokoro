//! Egg stage — the beginning of every Kobara's life.
//!
//! Before a creature is born, it exists as an egg (or species-appropriate form):
//! - **Moluun**: warm, fur-wrapped egg (mammalian nest)
//! - **Pylum**: speckled hard-shell bird egg
//! - **Skael**: crystalline reptile egg with faint glow
//! - **Nyxal**: translucent gelatinous roe cluster
//!
//! The player incubates the egg by interacting with it. Natural progress is
//! slow (~120 ticks / 2 minutes), but tapping speeds it up significantly.
//! At 100% incubation, the egg hatches and a Baby creature is born.

use bevy::prelude::*;

use crate::genome::Species;

use crate::config;

/// Tracks the incubation state of an unhatched creature.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct EggData {
    /// 0.0 = just laid, 1.0 = ready to hatch
    pub progress: f32,
    /// Whether this creature has hatched (born).
    pub hatched: bool,
}

use serde::{Serialize, Deserialize};

/// Marker component for the egg entity in the world.
#[derive(Component)]
pub struct EggEntity;

pub struct EggPlugin;

impl Plugin for EggPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EggTapEvent>()
           .add_systems(Update, (natural_incubation, handle_egg_tap));
    }
}

/// Advances incubation naturally each tick.
fn natural_incubation(
    mut collection: ResMut<crate::creature::collection::CreatureCollection>,
) {
    let idx = collection.active_index;
    let Some(creature) = collection.creatures.get_mut(idx) else { return };

    if creature.egg.hatched {
        return;
    }

    let rate = 1.0 / config::egg::NATURAL_INCUBATION_TICKS;
    creature.egg.progress = (creature.egg.progress + rate).min(1.0);

    if creature.egg.progress >= 1.0 {
        creature.egg.hatched = true;
        info!("Egg hatched! A new {:?} is born!", creature.genome.species);
        collection.visuals_dirty = true;
    }
}

/// When the player taps the egg (via the "Warm" action), boost incubation.
fn handle_egg_tap(
    mut events: EventReader<EggTapEvent>,
    mut collection: ResMut<crate::creature::collection::CreatureCollection>,
) {
    for _ in events.read() {
        let idx = collection.active_index;
        let Some(creature) = collection.creatures.get_mut(idx) else { continue };

        if creature.egg.hatched {
            continue;
        }

        creature.egg.progress = (creature.egg.progress + config::egg::TAP_BOOST).min(1.0);

        if creature.egg.progress >= 1.0 {
            creature.egg.hatched = true;
            info!("Egg hatched from player interaction!");
            collection.visuals_dirty = true;
        }
    }
}

/// Event fired when the player taps/warms the egg.
#[derive(Event)]
pub struct EggTapEvent;

/// Returns the display name for the egg form of a species.
#[allow(dead_code)]
pub fn egg_name(species: &Species) -> &'static str {
    match species {
        Species::Moluun => "Fur Egg",
        Species::Pylum  => "Speckled Egg",
        Species::Skael  => "Crystal Egg",
        Species::Nyxal  => "Roe Cluster",
    }
}

/// Returns the color for the egg of a species.
pub fn egg_color(species: &Species) -> Color {
    match species {
        Species::Moluun => Color::srgb(0.85, 0.75, 0.65),  // warm brown
        Species::Pylum  => Color::srgb(0.90, 0.85, 0.70),  // speckled cream
        Species::Skael  => Color::srgb(0.50, 0.75, 0.55),  // crystalline green
        Species::Nyxal  => Color::srgb(0.45, 0.30, 0.65),  // translucent purple
    }
}
