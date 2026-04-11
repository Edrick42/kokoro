//! Nutrition system — tracks nutrient levels and applies deficiency effects.
//!
//! Each creature maintains levels of 7 fundamental nutrients (0.0–100.0).
//! Nutrients decay per tick based on species biology. Deficiencies cause
//! specific stat penalties (low protein → health drops, low carbs → energy drops).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::state::AppState;

use crate::config::nutrition::{FoodType, NutrientProfile, self as nutr};
use crate::creature::species::CreatureRoot;
use crate::genome::Genome;
use crate::mind::Mind;

/// Tracks the creature's nutrient levels (all 0.0–100.0).
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct NutrientState {
    pub protein: f32,
    pub carbs: f32,
    pub fat: f32,
    pub water: f32,
    pub minerals: f32,
    pub vitamins: f32,
    pub fiber: f32,
}

impl Default for NutrientState {
    fn default() -> Self {
        Self {
            protein: 60.0,
            carbs: 60.0,
            fat: 60.0,
            water: 70.0,
            minerals: 50.0,
            vitamins: 50.0,
            fiber: 50.0,
        }
    }
}

impl NutrientState {
    /// Add nutrients from a food item (clamped to 100).
    pub fn add_food(&mut self, profile: &NutrientProfile) {
        self.protein  = (self.protein  + profile.protein).min(100.0);
        self.carbs    = (self.carbs    + profile.carbs).min(100.0);
        self.fat      = (self.fat      + profile.fat).min(100.0);
        self.water    = (self.water    + profile.water).min(100.0);
        self.minerals = (self.minerals + profile.minerals).min(100.0);
        self.vitamins = (self.vitamins + profile.vitamins).min(100.0);
        self.fiber    = (self.fiber    + profile.fiber).min(100.0);
    }

    /// Apply per-tick nutrient decay based on species.
    pub fn decay(&mut self, rates: &NutrientProfile) {
        self.protein  = (self.protein  - rates.protein).max(0.0);
        self.carbs    = (self.carbs    - rates.carbs).max(0.0);
        self.fat      = (self.fat      - rates.fat).max(0.0);
        self.water    = (self.water    - rates.water).max(0.0);
        self.minerals = (self.minerals - rates.minerals).max(0.0);
        self.vitamins = (self.vitamins - rates.vitamins).max(0.0);
        self.fiber    = (self.fiber    - rates.fiber).max(0.0);
    }

    /// Average nutrient fullness (0.0–100.0). Used to derive hunger.
    pub fn average_fullness(&self) -> f32 {
        (self.protein + self.carbs + self.fat + self.water
         + self.minerals + self.vitamins + self.fiber) / 7.0
    }

    /// Check if a specific nutrient is deficient.
    #[allow(dead_code)]
    pub fn is_deficient(&self, nutrient: &str) -> bool {
        let val = match nutrient {
            "protein"  => self.protein,
            "carbs"    => self.carbs,
            "fat"      => self.fat,
            "water"    => self.water,
            "minerals" => self.minerals,
            "vitamins" => self.vitamins,
            "fiber"    => self.fiber,
            _ => 100.0,
        };
        val < nutr::DEFICIENCY_THRESHOLD
    }
}

/// Whether a food is preferred by a species (matches their biome).
pub fn is_preferred_food(species: &crate::genome::Species, food: &FoodType) -> bool {
    use crate::genome::Species;
    matches!(
        (species, food),
        (Species::Moluun, FoodType::VerdanceBerry) |
        (Species::Pylum,  FoodType::ThermalSeed) |
        (Species::Skael,  FoodType::CaveCrustacean) |
        (Species::Nyxal,  FoodType::BiolumPlankton)
    )
}

pub struct NutritionPlugin;

impl Plugin for NutritionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, nutrient_decay_system.run_if(in_state(AppState::Gameplay)));
    }
}

/// Decays nutrients per tick and applies deficiency effects to vital stats.
fn nutrient_decay_system(
    genome: Res<Genome>,
    mut mind: ResMut<Mind>,
    mut nutrient_q: Query<&mut NutrientState, With<CreatureRoot>>,
) {
    let Ok(mut nutrients) = nutrient_q.single_mut() else { return };

    // Decay nutrients based on species biology
    let rates = nutr::species_decay(&genome.species);
    nutrients.decay(&rates);

    // Derive hunger from nutrient fullness (inverted: 100 fullness = 0 hunger)
    let fullness = nutrients.average_fullness();
    mind.stats.hunger = (100.0 - fullness).clamp(0.0, 100.0);

    // Deficiency effects
    let threshold = nutr::DEFICIENCY_THRESHOLD;

    // Low protein → health drops faster
    if nutrients.protein < threshold {
        let severity = 1.0 - (nutrients.protein / threshold);
        mind.stats.health = (mind.stats.health - 0.02 * severity).max(0.0);
    }

    // Low carbs → energy drops faster
    if nutrients.carbs < threshold {
        let severity = 1.0 - (nutrients.carbs / threshold);
        mind.stats.energy = (mind.stats.energy - 0.03 * severity).max(0.0);
    }

    // Low water → health penalty (dehydration)
    if nutrients.water < threshold {
        let severity = 1.0 - (nutrients.water / threshold);
        mind.stats.health = (mind.stats.health - 0.05 * severity).max(0.0);
    }

    // Low fiber → appetite increases (hunger grows faster)
    // (This is handled by the hunger derivation — low fiber = lower average = higher hunger)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::nutrition::FoodType;

    #[test]
    fn food_adds_nutrients() {
        let mut state = NutrientState::default();
        let before_protein = state.protein;
        state.add_food(&FoodType::CaveCrustacean.nutrients());
        assert!(state.protein > before_protein);
        assert!(state.protein <= 100.0);
    }

    #[test]
    fn decay_reduces_nutrients() {
        let mut state = NutrientState::default();
        let before = state.carbs;
        let rates = NutrientProfile {
            protein: 0.0, carbs: 1.0, fat: 0.0, water: 0.0,
            minerals: 0.0, vitamins: 0.0, fiber: 0.0,
        };
        state.decay(&rates);
        assert!(state.carbs < before);
    }

    #[test]
    fn average_fullness_calculation() {
        let state = NutrientState {
            protein: 100.0, carbs: 100.0, fat: 100.0, water: 100.0,
            minerals: 100.0, vitamins: 100.0, fiber: 100.0,
        };
        assert!((state.average_fullness() - 100.0).abs() < 0.01);
    }

    #[test]
    fn deficiency_detection() {
        let state = NutrientState {
            protein: 10.0, carbs: 60.0, fat: 60.0, water: 60.0,
            minerals: 60.0, vitamins: 60.0, fiber: 60.0,
        };
        assert!(state.is_deficient("protein"));
        assert!(!state.is_deficient("carbs"));
    }

    #[test]
    fn preferred_food_check() {
        use crate::genome::Species;
        assert!(is_preferred_food(&Species::Moluun, &FoodType::VerdanceBerry));
        assert!(!is_preferred_food(&Species::Moluun, &FoodType::CaveCrustacean));
    }
}
