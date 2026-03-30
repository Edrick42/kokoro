//! # Genome
//!
//! Every Kobara is born with a set of genes — f32 values between 0.0 and 1.0.
//! The species defines the possible ranges; the individual fills them in.
//! This guarantees each creature is unique, just like in real biology.

use bevy::prelude::Resource;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Available species. Each species has different gene ranges.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Species {
    Kobara,
    // Future species: Lumini, Verun, Drakel ...
}

/// The creature's DNA. Each field is a value between 0.0 and 1.0.
///
/// Examples of how genes manifest in behaviour:
/// - `curiosity = 0.9`            → always exploring, hard to calm down
/// - `appetite = 0.2`             → eats little, stays full for longer
/// - `circadian = 0.1`            → night owl, most active after dark
/// - `loneliness_sensitivity = 0.8` → suffers quickly when left alone
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub species: Species,

    /// Tendency to explore and interact with the environment (0 = apathetic, 1 = hyperactive)
    pub curiosity: f32,

    /// How much the creature suffers when alone (0 = independent, 1 = very dependent)
    pub loneliness_sensitivity: f32,

    /// Rate at which hunger increases (0 = slow metabolism, 1 = fast metabolism)
    pub appetite: f32,

    /// Peak activity time (0.0 = late night, 0.5 = midday, 1.0 = midnight)
    pub circadian: f32,

    /// Emotional resilience — how fast it recovers from negative states (0 = fragile, 1 = resilient)
    pub resilience: f32,

    /// Neural network learning speed (0 = slow learner, 1 = fast learner)
    pub learning_rate: f32,

    /// Base body color as HSL hue (0.0–360.0)
    pub hue: f32,
}

impl Genome {
    /// Generates a random genome for the Kobara species.
    pub fn random() -> Self {
        let mut rng = rand::rng();
        Self {
            species:                Species::Kobara,
            curiosity:              rng.random_range(0.2..=1.0),
            loneliness_sensitivity: rng.random_range(0.1..=0.9),
            appetite:               rng.random_range(0.1..=0.8),
            circadian:              rng.random_range(0.0..=1.0),
            resilience:             rng.random_range(0.2..=1.0),
            learning_rate:          rng.random_range(0.1..=0.6),
            hue:                    rng.random_range(0.0..360.0),
        }
    }

    /// Returns the body color derived from the `hue` gene.
    pub fn body_color(&self) -> bevy::color::Color {
        bevy::color::Color::hsl(self.hue, 0.7, 0.75)
    }

    /// Returns a tint color for sprite rendering.
    ///
    /// Unlike `body_color()` which is meant for procedural meshes, this returns
    /// a slightly lighter, more saturated color that looks good when multiplied
    /// onto a flat-colored pixel art sprite.
    pub fn tint_color(&self) -> bevy::color::Color {
        bevy::color::Color::hsl(self.hue, 0.65, 0.80)
    }
}
