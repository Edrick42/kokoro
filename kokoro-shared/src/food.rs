//! Food types and nutritional profiles — shared between game and API.

use serde::{Deserialize, Serialize};
use super::species::Species;

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
    pub const ALL: &[FoodType] = &[
        FoodType::VerdanceBerry, FoodType::LatticeFruit, FoodType::ThermalSeed,
        FoodType::CaveCrustacean, FoodType::BiolumPlankton, FoodType::RootTuber,
        FoodType::SporeMoss, FoodType::CrystalWater,
    ];

    pub fn name(&self) -> &'static str {
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

    pub fn description(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry  => "Bioluminescent forest berry. Rich in carbs and vitamins.",
            FoodType::LatticeFruit   => "Golden canopy fruit. Balanced nutrition.",
            FoodType::ThermalSeed    => "Volcanic mineral seed. High protein and fat.",
            FoodType::CaveCrustacean => "Armored cave invertebrate. Extremely protein-dense.",
            FoodType::BiolumPlankton => "Microscopic luminous organisms. High water and fat.",
            FoodType::RootTuber      => "Underground starchy growth. High carbs and fiber.",
            FoodType::SporeMoss      => "Medicinal moss. Rich in vitamins and minerals.",
            FoodType::CrystalWater   => "Mineral-filtered water. Pure hydration.",
        }
    }

    pub fn biome(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry  => "The Verdance",
            FoodType::LatticeFruit   => "The Verdance",
            FoodType::ThermalSeed    => "Veridian Highlands",
            FoodType::CaveCrustacean => "Abyssal Shallows",
            FoodType::BiolumPlankton => "Abyssal Depths",
            FoodType::RootTuber      => "The Verdance",
            FoodType::SporeMoss      => "Multiple biomes",
            FoodType::CrystalWater   => "Multiple biomes",
        }
    }

    /// Preferred food for each species.
    pub fn is_preferred_by(&self, species: &Species) -> bool {
        matches!(
            (species, self),
            (Species::Moluun, FoodType::VerdanceBerry) |
            (Species::Pylum,  FoodType::ThermalSeed) |
            (Species::Skael,  FoodType::CaveCrustacean) |
            (Species::Nyxal,  FoodType::BiolumPlankton)
        )
    }
}
