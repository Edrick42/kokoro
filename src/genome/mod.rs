//! # Genome
//!
//! Every Kobara is born with a set of genes — f32 values between 0.0 and 1.0.
//! "Kobara" is the universal term for all creatures in this world (a fusion of
//! kokoro 心 + hara 腹 — "where the spirit lives"). Species determines the
//! creature's body type and visual template.
//!
//! The species defines the possible gene ranges; the individual fills them in.
//! This guarantees each creature is unique, just like in real biology.

use bevy::prelude::Resource;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Available species of Kobara. Each has different body shapes, rigs,
/// and gene ranges.
///
/// All creatures are Kobaras — species determines their physical form:
/// - **Marumi** (丸み, "roundness") — soft, round mammal-like Kobaras
/// - **Tsubasa** (翼, "wing") — bird-like Kobaras
/// - **Uroko** (鱗, "scale") — reptile-like Kobaras
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Species {
    /// Round, soft, mammal-like Kobara. The first and most common species.
    Marumi,
    /// Bird-like Kobara with wings and a beak. Lighter, more expressive.
    Tsubasa,
    /// Reptile-like Kobara with scales and a tail. Sturdy, sharp features.
    Uroko,
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
    /// Generates a random genome for the given species.
    pub fn random_for(species: Species) -> Self {
        let mut rng = rand::rng();

        // Gene ranges vary by species
        let (curiosity_range, appetite_range, resilience_range) = match &species {
            Species::Marumi  => (0.2..=1.0, 0.1..=0.8, 0.2..=1.0),
            Species::Tsubasa => (0.4..=1.0, 0.1..=0.5, 0.3..=0.9), // curious, light eaters
            Species::Uroko   => (0.1..=0.7, 0.3..=1.0, 0.5..=1.0), // calm, hungry, tough
        };

        Self {
            species,
            curiosity:              rng.random_range(curiosity_range),
            loneliness_sensitivity: rng.random_range(0.1..=0.9),
            appetite:               rng.random_range(appetite_range),
            circadian:              rng.random_range(0.0..=1.0),
            resilience:             rng.random_range(resilience_range),
            learning_rate:          rng.random_range(0.1..=0.6),
            hue:                    rng.random_range(0.0..360.0),
        }
    }

    /// Generates a random genome for a Marumi Kobara (default species).
    pub fn random() -> Self {
        Self::random_for(Species::Marumi)
    }

    /// Creates a child genome by crossing two parent genomes with mutation.
    pub fn crossover(parent_a: &Genome, parent_b: &Genome, child_species: Species) -> Self {
        use rand::Rng;
        let mut rng = rand::rng();

        fn pick(rng: &mut impl Rng, a: f32, b: f32) -> f32 {
            if rng.random_bool(0.5) { a } else { b }
        }

        fn mutate(rng: &mut impl Rng, val: f32, min: f32, max: f32) -> f32 {
            if rng.random_range(0.0f32..1.0) < 0.15 {
                let shift = rng.random_range(-0.1f32..0.1);
                (val + shift).clamp(min, max)
            } else {
                val
            }
        }

        let c = pick(&mut rng, parent_a.curiosity, parent_b.curiosity);
        let curiosity = mutate(&mut rng, c, 0.0, 1.0);
        let l = pick(&mut rng, parent_a.loneliness_sensitivity, parent_b.loneliness_sensitivity);
        let loneliness_sensitivity = mutate(&mut rng, l, 0.0, 1.0);
        let a = pick(&mut rng, parent_a.appetite, parent_b.appetite);
        let appetite = mutate(&mut rng, a, 0.0, 1.0);
        let ci = pick(&mut rng, parent_a.circadian, parent_b.circadian);
        let circadian = mutate(&mut rng, ci, 0.0, 1.0);
        let r = pick(&mut rng, parent_a.resilience, parent_b.resilience);
        let resilience = mutate(&mut rng, r, 0.0, 1.0);
        let lr = pick(&mut rng, parent_a.learning_rate, parent_b.learning_rate);
        let learning_rate = mutate(&mut rng, lr, 0.0, 1.0);
        let h = pick(&mut rng, parent_a.hue, parent_b.hue);
        let hue = mutate(&mut rng, h / 360.0, 0.0, 1.0) * 360.0;

        Self {
            species: child_species,
            curiosity,
            loneliness_sensitivity,
            appetite,
            circadian,
            resilience,
            learning_rate,
            hue,
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
