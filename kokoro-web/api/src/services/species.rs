//! Species service — business logic for species data.
//! No HTTP types here — just plain data in, plain data out.

use kokoro_shared::species::Species;
use kokoro_shared::genes;
use crate::models::species::{SpeciesSummary, SpeciesDetail, GeneRangesDto};

/// Returns a summary of all available species.
pub fn list_all() -> Vec<SpeciesSummary> {
    Species::ALL.iter().map(|s| SpeciesSummary {
        name: s.name().into(),
        biome: s.biome().into(),
    }).collect()
}

/// Finds a species by name (case-insensitive). Returns None if not found.
pub fn find_by_name(name: &str) -> Option<SpeciesDetail> {
    let species = Species::ALL.iter()
        .find(|s| s.name().to_lowercase() == name.to_lowercase())?;

    let ranges = genes::gene_ranges(species);

    Some(SpeciesDetail {
        name: species.name().into(),
        classification: species.classification().into(),
        biome: species.biome().into(),
        description: species.description().into(),
        gene_ranges: GeneRangesDto {
            curiosity: ranges.curiosity,
            appetite: ranges.appetite,
            resilience: ranges.resilience,
        },
    })
}
