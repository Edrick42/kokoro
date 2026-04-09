//! Biome definitions — world geography shared between game and API.

use super::species::Species;

pub struct BiomeInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub species: &'static str,
}

pub fn biome_by_name(name: &str) -> Option<BiomeInfo> {
    match name.to_lowercase().as_str() {
        "verdance" => Some(BiomeInfo {
            name: "The Verdance",
            description: "Vast bioluminescent forests with spiral-formation trees and luminescent spores.",
            species: "Moluun",
        }),
        "highlands" => Some(BiomeInfo {
            name: "Veridian Highlands",
            description: "Towering mesa formations with thermal updrafts and floating mineral deposits.",
            species: "Pylum",
        }),
        "shallows" => Some(BiomeInfo {
            name: "Abyssal Shallows",
            description: "Crystalline cave network with underground rivers and bioluminescent walls.",
            species: "Skael",
        }),
        "depths" => Some(BiomeInfo {
            name: "Abyssal Depths",
            description: "Vast subterranean ocean. Lightless, pressurized, etharin-saturated.",
            species: "Nyxal",
        }),
        _ => None,
    }
}

pub fn all_biomes() -> Vec<BiomeInfo> {
    Species::ALL.iter().map(|s| BiomeInfo {
        name: s.biome(),
        description: s.biome_description(),
        species: s.name(),
    }).collect()
}
