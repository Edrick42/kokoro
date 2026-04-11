//! Learning and preferences — creatures develop individual tastes over time.
//!
//! Each creature tracks what food it's been given and how it felt.
//! After enough experience, it forms opinions: likes, dislikes, requests.
//! Creatures can refuse food they dislike and ask for what they want.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::game::state::AppState;
use serde::{Deserialize, Serialize};

use crate::config::nutrition::FoodType;
use crate::creature::species::CreatureRoot;
use crate::genome::Genome;
use crate::mind::Mind;
use crate::mind::nutrition::is_preferred_food;

/// Minimum feedings before an opinion forms.
const OPINION_THRESHOLD: u32 = 5;
/// Feedings before a strong preference (can refuse).
#[allow(dead_code)]
const STRONG_THRESHOLD: u32 = 10;
/// Ticks between preference checks.
const CHECK_INTERVAL: u64 = 30;

/// Memory of a specific food.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodMemory {
    pub times_fed: u32,
    /// Running average satisfaction (0-1, 0.5 = neutral).
    pub satisfaction: f32,
}

/// What the creature currently wants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CreatureRequest {
    WantsFood(FoodType),
    WantsPlay,
    WantsSleep,
    RefusesFood(FoodType),
    None,
}

/// Tracks food and interaction history to form preferences.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceMemory {
    pub food_history: HashMap<String, FoodMemory>,
    pub touch_count: u32,
    pub current_request: CreatureRequest,
}

impl Default for PreferenceMemory {
    fn default() -> Self {
        Self {
            food_history: HashMap::new(),
            touch_count: 0,
            current_request: CreatureRequest::None,
        }
    }
}

impl PreferenceMemory {
    /// Record a feeding event. Returns satisfaction level.
    #[allow(dead_code)]
    pub fn record_feeding(&mut self, food: &FoodType, species: &crate::genome::Species) -> f32 {
        let key = food.event_key().to_string();
        let is_pref = is_preferred_food(species, food);
        let satisfaction = if is_pref { 0.8 } else { 0.3 };

        let memory = self.food_history.entry(key).or_insert(FoodMemory {
            times_fed: 0,
            satisfaction: 0.5,
        });
        memory.times_fed += 1;
        // Exponential moving average
        memory.satisfaction = memory.satisfaction * 0.7 + satisfaction * 0.3;
        memory.satisfaction
    }

    /// Check if creature will refuse this food.
    #[allow(dead_code)]
    pub fn will_refuse(&self, food: &FoodType) -> bool {
        let key = food.event_key();
        if let Some(memory) = self.food_history.get(key) {
            memory.times_fed >= STRONG_THRESHOLD && memory.satisfaction < 0.35
        } else {
            false
        }
    }

    /// Get the creature's favorite food (if one exists).
    pub fn favorite_food(&self) -> Option<FoodType> {
        self.food_history.iter()
            .filter(|(_, m)| m.times_fed >= OPINION_THRESHOLD && m.satisfaction > 0.6)
            .max_by(|a, b| a.1.satisfaction.partial_cmp(&b.1.satisfaction).unwrap_or(std::cmp::Ordering::Equal))
            .and_then(|(key, _)| {
                FoodType::ALL.iter().find(|f| f.event_key() == key).copied()
            })
    }
}

pub struct PreferencePlugin;

impl Plugin for PreferencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, preference_check_system.run_if(in_state(AppState::Gameplay)));
    }
}

/// Periodically checks if the creature wants to express a preference.
fn preference_check_system(
    mind: Res<Mind>,
    genome: Res<Genome>,
    mut pref_q: Query<&mut PreferenceMemory, With<CreatureRoot>>,
) {
    if mind.age_ticks % CHECK_INTERVAL != 0 {
        return;
    }

    let Ok(mut prefs) = pref_q.single_mut() else { return };

    // Determine what the creature wants based on stats and preferences
    let request = if mind.stats.energy < 20.0 {
        CreatureRequest::WantsSleep
    } else if mind.stats.hunger > 60.0 {
        if let Some(fav) = prefs.favorite_food() {
            CreatureRequest::WantsFood(fav)
        } else {
            CreatureRequest::None
        }
    } else if mind.stats.happiness > 70.0 && mind.stats.energy > 50.0 && genome.curiosity > 0.5 {
        CreatureRequest::WantsPlay
    } else {
        CreatureRequest::None
    };

    if request != prefs.current_request {
        if request != CreatureRequest::None {
            info!("Creature wants: {:?}", request);
        }
        prefs.current_request = request;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::Species;

    #[test]
    fn preferred_food_builds_satisfaction() {
        let mut prefs = PreferenceMemory::default();
        for _ in 0..10 {
            prefs.record_feeding(&FoodType::VerdanceBerry, &Species::Moluun);
        }
        let mem = prefs.food_history.get("verdance_berry").unwrap();
        assert!(mem.satisfaction > 0.6);
        assert!(mem.times_fed == 10);
    }

    #[test]
    fn non_preferred_food_low_satisfaction() {
        let mut prefs = PreferenceMemory::default();
        for _ in 0..10 {
            prefs.record_feeding(&FoodType::CaveCrustacean, &Species::Moluun);
        }
        let mem = prefs.food_history.get("cave_crustacean").unwrap();
        assert!(mem.satisfaction < 0.5);
    }

    #[test]
    fn refuse_after_strong_threshold() {
        let mut prefs = PreferenceMemory::default();
        for _ in 0..STRONG_THRESHOLD {
            prefs.record_feeding(&FoodType::CaveCrustacean, &Species::Moluun);
        }
        assert!(prefs.will_refuse(&FoodType::CaveCrustacean));
        assert!(!prefs.will_refuse(&FoodType::VerdanceBerry));
    }

    #[test]
    fn favorite_food_detection() {
        let mut prefs = PreferenceMemory::default();
        for _ in 0..OPINION_THRESHOLD {
            prefs.record_feeding(&FoodType::VerdanceBerry, &Species::Moluun);
        }
        assert_eq!(prefs.favorite_food(), Some(FoodType::VerdanceBerry));
    }
}
