//! Species definitions — the single source of truth.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Species {
    Moluun,
    Pylum,
    Skael,
    Nyxal,
}

impl Species {
    pub const ALL: &[Species] = &[
        Species::Moluun,
        Species::Pylum,
        Species::Skael,
        Species::Nyxal,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Species::Moluun => "Moluun",
            Species::Pylum  => "Pylum",
            Species::Skael  => "Skael",
            Species::Nyxal  => "Nyxal",
        }
    }

    pub fn classification(&self) -> &'static str {
        match self {
            Species::Moluun => "K. moluunaris",
            Species::Pylum  => "K. pylumensis",
            Species::Skael  => "K. skaelith",
            Species::Nyxal  => "K. nyxalaris",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Species::Moluun => "Round, soft, forest-dwelling Kobara. Most social manifestation.",
            Species::Pylum  => "Winged, curious Kobara from the highlands. Reflects the seeking spirit.",
            Species::Skael  => "Scaled, resilient Kobara from underground caves. Quiet strength.",
            Species::Nyxal  => "Tentacled, intelligent Kobara from the deep ocean. Mirrors your thinking.",
        }
    }

    pub fn biome(&self) -> &'static str {
        match self {
            Species::Moluun => "The Verdance",
            Species::Pylum  => "Veridian Highlands",
            Species::Skael  => "Abyssal Shallows",
            Species::Nyxal  => "Abyssal Depths",
        }
    }

    pub fn biome_description(&self) -> &'static str {
        match self {
            Species::Moluun => "Vast bioluminescent forests with spiral-formation trees and luminescent spores.",
            Species::Pylum  => "Towering mesa formations with thermal updrafts and floating mineral deposits.",
            Species::Skael  => "Crystalline cave network with underground rivers and bioluminescent walls.",
            Species::Nyxal  => "Vast subterranean ocean. Lightless, pressurized, etharin-saturated.",
        }
    }

    /// Sprite directory name.
    pub fn dir_name(&self) -> &'static str {
        match self {
            Species::Moluun => "moluun",
            Species::Pylum  => "pylum",
            Species::Skael  => "skael",
            Species::Nyxal  => "nyxal",
        }
    }
}
