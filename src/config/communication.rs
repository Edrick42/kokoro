//! Communication system — multi-channel species profiles.
//!
//! Each species communicates through different channels with varying ability.
//! Sound is just one channel — visual, kinetic, chemical, and tactile
//! are equally important for different species.

use crate::genome::Species;

/// How capable a species is at each communication channel (0.0–1.0).
#[derive(Debug, Clone, Copy)]
pub struct CommunicationProfile {
    /// Sound production (vocalizations).
    pub vocal: f32,
    /// Color/pattern changes (chromatophores, bioluminescence).
    pub visual: f32,
    /// Body language, posture, movement.
    pub kinetic: f32,
    /// Scent, pheromones.
    pub chemical: f32,
    /// Physical touch, vibration.
    pub tactile: f32,
}

pub fn species_profile(species: &Species) -> CommunicationProfile {
    match species {
        // Moluun: body language masters (ears, posture)
        Species::Moluun => CommunicationProfile {
            vocal: 0.6, visual: 0.3, kinetic: 0.9, chemical: 0.4, tactile: 0.5,
        },
        // Pylum: vocal masters (songs, calls — syrinx organ)
        Species::Pylum => CommunicationProfile {
            vocal: 0.9, visual: 0.5, kinetic: 0.7, chemical: 0.2, tactile: 0.3,
        },
        // Skael: chemical masters (scent marking, territorial)
        Species::Skael => CommunicationProfile {
            vocal: 0.3, visual: 0.4, kinetic: 0.5, chemical: 0.8, tactile: 0.4,
        },
        // Nyxal: visual masters (chromatophores) + tactile (tentacles)
        Species::Nyxal => CommunicationProfile {
            vocal: 0.4, visual: 0.9, kinetic: 0.3, chemical: 0.3, tactile: 0.8,
        },
    }
}

/// Vocal repertoire limits by species.
pub mod vocal {
    /// Base number of sounds the species knows innately.
    pub fn base_sounds(species: &super::Species) -> usize {
        match species {
            super::Species::Moluun => 4,  // purr, whine, chirp, growl
            super::Species::Pylum  => 5,  // chirp, trill, screech, coo, whistle
            super::Species::Skael  => 3,  // hiss, rumble, click
            super::Species::Nyxal  => 3,  // pulse, drone, click
        }
    }

    /// Maximum total sounds (base + learned) the species can know.
    pub fn max_capacity(species: &super::Species) -> usize {
        match species {
            super::Species::Moluun => 8,
            super::Species::Pylum  => 15, // parrots learn the most
            super::Species::Skael  => 5,
            super::Species::Nyxal  => 10,
        }
    }

    /// Learning speed multiplier (higher = learns faster).
    pub fn learning_speed(species: &super::Species) -> f32 {
        match species {
            super::Species::Moluun => 0.5,
            super::Species::Pylum  => 1.0,  // fastest learner
            super::Species::Skael  => 0.2,  // slowest
            super::Species::Nyxal  => 0.6,
        }
    }
}
