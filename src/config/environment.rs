//! Environment configuration — temperature, biome comfort ranges.

/// Species temperature comfort ranges (Celsius).
pub mod comfort {
    use crate::genome::Species;

    pub fn range(species: &Species) -> (f32, f32) {
        match species {
            Species::Moluun => (15.0, 28.0),   // temperate forest
            Species::Pylum  => (20.0, 35.0),   // warm highlands
            Species::Skael  => (25.0, 42.0),   // hot caves
            Species::Nyxal  => (2.0, 15.0),    // cold deep ocean
        }
    }
}

/// Temperature cycle amplitude (day/night variation).
pub const DAY_AMPLITUDE: f32 = 8.0;  // ±8°C from base

/// Base temperature per biome (midpoint of comfort range).
pub const BASE_TEMP: f32 = 22.0;

/// Energy drain per tick when outside comfort zone.
pub const DISCOMFORT_ENERGY_DRAIN: f32 = 0.02;

/// Happiness drain per tick when outside comfort zone.
pub const DISCOMFORT_HAPPINESS_DRAIN: f32 = 0.01;

/// Fat insulation reduces temperature discomfort by this factor per fat.level unit.
pub const FAT_INSULATION_FACTOR: f32 = 0.4;

/// Skin insulation per species covering type.
pub mod skin_insulation {
    pub const FUR: f32 = 0.5;       // Moluun: good insulation
    pub const PLUMAGE: f32 = 0.4;   // Pylum: decent
    pub const SCALES: f32 = 0.6;    // Skael: retains heat well
    pub const MEMBRANE: f32 = 0.1;  // Nyxal: almost none (deep ocean is stable)
}
