//! Nutrition system configuration — food taxonomy, nutrients, species decay rates.
//!
//! Every food belongs to a **category** (fruit, meat, herb, etc.) and originates
//! from a **biome**. Creatures prefer food from their home biome and get bonus
//! happiness. Some foods have special effects (healing, bone repair, mood boost).

use bevy::prelude::Color;
use serde::{Deserialize, Serialize};

// ===================================================================
// NUTRIENT PROFILE
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
// FOOD CATEGORIES
// ===================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodCategory {
    Water,
    Fruit,
    Meat,
    Herb,
    Tuber,
    Fungus,
    Seed,
    Algae,
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

// ===================================================================
// BIOMES
// ===================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodBiome {
    Verdance,       // Moluun's forest
    Highlands,      // Pylum's mesas
    AbyssalShallow, // Skael's caves
    AbyssalDeep,    // Nyxal's deep ocean
    Universal,      // available everywhere
}

impl FoodBiome {
    pub fn label(&self) -> &'static str {
        match self {
            FoodBiome::Verdance       => "The Verdance",
            FoodBiome::Highlands      => "Veridian Highlands",
            FoodBiome::AbyssalShallow => "Abyssal Shallows",
            FoodBiome::AbyssalDeep    => "Abyssal Depths",
            FoodBiome::Universal      => "Universal",
        }
    }

    /// The species native to this biome.
    pub fn native_species(&self) -> Option<crate::genome::Species> {
        use crate::genome::Species;
        match self {
            FoodBiome::Verdance       => Some(Species::Moluun),
            FoodBiome::Highlands      => Some(Species::Pylum),
            FoodBiome::AbyssalShallow => Some(Species::Skael),
            FoodBiome::AbyssalDeep    => Some(Species::Nyxal),
            FoodBiome::Universal      => None,
        }
    }
}

// ===================================================================
// SPECIAL EFFECTS
// ===================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SpecialEffect {
    /// Reduces disease duration by N ticks.
    Healing(f32),
    /// Boosts mineral → skeleton repair rate temporarily.
    BoneStrength(f32),
    /// Extra happiness beyond normal feeding.
    MoodBoost(f32),
    /// Nyxal bioluminescence intensity boost (temporary).
    BiolumBoost(f32),
    /// Warms the creature (reduces cold discomfort).
    Warming(f32),
}

// ===================================================================
// FOOD ITEMS — 24 types across 4 biomes + universal
// ===================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodType {
    // === The Verdance (Moluun's forest) ===
    VerdanceBerry,    // fruit — vitamins + carbs
    LatticeFruit,     // fruit — balanced
    RootTuber,        // tuber — carbs + fiber
    ForestMushroom,   // fungus — minerals + mild healing
    HoneySap,         // water — sweet, happiness boost
    BarkGrub,         // meat — small protein snack

    // === Veridian Highlands (Pylum's mesas) ===
    ThermalSeed,      // seed — protein + fat
    HighlandClover,   // herb — healing, cures Cold
    WindDriedBerry,   // fruit — concentrated carbs
    CliffEgg,         // meat — high protein, rare
    MesaCactusWater,  // water — hydration + fiber
    LichenCrust,      // fungus — minerals, slow energy

    // === Abyssal Shallows (Skael's caves) ===
    CaveCrustacean,   // meat — pure protein + minerals
    SporeMoss,        // fungus — vitamins + minerals
    CrystalWater,     // water — pure hydration + minerals
    CaveRoot,         // tuber — dense carbs, tough fiber
    StalactiteFungi,  // fungus — bone-strengthening
    BlindFish,        // meat — protein + fat, oily

    // === Abyssal Depths (Nyxal's deep ocean) ===
    BiolumPlankton,    // algae — water + fat
    ThermalVentShrimp, // meat — protein + minerals, warming
    AbyssKelp,         // algae — fiber + water + vitamins
    PressurePearl,     // water — pure hydration + minerals, rare
    DepthJelly,        // fungus — healing + biolum boost
    CoralPolyp,        // meat — protein + calcium (bone repair)
}

impl FoodType {
    pub const ALL: &[FoodType] = &[
        // Verdance
        FoodType::VerdanceBerry, FoodType::LatticeFruit, FoodType::RootTuber,
        FoodType::ForestMushroom, FoodType::HoneySap, FoodType::BarkGrub,
        // Highlands
        FoodType::ThermalSeed, FoodType::HighlandClover, FoodType::WindDriedBerry,
        FoodType::CliffEgg, FoodType::MesaCactusWater, FoodType::LichenCrust,
        // Abyssal Shallows
        FoodType::CaveCrustacean, FoodType::SporeMoss, FoodType::CrystalWater,
        FoodType::CaveRoot, FoodType::StalactiteFungi, FoodType::BlindFish,
        // Abyssal Depths
        FoodType::BiolumPlankton, FoodType::ThermalVentShrimp, FoodType::AbyssKelp,
        FoodType::PressurePearl, FoodType::DepthJelly, FoodType::CoralPolyp,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry     => "Berry",
            FoodType::LatticeFruit      => "Fruit",
            FoodType::RootTuber         => "Tuber",
            FoodType::ForestMushroom    => "Mush.",
            FoodType::HoneySap          => "Honey",
            FoodType::BarkGrub          => "Grub",
            FoodType::ThermalSeed       => "Seed",
            FoodType::HighlandClover    => "Clover",
            FoodType::WindDriedBerry    => "W.Berry",
            FoodType::CliffEgg          => "Egg",
            FoodType::MesaCactusWater   => "Cactus",
            FoodType::LichenCrust       => "Lichen",
            FoodType::CaveCrustacean    => "Crust.",
            FoodType::SporeMoss         => "Moss",
            FoodType::CrystalWater      => "Water",
            FoodType::CaveRoot          => "C.Root",
            FoodType::StalactiteFungi   => "Stalac.",
            FoodType::BlindFish         => "Fish",
            FoodType::BiolumPlankton    => "Plnktn",
            FoodType::ThermalVentShrimp => "Shrimp",
            FoodType::AbyssKelp         => "Kelp",
            FoodType::PressurePearl     => "Pearl",
            FoodType::DepthJelly        => "Jelly",
            FoodType::CoralPolyp        => "Coral",
        }
    }

    pub fn full_name(&self) -> &'static str {
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

    pub fn special_effect(&self) -> Option<SpecialEffect> {
        match self {
            FoodType::ForestMushroom    => Some(SpecialEffect::Healing(5.0)),
            FoodType::HoneySap          => Some(SpecialEffect::MoodBoost(8.0)),
            FoodType::HighlandClover    => Some(SpecialEffect::Healing(10.0)),
            FoodType::WindDriedBerry    => Some(SpecialEffect::MoodBoost(5.0)),
            FoodType::StalactiteFungi   => Some(SpecialEffect::BoneStrength(0.05)),
            FoodType::ThermalVentShrimp => Some(SpecialEffect::Warming(15.0)),
            FoodType::DepthJelly        => Some(SpecialEffect::BiolumBoost(0.3)),
            FoodType::CoralPolyp        => Some(SpecialEffect::BoneStrength(0.08)),
            _ => None,
        }
    }

    pub fn color(&self) -> Color {
        match self.category() {
            FoodCategory::Water  => Color::srgb(0.50, 0.70, 0.95),
            FoodCategory::Fruit  => Color::srgb(0.90, 0.70, 0.30),
            FoodCategory::Meat   => Color::srgb(0.85, 0.35, 0.30),
            FoodCategory::Herb   => Color::srgb(0.35, 0.80, 0.40),
            FoodCategory::Tuber  => Color::srgb(0.65, 0.50, 0.35),
            FoodCategory::Fungus => Color::srgb(0.55, 0.45, 0.60),
            FoodCategory::Seed   => Color::srgb(0.85, 0.75, 0.40),
            FoodCategory::Algae  => Color::srgb(0.30, 0.75, 0.70),
        }
    }

    pub fn nutrients(&self) -> NutrientProfile {
        match self {
            // === Verdance ===
            FoodType::VerdanceBerry => NutrientProfile {
                protein: 2.0, carbs: 25.0, fat: 3.0, water: 15.0,
                minerals: 5.0, vitamins: 20.0, fiber: 10.0,
            },
            FoodType::LatticeFruit => NutrientProfile {
                protein: 5.0, carbs: 30.0, fat: 8.0, water: 20.0,
                minerals: 3.0, vitamins: 15.0, fiber: 12.0,
            },
            FoodType::RootTuber => NutrientProfile {
                protein: 5.0, carbs: 35.0, fat: 2.0, water: 15.0,
                minerals: 8.0, vitamins: 5.0, fiber: 20.0,
            },
            FoodType::ForestMushroom => NutrientProfile {
                protein: 4.0, carbs: 8.0, fat: 1.0, water: 20.0,
                minerals: 18.0, vitamins: 12.0, fiber: 8.0,
            },
            FoodType::HoneySap => NutrientProfile {
                protein: 0.0, carbs: 15.0, fat: 0.0, water: 40.0,
                minerals: 5.0, vitamins: 3.0, fiber: 0.0,
            },
            FoodType::BarkGrub => NutrientProfile {
                protein: 20.0, carbs: 3.0, fat: 8.0, water: 5.0,
                minerals: 4.0, vitamins: 2.0, fiber: 0.0,
            },

            // === Highlands ===
            FoodType::ThermalSeed => NutrientProfile {
                protein: 15.0, carbs: 20.0, fat: 12.0, water: 5.0,
                minerals: 10.0, vitamins: 8.0, fiber: 8.0,
            },
            FoodType::HighlandClover => NutrientProfile {
                protein: 2.0, carbs: 5.0, fat: 1.0, water: 15.0,
                minerals: 8.0, vitamins: 30.0, fiber: 5.0,
            },
            FoodType::WindDriedBerry => NutrientProfile {
                protein: 3.0, carbs: 35.0, fat: 2.0, water: 5.0,
                minerals: 4.0, vitamins: 15.0, fiber: 8.0,
            },
            FoodType::CliffEgg => NutrientProfile {
                protein: 30.0, carbs: 2.0, fat: 15.0, water: 10.0,
                minerals: 8.0, vitamins: 10.0, fiber: 0.0,
            },
            FoodType::MesaCactusWater => NutrientProfile {
                protein: 0.0, carbs: 5.0, fat: 0.0, water: 45.0,
                minerals: 3.0, vitamins: 2.0, fiber: 5.0,
            },
            FoodType::LichenCrust => NutrientProfile {
                protein: 3.0, carbs: 12.0, fat: 1.0, water: 8.0,
                minerals: 15.0, vitamins: 8.0, fiber: 10.0,
            },

            // === Abyssal Shallows ===
            FoodType::CaveCrustacean => NutrientProfile {
                protein: 35.0, carbs: 5.0, fat: 10.0, water: 10.0,
                minerals: 20.0, vitamins: 5.0, fiber: 0.0,
            },
            FoodType::SporeMoss => NutrientProfile {
                protein: 3.0, carbs: 8.0, fat: 2.0, water: 25.0,
                minerals: 15.0, vitamins: 25.0, fiber: 15.0,
            },
            FoodType::CrystalWater => NutrientProfile {
                protein: 0.0, carbs: 0.0, fat: 0.0, water: 50.0,
                minerals: 10.0, vitamins: 0.0, fiber: 0.0,
            },
            FoodType::CaveRoot => NutrientProfile {
                protein: 4.0, carbs: 40.0, fat: 1.0, water: 10.0,
                minerals: 10.0, vitamins: 3.0, fiber: 25.0,
            },
            FoodType::StalactiteFungi => NutrientProfile {
                protein: 5.0, carbs: 6.0, fat: 2.0, water: 12.0,
                minerals: 30.0, vitamins: 8.0, fiber: 5.0,
            },
            FoodType::BlindFish => NutrientProfile {
                protein: 25.0, carbs: 0.0, fat: 18.0, water: 12.0,
                minerals: 8.0, vitamins: 5.0, fiber: 0.0,
            },

            // === Abyssal Depths ===
            FoodType::BiolumPlankton => NutrientProfile {
                protein: 8.0, carbs: 10.0, fat: 15.0, water: 30.0,
                minerals: 5.0, vitamins: 10.0, fiber: 3.0,
            },
            FoodType::ThermalVentShrimp => NutrientProfile {
                protein: 28.0, carbs: 3.0, fat: 8.0, water: 8.0,
                minerals: 18.0, vitamins: 5.0, fiber: 0.0,
            },
            FoodType::AbyssKelp => NutrientProfile {
                protein: 3.0, carbs: 10.0, fat: 1.0, water: 35.0,
                minerals: 8.0, vitamins: 18.0, fiber: 15.0,
            },
            FoodType::PressurePearl => NutrientProfile {
                protein: 0.0, carbs: 0.0, fat: 0.0, water: 45.0,
                minerals: 20.0, vitamins: 0.0, fiber: 0.0,
            },
            FoodType::DepthJelly => NutrientProfile {
                protein: 5.0, carbs: 8.0, fat: 3.0, water: 20.0,
                minerals: 12.0, vitamins: 15.0, fiber: 2.0,
            },
            FoodType::CoralPolyp => NutrientProfile {
                protein: 22.0, carbs: 2.0, fat: 5.0, water: 10.0,
                minerals: 25.0, vitamins: 3.0, fiber: 0.0,
            },
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry     => "Sweet forest berry. Rich in vitamins and carbs.",
            FoodType::LatticeFruit      => "Canopy fruit with balanced nutrients.",
            FoodType::RootTuber         => "Underground tuber. Heavy in carbs and fiber.",
            FoodType::ForestMushroom    => "Healing forest mushroom. Minerals and mild medicine.",
            FoodType::HoneySap          => "Sweet tree sap. Hydrating and mood-lifting.",
            FoodType::BarkGrub          => "Protein-rich grub from under bark. Small but nutritious.",
            FoodType::ThermalSeed       => "Heat-resistant seed. High in protein and fat.",
            FoodType::HighlandClover    => "Medicinal highland herb. Cures cold and boosts health.",
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

    pub fn event_key(&self) -> &'static str {
        match self {
            FoodType::VerdanceBerry     => "verdance_berry",
            FoodType::LatticeFruit      => "lattice_fruit",
            FoodType::RootTuber         => "root_tuber",
            FoodType::ForestMushroom    => "forest_mushroom",
            FoodType::HoneySap          => "honey_sap",
            FoodType::BarkGrub          => "bark_grub",
            FoodType::ThermalSeed       => "thermal_seed",
            FoodType::HighlandClover    => "highland_clover",
            FoodType::WindDriedBerry    => "wind_dried_berry",
            FoodType::CliffEgg          => "cliff_egg",
            FoodType::MesaCactusWater   => "mesa_cactus_water",
            FoodType::LichenCrust       => "lichen_crust",
            FoodType::CaveCrustacean    => "cave_crustacean",
            FoodType::SporeMoss         => "spore_moss",
            FoodType::CrystalWater      => "crystal_water",
            FoodType::CaveRoot          => "cave_root",
            FoodType::StalactiteFungi   => "stalactite_fungi",
            FoodType::BlindFish         => "blind_fish",
            FoodType::BiolumPlankton    => "biolum_plankton",
            FoodType::ThermalVentShrimp => "thermal_vent_shrimp",
            FoodType::AbyssKelp         => "abyss_kelp",
            FoodType::PressurePearl     => "pressure_pearl",
            FoodType::DepthJelly        => "depth_jelly",
            FoodType::CoralPolyp        => "coral_polyp",
        }
    }

    /// Foods available in a specific biome (includes universal).
    pub fn for_biome(biome: FoodBiome) -> Vec<FoodType> {
        Self::ALL.iter()
            .filter(|f| f.biome() == biome || f.biome() == FoodBiome::Universal)
            .copied()
            .collect()
    }

    /// Is this food from the creature's home biome?
    pub fn is_native_for(&self, species: &crate::genome::Species) -> bool {
        self.biome().native_species().as_ref() == Some(species)
    }
}

// ===================================================================
// SPECIES DECAY RATES
// ===================================================================

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
// CONSTANTS
// ===================================================================

/// Nutrient level below which deficiency effects kick in.
pub const DEFICIENCY_THRESHOLD: f32 = 20.0;

/// Happiness bonus from feeding a preferred (native biome) food.
pub const PREFERRED_FOOD_HAPPINESS: f32 = 8.0;

/// Base happiness from any feeding.
pub const BASE_FEED_HAPPINESS: f32 = 3.0;

/// Happiness penalty for feeding non-native biome food.
pub const FOREIGN_FOOD_PENALTY: f32 = -2.0;
