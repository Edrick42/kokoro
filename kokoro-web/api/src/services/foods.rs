//! Food service — business logic for food data.

use kokoro_shared::food::FoodType;
use crate::models::foods::FoodResponse;

/// Returns all available food items.
pub fn list_all() -> Vec<FoodResponse> {
    FoodType::ALL.iter().map(|f| FoodResponse {
        name: f.name().into(),
        description: f.description().into(),
        biome: f.biome().label().into(),
        category: f.category().label().into(),
    }).collect()
}
