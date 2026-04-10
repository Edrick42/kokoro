//! Nutrition system configuration — nutrients, food items, species decay rates.
//!
//! Every food is composed of fundamental nutrients (just like real biology).
//! Each species burns nutrients at different rates based on their biology.

use bevy::prelude::Color;
use serde::{Deserialize, Serialize};

// ===================================================================
// NUTRIENT PROFILE — what a food provides
// ===================================================================

/// How much of each nutrient a food item provides (0.0–50.0 range).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NutrientProfile {
    pub protein: f32,
    pub carbs: f32,
    pub fat: f32,
    pub water: f32,
    pub minerals: f32,
    pub vitamins: f32,
    pub fiber: f32,
}

impl NutrientProfile {
    pub const ZERO: Self = Self {
        protein: 0.0, carbs: 0.0, fat: 0.0, water: 0.0,
        minerals: 0.0, vitamins: 0.0, fiber: 0.0,
    };
}

// ===================================================================
// FOOD ITEMS — each is a composition of nutrients
// ===================================================================

/// A food item the player can give to a creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodType {
    VerdanceBerry,
    LatticeFruit,
    ThermalSeed,
    CaveCrustacean,
    BiolumPlankton,
    RootTuber,
    SporeMoss,
    CrystalWater,
}

impl FoodType {
    /// All available food types (for UI iteration).
    pub const ALL: &[FoodType] = &[
        FoodType::VerdanceBerry,
        FoodType::LatticeFruit,
        FoodType::ThermalSeed,
        FoodType::CaveCrustacean,
        FoodType::BiolumPlankton,
        FoodType::RootTuber,
        FoodType::SporeMoss,
        FoodType::CrystalWater,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry  => "Berry",
            FoodType::LatticeFruit   => "Fruit",
            FoodType::ThermalSeed    => "Seed",
            FoodType::CaveCrustacean => "Crust.",
            FoodType::BiolumPlankton => "Plnktn",
            FoodType::RootTuber      => "Tuber",
            FoodType::SporeMoss      => "Moss",
            FoodType::CrystalWater   => "Water",
        }
    }

    pub fn full_name(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry  => "Verdance Berry",
            FoodType::LatticeFruit   => "Lattice Fruit",
            FoodType::ThermalSeed    => "Thermal Seed",
            FoodType::CaveCrustacean => "Cave Crustacean",
            FoodType::BiolumPlankton => "Biolum Plankton",
            FoodType::RootTuber      => "Root Tuber",
            FoodType::SporeMoss      => "Spore Moss",
            FoodType::CrystalWater   => "Crystal Water",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            FoodType::VerdanceBerry  => Color::srgb(0.40, 0.80, 0.35),
            FoodType::LatticeFruit   => Color::srgb(0.90, 0.70, 0.30),
            FoodType::ThermalSeed    => Color::srgb(0.85, 0.75, 0.40),
            FoodType::CaveCrustacean => Color::srgb(0.85, 0.35, 0.30),
            FoodType::BiolumPlankton => Color::srgb(0.30, 0.75, 0.80),
            FoodType::RootTuber      => Color::srgb(0.65, 0.50, 0.35),
            FoodType::SporeMoss      => Color::srgb(0.45, 0.70, 0.50),
            FoodType::CrystalWater   => Color::srgb(0.50, 0.70, 0.95),
        }
    }

    pub fn nutrients(&self) -> NutrientProfile {
        match self {
            FoodType::VerdanceBerry => NutrientProfile {
                protein: 2.0, carbs: 25.0, fat: 3.0, water: 15.0,
                minerals: 5.0, vitamins: 20.0, fiber: 10.0,
            },
            FoodType::LatticeFruit => NutrientProfile {
                protein: 5.0, carbs: 30.0, fat: 8.0, water: 20.0,
                minerals: 3.0, vitamins: 15.0, fiber: 12.0,
            },
            FoodType::ThermalSeed => NutrientProfile {
                protein: 15.0, carbs: 20.0, fat: 12.0, water: 5.0,
                minerals: 10.0, vitamins: 8.0, fiber: 8.0,
            },
            FoodType::CaveCrustacean => NutrientProfile {
                protein: 35.0, carbs: 5.0, fat: 10.0, water: 10.0,
                minerals: 20.0, vitamins: 5.0, fiber: 0.0,
            },
            FoodType::BiolumPlankton => NutrientProfile {
                protein: 8.0, carbs: 10.0, fat: 15.0, water: 30.0,
                minerals: 5.0, vitamins: 10.0, fiber: 3.0,
            },
            FoodType::RootTuber => NutrientProfile {
                protein: 5.0, carbs: 35.0, fat: 2.0, water: 15.0,
                minerals: 8.0, vitamins: 5.0, fiber: 20.0,
            },
            FoodType::SporeMoss => NutrientProfile {
                protein: 3.0, carbs: 8.0, fat: 2.0, water: 25.0,
                minerals: 15.0, vitamins: 25.0, fiber: 15.0,
            },
            FoodType::CrystalWater => NutrientProfile {
                protein: 0.0, carbs: 0.0, fat: 0.0, water: 50.0,
                minerals: 10.0, vitamins: 0.0, fiber: 0.0,
            },
        }
    }

    /// Short description of the food for UI tooltips.
    pub fn description(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry  => "Sweet forest berry. Rich in vitamins and carbs.",
            FoodType::LatticeFruit   => "Canopy fruit with balanced nutrients.",
            FoodType::ThermalSeed    => "Heat-resistant seed. High in protein and fat.",
            FoodType::CaveCrustacean => "Cave-dwelling shellfish. Pure protein and minerals.",
            FoodType::BiolumPlankton => "Glowing deep-sea plankton. Rich in fat and water.",
            FoodType::RootTuber      => "Underground tuber. Heavy in carbs and fiber.",
            FoodType::SporeMoss      => "Cave moss with spores. Packed with vitamins and minerals.",
            FoodType::CrystalWater   => "Pure mineral water. Maximum hydration.",
        }
    }

    /// Event log key for this food type.
    pub fn event_key(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry  => "verdance_berry",
            FoodType::LatticeFruit   => "lattice_fruit",
            FoodType::ThermalSeed    => "thermal_seed",
            FoodType::CaveCrustacean => "cave_crustacean",
            FoodType::BiolumPlankton => "biolum_plankton",
            FoodType::RootTuber      => "root_tuber",
            FoodType::SporeMoss      => "spore_moss",
            FoodType::CrystalWater   => "crystal_water",
        }
    }
}

// ===================================================================
// SPECIES DECAY RATES — how fast each species burns each nutrient
// ===================================================================

/// Nutrient decay rates per tick for each species.
/// Higher = burns faster = needs more of that nutrient.
pub fn species_decay(species: &crate::genome::Species) -> NutrientProfile {
    use crate::genome::Species;
    match species {
        Species::Moluun => NutrientProfile {
            protein: 0.02, carbs: 0.03, fat: 0.02, water: 0.03,
            minerals: 0.01, vitamins: 0.02, fiber: 0.01,
        },
        Species::Pylum => NutrientProfile {
            protein: 0.03, carbs: 0.05, fat: 0.04, water: 0.03,
            minerals: 0.01, vitamins: 0.02, fiber: 0.01,
        },
        Species::Skael => NutrientProfile {
            protein: 0.05, carbs: 0.02, fat: 0.02, water: 0.02,
            minerals: 0.03, vitamins: 0.01, fiber: 0.01,
        },
        Species::Nyxal => NutrientProfile {
            protein: 0.01, carbs: 0.02, fat: 0.03, water: 0.01,
            minerals: 0.01, vitamins: 0.02, fiber: 0.01,
        },
    }
}

// ===================================================================
// DEFICIENCY THRESHOLDS
// ===================================================================

/// Below this level, the nutrient is considered deficient.
pub const DEFICIENCY_THRESHOLD: f32 = 20.0;

/// Extra happiness boost when feeding preferred food (species-matching biome food).
pub const PREFERRED_FOOD_HAPPINESS: f32 = 8.0;

/// Base happiness from any feeding.
pub const BASE_FEED_HAPPINESS: f32 = 3.0;
