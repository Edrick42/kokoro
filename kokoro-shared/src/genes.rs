//! Gene definitions and species-specific ranges.

use super::species::Species;

/// Gene range: (min, max) for random generation.
pub struct GeneRanges {
    pub curiosity: (f32, f32),
    pub appetite: (f32, f32),
    pub resilience: (f32, f32),
    // Universal genes (same range for all species)
    pub loneliness_sensitivity: (f32, f32),
    pub circadian: (f32, f32),
    pub learning_rate: (f32, f32),
    pub hue: (f32, f32),
}

pub fn gene_ranges(species: &Species) -> GeneRanges {
    let (curiosity, appetite, resilience) = match species {
        Species::Moluun => ((0.2, 1.0), (0.1, 0.8), (0.2, 1.0)),
        Species::Pylum  => ((0.4, 1.0), (0.1, 0.5), (0.3, 0.9)),
        Species::Skael  => ((0.1, 0.7), (0.3, 1.0), (0.5, 1.0)),
        Species::Nyxal  => ((0.5, 1.0), (0.2, 0.7), (0.1, 0.6)),
    };
    GeneRanges {
        curiosity,
        appetite,
        resilience,
        loneliness_sensitivity: (0.1, 0.9),
        circadian: (0.0, 1.0),
        learning_rate: (0.1, 0.6),
        hue: (0.0, 360.0),
    }
}

/// Gene metadata for display/documentation.
pub struct GeneMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub range: &'static str,
}

pub const GENE_META: &[GeneMeta] = &[
    GeneMeta { name: "Curiosity",              description: "Exploration tendency, affects eye spacing",  range: "0.0–1.0" },
    GeneMeta { name: "Loneliness Sensitivity",  description: "Suffering when alone",                     range: "0.0–1.0" },
    GeneMeta { name: "Appetite",               description: "Hunger rate, affects body width",           range: "0.0–1.0" },
    GeneMeta { name: "Circadian",              description: "Sleep preference (0=night owl, 1=early bird)", range: "0.0–1.0" },
    GeneMeta { name: "Resilience",             description: "Emotional recovery speed",                  range: "0.0–1.0" },
    GeneMeta { name: "Learning Rate",          description: "Neural network learning speed",             range: "0.0–1.0" },
    GeneMeta { name: "Hue",                    description: "Body color (HSL hue)",                     range: "0.0–360.0" },
];
