//! Species-related DTOs for the species controller.

use serde::Serialize;

/// Summary view — used in list endpoints.
#[derive(Serialize)]
pub struct SpeciesSummary {
    pub name: String,
    pub biome: String,
}

/// Detailed view — used in single-species endpoint.
#[derive(Serialize)]
pub struct SpeciesDetail {
    pub name: String,
    pub classification: String,
    pub biome: String,
    pub description: String,
    pub gene_ranges: GeneRangesDto,
}

/// Gene range representation for the API response.
#[derive(Serialize)]
pub struct GeneRangesDto {
    pub curiosity: (f32, f32),
    pub appetite: (f32, f32),
    pub resilience: (f32, f32),
}
