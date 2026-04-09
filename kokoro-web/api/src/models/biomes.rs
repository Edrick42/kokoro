//! Biome-related DTOs for the biomes controller.

use serde::Serialize;

/// Biome information returned by the API.
#[derive(Serialize)]
pub struct BiomeResponse {
    pub name: String,
    pub description: String,
    pub species: String,
}
