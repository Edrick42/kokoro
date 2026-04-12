//! Food-related DTOs for the foods controller.

use serde::Serialize;

/// Food item information returned by the API.
#[derive(Serialize)]
pub struct FoodResponse {
    pub name: String,
    pub description: String,
    pub biome: String,
    pub category: String,
}
