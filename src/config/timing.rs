//! Timing constants — tick intervals, autosave, neural training, circadian.

/// Seconds between game ticks.
pub const TICK_INTERVAL: f32 = 1.0;

/// Ticks between autosaves.
pub const AUTOSAVE_INTERVAL: u64 = 60;

/// Neural network training schedule.
pub mod neural {
    pub const TRAIN_INTERVAL: u64 = 120;
    pub const MIN_EVENTS: usize = 20;
    pub const SAMPLE_LIMIT: usize = 200;
    pub const EPOCHS: usize = 5;
}

/// Circadian system — day/night preferences.
pub mod circadian {
    pub const NIGHT_OWL_THRESHOLD: f32 = 0.3;
    pub const EARLY_BIRD_THRESHOLD: f32 = 0.7;
    pub const PREFERRED_BONUS: f32 = 1.5;
    pub const NON_PREFERRED_PENALTY: f32 = -1.0;
}

/// Preference learning system.
pub mod preferences {
    /// Minimum interactions before forming an opinion about a food.
    pub const OPINION_THRESHOLD: u32 = 5;
    /// Interactions before the opinion becomes strong (hard to change).
    pub const STRONG_THRESHOLD: u32 = 10;
    /// Ticks between preference checks.
    pub const CHECK_INTERVAL: u64 = 30;
}

/// Reproduction.
pub mod breeding {
    /// Ticks between breeding attempts for the same pair.
    pub const BREED_COOLDOWN: u64 = 60;
}
