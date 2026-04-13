//! Autonomic nervous system configuration — sympathetic/parasympathetic thresholds.

/// Sympathetic threshold — above this, fight-or-flight responses activate.
pub const SYMPATHETIC_THRESHOLD: f32 = 0.6;

/// Parasympathetic threshold — below this, rest-and-digest dominates.
pub const PARASYMPATHETIC_THRESHOLD: f32 = 0.3;

/// How fast the autonomic state shifts toward its target (per tick).
pub const BLEND_SPEED: f32 = 0.02;

/// Mood influence on autonomic state.
pub mod mood_drive {
    use crate::mind::MoodState;

    /// Target autonomic level for each mood (0.0 = full parasympathetic, 1.0 = full sympathetic).
    pub fn target(mood: &MoodState) -> f32 {
        match mood {
            MoodState::Sleeping => 0.05,  // deep rest
            MoodState::Happy    => 0.25,  // calm contentment
            MoodState::Tired    => 0.20,  // winding down
            MoodState::Lonely   => 0.45,  // mild anxiety
            MoodState::Hungry   => 0.55,  // growing urgency
            MoodState::Playful  => 0.70,  // high arousal (positive)
            MoodState::Sick     => 0.75,  // body under stress
        }
    }
}

/// Idle behavior timing.
pub mod idle {
    /// Minimum ticks between idle micro-animations.
    pub const MIN_INTERVAL: u32 = 40;
    /// Maximum ticks between idle micro-animations.
    pub const MAX_INTERVAL: u32 = 120;
    /// When sympathetic > threshold, intervals get shorter (multiply by this).
    pub const SYMPATHETIC_SPEED: f32 = 0.5;
    /// When parasympathetic, intervals get longer (multiply by this).
    pub const PARASYMPATHETIC_SPEED: f32 = 1.8;
}
