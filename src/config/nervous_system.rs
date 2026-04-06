//! Nervous system configuration — sensory receptor sensitivity per species×body part.
//!
//! Based on real biology: skin has mechanoreceptors (pressure), thermoreceptors
//! (warmth), nociceptors (pain), and nerve endings (pleasure). Each body part
//! has different receptor density, creating sweet spots and sensitive areas.

use crate::genome::Species;

/// Sensory sensitivity profile for a body part.
#[derive(Debug, Clone, Copy)]
pub struct Sensitivity {
    /// How good touch feels here (0 = nothing, 1 = bliss).
    pub pleasure: f32,
    /// How sensitive to rough/unwanted touch (0 = numb, 1 = very painful).
    pub pain: f32,
    /// How much warmth/comfort is felt (0 = cold, 1 = warm).
    pub warmth: f32,
}

impl Sensitivity {
    pub const NEUTRAL: Self = Self { pleasure: 0.3, pain: 0.2, warmth: 0.3 };
}

/// Returns the sensitivity profile for a given species and body part slot.
pub fn sensitivity(species: &Species, slot: &str) -> Sensitivity {
    match species {
        Species::Moluun => match slot {
            "body"      => Sensitivity { pleasure: 0.5, pain: 0.2, warmth: 0.7 },
            "ear_left" | "ear_right" =>
                           Sensitivity { pleasure: 0.9, pain: 0.3, warmth: 0.4 },
            "eye_left" | "eye_right" =>
                           Sensitivity { pleasure: 0.1, pain: 0.9, warmth: 0.1 },
            "mouth"     => Sensitivity { pleasure: 0.3, pain: 0.5, warmth: 0.3 },
            _           => Sensitivity::NEUTRAL,
        },
        Species::Pylum => match slot {
            "body"      => Sensitivity { pleasure: 0.4, pain: 0.2, warmth: 0.5 },
            "wing_left" | "wing_right" =>
                           Sensitivity { pleasure: 0.8, pain: 0.2, warmth: 0.3 },
            "eye_left" | "eye_right" =>
                           Sensitivity { pleasure: 0.1, pain: 0.9, warmth: 0.1 },
            "beak"      => Sensitivity { pleasure: 0.1, pain: 0.8, warmth: 0.1 },
            "tail"      => Sensitivity { pleasure: 0.5, pain: 0.3, warmth: 0.4 },
            _           => Sensitivity::NEUTRAL,
        },
        Species::Skael => match slot {
            "body"      => Sensitivity { pleasure: 0.3, pain: 0.1, warmth: 0.6 },
            "crest_left" | "crest_right" =>
                           Sensitivity { pleasure: 0.6, pain: 0.3, warmth: 0.2 },
            "eye_left" | "eye_right" =>
                           Sensitivity { pleasure: 0.1, pain: 0.9, warmth: 0.1 },
            "snout"     => Sensitivity { pleasure: 0.2, pain: 0.7, warmth: 0.3 },
            "tail"      => Sensitivity { pleasure: 0.4, pain: 0.2, warmth: 0.5 },
            _           => Sensitivity::NEUTRAL,
        },
        Species::Nyxal => match slot {
            "body"      => Sensitivity { pleasure: 0.4, pain: 0.3, warmth: 0.5 },
            "mantle"    => Sensitivity { pleasure: 0.5, pain: 0.2, warmth: 0.8 },
            "eye_left" | "eye_right" =>
                           Sensitivity { pleasure: 0.1, pain: 0.8, warmth: 0.1 },
            s if s.contains("tentacle_front") =>
                           Sensitivity { pleasure: 0.9, pain: 0.2, warmth: 0.4 },
            s if s.contains("tentacle_back") =>
                           Sensitivity { pleasure: 0.6, pain: 0.3, warmth: 0.3 },
            _           => Sensitivity::NEUTRAL,
        },
    }
}

/// Happiness change from a touch event.
pub fn touch_happiness(sens: &Sensitivity) -> f32 {
    // Pleasure gives positive, pain gives negative
    (sens.pleasure * 5.0) - (sens.pain * 3.0)
}

/// Energy change from warmth.
pub fn touch_energy(sens: &Sensitivity) -> f32 {
    sens.warmth * 2.0
}

/// Hit radius for body part click detection (pixels from anchor center).
pub const HIT_RADIUS: f32 = 40.0;
