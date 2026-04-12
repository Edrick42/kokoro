//! Food types — shared between game and API.
//!
//! 24 foods across 8 categories and 4 biomes. Each species prefers food
//! from their home biome.

use serde::{Deserialize, Serialize};
use super::species::Species;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodCategory {
    Water, Fruit, Meat, Herb, Tuber, Fungus, Seed, Algae,
}

impl FoodCategory {
    pub fn label(&self) -> &'static str {
        match self {
            FoodCategory::Water  => "Water",
            FoodCategory::Fruit  => "Fruit",
            FoodCategory::Meat   => "Meat",
            FoodCategory::Herb   => "Herb",
            FoodCategory::Tuber  => "Tuber",
            FoodCategory::Fungus => "Fungus",
            FoodCategory::Seed   => "Seed",
            FoodCategory::Algae  => "Algae",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodBiome {
    Verdance, Highlands, AbyssalShallow, AbyssalDeep,
}

impl FoodBiome {
    pub fn label(&self) -> &'static str {
        match self {
            FoodBiome::Verdance       => "The Verdance",
            FoodBiome::Highlands      => "Veridian Highlands",
            FoodBiome::AbyssalShallow => "Abyssal Shallows",
            FoodBiome::AbyssalDeep    => "Abyssal Depths",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodType {
    // Verdance
    VerdanceBerry, LatticeFruit, RootTuber, ForestMushroom, HoneySap, BarkGrub,
    // Highlands
    ThermalSeed, HighlandClover, WindDriedBerry, CliffEgg, MesaCactusWater, LichenCrust,
    // Abyssal Shallows
    CaveCrustacean, SporeMoss, CrystalWater, CaveRoot, StalactiteFungi, BlindFish,
    // Abyssal Depths
    BiolumPlankton, ThermalVentShrimp, AbyssKelp, PressurePearl, DepthJelly, CoralPolyp,
}

impl FoodType {
    pub const ALL: &[FoodType] = &[
        FoodType::VerdanceBerry, FoodType::LatticeFruit, FoodType::RootTuber,
        FoodType::ForestMushroom, FoodType::HoneySap, FoodType::BarkGrub,
        FoodType::ThermalSeed, FoodType::HighlandClover, FoodType::WindDriedBerry,
        FoodType::CliffEgg, FoodType::MesaCactusWater, FoodType::LichenCrust,
        FoodType::CaveCrustacean, FoodType::SporeMoss, FoodType::CrystalWater,
        FoodType::CaveRoot, FoodType::StalactiteFungi, FoodType::BlindFish,
        FoodType::BiolumPlankton, FoodType::ThermalVentShrimp, FoodType::AbyssKelp,
        FoodType::PressurePearl, FoodType::DepthJelly, FoodType::CoralPolyp,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry     => "Verdance Berry",
            FoodType::LatticeFruit      => "Lattice Fruit",
            FoodType::RootTuber         => "Root Tuber",
            FoodType::ForestMushroom    => "Forest Mushroom",
            FoodType::HoneySap          => "Honey Sap",
            FoodType::BarkGrub          => "Bark Grub",
            FoodType::ThermalSeed       => "Thermal Seed",
            FoodType::HighlandClover    => "Highland Clover",
            FoodType::WindDriedBerry    => "Wind-Dried Berry",
            FoodType::CliffEgg          => "Cliff Egg",
            FoodType::MesaCactusWater   => "Mesa Cactus Water",
            FoodType::LichenCrust       => "Lichen Crust",
            FoodType::CaveCrustacean    => "Cave Crustacean",
            FoodType::SporeMoss         => "Spore Moss",
            FoodType::CrystalWater      => "Crystal Water",
            FoodType::CaveRoot          => "Cave Root",
            FoodType::StalactiteFungi   => "Stalactite Fungi",
            FoodType::BlindFish         => "Blind Fish",
            FoodType::BiolumPlankton    => "Biolum Plankton",
            FoodType::ThermalVentShrimp => "Thermal Vent Shrimp",
            FoodType::AbyssKelp         => "Abyss Kelp",
            FoodType::PressurePearl     => "Pressure Pearl",
            FoodType::DepthJelly        => "Depth Jelly",
            FoodType::CoralPolyp        => "Coral Polyp",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry     => "Sweet forest berry. Rich in vitamins and carbs.",
            FoodType::LatticeFruit      => "Canopy fruit with balanced nutrients.",
            FoodType::RootTuber         => "Underground tuber. Heavy in carbs and fiber.",
            FoodType::ForestMushroom    => "Healing forest mushroom. Minerals and mild medicine.",
            FoodType::HoneySap          => "Sweet tree sap. Hydrating and mood-lifting.",
            FoodType::BarkGrub          => "Protein-rich grub from under bark.",
            FoodType::ThermalSeed       => "Heat-resistant seed. High in protein and fat.",
            FoodType::HighlandClover    => "Medicinal highland herb. Cures cold, boosts health.",
            FoodType::WindDriedBerry    => "Sun-dried highland berry. Concentrated carbs.",
            FoodType::CliffEgg          => "Rare cliff-nesting egg. Extremely protein-rich.",
            FoodType::MesaCactusWater   => "Cactus water from mesa. Maximum hydration.",
            FoodType::LichenCrust       => "Dried lichen. Minerals and slow-release energy.",
            FoodType::CaveCrustacean    => "Cave-dwelling shellfish. Pure protein and minerals.",
            FoodType::SporeMoss         => "Cave moss with spores. Packed with vitamins.",
            FoodType::CrystalWater      => "Pure mineral water from crystal caves.",
            FoodType::CaveRoot          => "Dense underground root. Tough fiber and heavy carbs.",
            FoodType::StalactiteFungi   => "Mineral-rich cave fungus. Strengthens bones.",
            FoodType::BlindFish         => "Eyeless cave fish. Oily, rich in protein and fat.",
            FoodType::BiolumPlankton    => "Glowing deep-sea plankton. Rich in fat and water.",
            FoodType::ThermalVentShrimp => "Vent-warmed shrimp. Protein-dense and warming.",
            FoodType::AbyssKelp         => "Deep kelp. Fiber, water, and vitamins.",
            FoodType::PressurePearl     => "Rare mineral pearl. Pure hydration and minerals.",
            FoodType::DepthJelly        => "Bioluminescent jelly. Mild healing and glow boost.",
            FoodType::CoralPolyp        => "Calcium-rich coral. Protein and bone repair.",
        }
    }

    pub fn category(&self) -> FoodCategory {
        match self {
            FoodType::HoneySap | FoodType::MesaCactusWater |
            FoodType::CrystalWater | FoodType::PressurePearl => FoodCategory::Water,
            FoodType::VerdanceBerry | FoodType::LatticeFruit |
            FoodType::WindDriedBerry => FoodCategory::Fruit,
            FoodType::BarkGrub | FoodType::CliffEgg | FoodType::CaveCrustacean |
            FoodType::BlindFish | FoodType::ThermalVentShrimp |
            FoodType::CoralPolyp => FoodCategory::Meat,
            FoodType::HighlandClover => FoodCategory::Herb,
            FoodType::RootTuber | FoodType::CaveRoot => FoodCategory::Tuber,
            FoodType::ForestMushroom | FoodType::LichenCrust | FoodType::SporeMoss |
            FoodType::StalactiteFungi | FoodType::DepthJelly => FoodCategory::Fungus,
            FoodType::ThermalSeed => FoodCategory::Seed,
            FoodType::BiolumPlankton | FoodType::AbyssKelp => FoodCategory::Algae,
        }
    }

    pub fn biome(&self) -> FoodBiome {
        match self {
            FoodType::VerdanceBerry | FoodType::LatticeFruit | FoodType::RootTuber |
            FoodType::ForestMushroom | FoodType::HoneySap | FoodType::BarkGrub
                => FoodBiome::Verdance,
            FoodType::ThermalSeed | FoodType::HighlandClover | FoodType::WindDriedBerry |
            FoodType::CliffEgg | FoodType::MesaCactusWater | FoodType::LichenCrust
                => FoodBiome::Highlands,
            FoodType::CaveCrustacean | FoodType::SporeMoss | FoodType::CrystalWater |
            FoodType::CaveRoot | FoodType::StalactiteFungi | FoodType::BlindFish
                => FoodBiome::AbyssalShallow,
            FoodType::BiolumPlankton | FoodType::ThermalVentShrimp | FoodType::AbyssKelp |
            FoodType::PressurePearl | FoodType::DepthJelly | FoodType::CoralPolyp
                => FoodBiome::AbyssalDeep,
        }
    }

    /// Is this food from the creature's home biome?
    pub fn is_native_for(&self, species: &Species) -> bool {
        match (species, self.biome()) {
            (Species::Moluun, FoodBiome::Verdance)       => true,
            (Species::Pylum,  FoodBiome::Highlands)      => true,
            (Species::Skael,  FoodBiome::AbyssalShallow) => true,
            (Species::Nyxal,  FoodBiome::AbyssalDeep)    => true,
            _ => false,
        }
    }

    /// Legacy compat: preferred food check.
    pub fn is_preferred_by(&self, species: &Species) -> bool {
        self.is_native_for(species)
    }
}
