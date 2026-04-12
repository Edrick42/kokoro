//! Disease system configuration — conditions, durations, recovery rates.

/// Duration of each condition in ticks before natural recovery.
pub mod duration {
    pub const COLD: u32 = 120;           // 2 min
    pub const PARASITE: u32 = 200;       // ~3 min
    pub const MALNUTRITION: u32 = 300;   // 5 min (persistent)
    pub const EXHAUSTION: u32 = 90;      // 1.5 min
    pub const INFECTION: u32 = 250;      // ~4 min
}

/// Health drain per tick for each condition.
pub mod health_drain {
    pub const COLD: f32 = 0.02;
    pub const PARASITE: f32 = 0.03;
    pub const MALNUTRITION: f32 = 0.01;
    pub const EXHAUSTION: f32 = 0.005;
    pub const INFECTION: f32 = 0.05;
}

/// Trigger thresholds.
pub mod trigger {
    /// Hygiene below this → risk of Parasite.
    pub const PARASITE_HYGIENE: f32 = 20.0;
    /// Energy below this for N ticks → Exhaustion.
    pub const EXHAUSTION_ENERGY: f32 = 10.0;
    /// Ticks of low energy before exhaustion triggers.
    pub const EXHAUSTION_TICKS: u32 = 30;
    /// Average nutrient below this → Malnutrition.
    pub const MALNUTRITION_FULLNESS: f32 = 25.0;
}

/// Resilience gene reduces recovery time by this factor per 0.1 resilience.
pub const RESILIENCE_RECOVERY_FACTOR: f32 = 0.1;
