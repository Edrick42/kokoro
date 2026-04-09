//! Biomes controller — handles HTTP for biome endpoints.

use axum::{extract::Path, Json};
use crate::constants::errors;
use crate::error::ApiError;
use crate::models::biomes::BiomeResponse;
use crate::services;

/// GET /api/biomes — returns all biomes.
pub async fn list() -> Json<Vec<BiomeResponse>> {
    Json(services::biomes::list_all())
}

/// GET /api/biome/{name} — returns a single biome by name.
pub async fn get_by_name(
    Path(name): Path<String>,
) -> Result<Json<BiomeResponse>, ApiError> {
    services::biomes::find_by_name(&name)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(errors::biome_not_found(&name)))
}
