//! Biome service — business logic for biome data.

use kokoro_shared::biomes;
use crate::models::biomes::BiomeResponse;

/// Finds a biome by name (case-insensitive). Returns None if not found.
pub fn find_by_name(name: &str) -> Option<BiomeResponse> {
    biomes::biome_by_name(name).map(|b| BiomeResponse {
        name: b.name.into(),
        description: b.description.into(),
        species: b.species.into(),
    })
}

/// Returns all biomes.
pub fn list_all() -> Vec<BiomeResponse> {
    biomes::all_biomes().into_iter().map(|b| BiomeResponse {
        name: b.name.into(),
        description: b.description.into(),
        species: b.species.into(),
    }).collect()
}
