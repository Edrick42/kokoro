//! Fat reserve system — energy storage, insulation, and body composition.
//!
//! Fat acts as a buffer between nutrition and survival:
//! - Well-fed creatures build fat reserves (store_rate per tick)
//! - When hungry, fat is burned before muscles atrophy (burn_rate per tick)
//! - Fat level affects body visual (rounder when high, thinner when low)
//! - Insulation protects against temperature changes
//!
//! Species differ in baseline fat:
//! - Moluun: moderate (forest foraging is inconsistent)
//! - Pylum: lean (weight impacts flight efficiency)
//! - Skael: high (insulates against cave cold)
//! - Nyxal: moderate (distributed for neutral buoyancy)

use serde::{Deserialize, Serialize};

/// Fat reserve state for one creature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatReserve {
    /// Current fat level (0.0 = emaciated, 1.0 = maximum reserve).
    pub level: f32,
    /// Rate at which fat is consumed when hungry (per tick).
    pub burn_rate: f32,
    /// Rate at which fat is stored when well-fed (per tick).
    pub store_rate: f32,
    /// Insulation factor — higher fat = better temperature regulation.
    pub insulation: f32,
}

#[allow(dead_code)]
impl FatReserve {
    /// Is the creature dangerously low on fat?
    pub fn is_emaciated(&self) -> bool {
        self.level < 0.1
    }

    /// Is the creature well-fed (above average reserves)?
    pub fn is_well_fed(&self) -> bool {
        self.level > 0.6
    }

    /// Update insulation based on current fat level.
    pub fn update_insulation(&mut self, base_factor: f32) {
        self.insulation = self.level * base_factor;
    }
}
