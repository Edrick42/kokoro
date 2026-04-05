//! # Genome
//!
//! Every Kobara is born with a set of genes — f32 values between 0.0 and 1.0.
//! "Kobara" is the universal term for all creatures in this world (a fusion of
//! kokoro 心 + hara 腹 — "where the spirit lives"). Species determines the
//! creature's body type and visual template.
//!
//! The species defines the possible gene ranges; the individual fills them in.
//! This guarantees each creature is unique, just like in real biology.

mod color;
mod crossover;
mod species;

pub use species::Species;

use bevy::prelude::Resource;
use rand::Rng;
use serde::{Deserialize, Serialize};

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
            Species::Moluun => (0.2..=1.0, 0.1..=0.8, 0.2..=1.0),
            Species::Pylum  => (0.4..=1.0, 0.1..=0.5, 0.3..=0.9), // curious, light eaters
            Species::Skael  => (0.1..=0.7, 0.3..=1.0, 0.5..=1.0), // calm, hungry, tough
            Species::Nyxal  => (0.5..=1.0, 0.2..=0.7, 0.1..=0.6), // intelligent, fragile
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

    /// Generates a random genome for a Moluun Kobara (default species).
    pub fn random() -> Self {
        Self::random_for(Species::Moluun)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_moluun_within_ranges() {
        for _ in 0..100 {
            let g = Genome::random_for(Species::Moluun);
            assert!(g.curiosity >= 0.2 && g.curiosity <= 1.0);
            assert!(g.appetite >= 0.1 && g.appetite <= 0.8);
            assert!(g.resilience >= 0.2 && g.resilience <= 1.0);
            assert!(g.hue >= 0.0 && g.hue <= 360.0);
        }
    }

    #[test]
    fn random_nyxal_within_ranges() {
        for _ in 0..100 {
            let g = Genome::random_for(Species::Nyxal);
            assert!(g.curiosity >= 0.5 && g.curiosity <= 1.0, "curiosity={}", g.curiosity);
            assert!(g.appetite >= 0.2 && g.appetite <= 0.7, "appetite={}", g.appetite);
            assert!(g.resilience >= 0.1 && g.resilience <= 0.6, "resilience={}", g.resilience);
        }
    }

    #[test]
    fn all_species_produce_valid_genomes() {
        let species = [Species::Moluun, Species::Pylum, Species::Skael, Species::Nyxal];
        for sp in &species {
            for _ in 0..50 {
                let g = Genome::random_for(sp.clone());
                assert!(g.loneliness_sensitivity >= 0.1 && g.loneliness_sensitivity <= 0.9);
                assert!(g.circadian >= 0.0 && g.circadian <= 1.0);
                assert!(g.learning_rate >= 0.1 && g.learning_rate <= 0.6);
            }
        }
    }
}
