//! Species controller — handles HTTP for species endpoints.

use axum::{extract::Path, Json};
use crate::constants::errors;
use crate::error::ApiError;
use crate::models::species::{SpeciesSummary, SpeciesDetail};
use crate::services;

/// GET /api/species — returns all species.
pub async fn list() -> Json<Vec<SpeciesSummary>> {
    Json(services::species::list_all())
}

/// GET /api/species/{name} — returns a single species by name.
pub async fn get_by_name(
    Path(name): Path<String>,
) -> Result<Json<SpeciesDetail>, ApiError> {
    services::species::find_by_name(&name)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(errors::species_not_found(&name)))
}
