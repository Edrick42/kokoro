//! Vital stats — initial values, FSM thresholds, decay rates, mood behavior.

/// Initial vital stats for a newborn creature.
pub mod initial_stats {
    pub const HUNGER: f32 = 30.0;
    pub const THIRST: f32 = 30.0;
    pub const HAPPINESS: f32 = 70.0;
    pub const ENERGY: f32 = 80.0;
    pub const HEALTH: f32 = 100.0;
}

/// FSM mood transition thresholds.
pub mod mood_thresholds {
    pub const ENERGY_SLEEP: f32 = 15.0;
    pub const HEALTH_SICK: f32 = 30.0;
    pub const HAPPINESS_SAD: f32 = 25.0;
    pub const ENERGY_PLAYFUL: f32 = 60.0;
    pub const LONELINESS_GENE_THRESHOLD: f32 = 0.6;
    pub const CURIOSITY_GENE_THRESHOLD: f32 = 0.6;

    pub const HUNGER_DEFAULT: f32 = 75.0;
    pub const HUNGER_SKAEL: f32 = 65.0;
    pub const HUNGER_PYLUM: f32 = 85.0;

    pub const THIRST_DEFAULT: f32 = 70.0;
    pub const THIRST_SKAEL: f32 = 80.0;   // reptiles tolerate dehydration better
    pub const THIRST_NYXAL: f32 = 55.0;   // aquatic = needs water constantly

    pub const PLAYFUL_DEFAULT: f32 = 80.0;
    pub const PLAYFUL_PYLUM: f32 = 70.0;
    pub const PLAYFUL_SKAEL: f32 = 90.0;
}

/// Natural stat decay per tick.
pub mod stat_decay {
    pub const HUNGER_BASE: f32 = 0.05;
    pub const HUNGER_APPETITE_MULTIPLIER: f32 = 0.1;
    pub const THIRST_BASE: f32 = 0.04;
    pub const ENERGY_DECAY: f32 = 0.03;
    pub const HAPPINESS_DECAY: f32 = 0.02;
}

/// Mood transition behavior.
pub mod mood {
    pub const DRAIN_RATE: f32 = 5.0;
    pub const COOLDOWN_TICKS: u32 = 5;
    pub const SLEEP_COOLDOWN_TICKS: u32 = 10;
    pub const PENDING_EPSILON: f32 = 0.1;
}
