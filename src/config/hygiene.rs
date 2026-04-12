//! Hygiene system configuration — decay rates, thresholds, cleaning effectiveness.

/// Hygiene decay per tick while active (moving, playing).
pub const ACTIVE_DECAY: f32 = 0.08;

/// Hygiene decay per tick while idle.
pub const IDLE_DECAY: f32 = 0.02;

/// Hygiene decay per tick while sleeping (very slow — bodies rest).
pub const SLEEP_DECAY: f32 = 0.005;

/// Below this level, health starts being affected.
pub const HEALTH_PENALTY_THRESHOLD: f32 = 30.0;

/// Health penalty per tick when hygiene is critically low.
pub const HEALTH_PENALTY: f32 = 0.01;

/// Below this level, happiness is affected.
pub const HAPPINESS_PENALTY_THRESHOLD: f32 = 40.0;

/// Happiness penalty per tick when dirty.
pub const HAPPINESS_PENALTY: f32 = 0.02;

/// Threshold below which the creature will auto-clean when idle.
pub const AUTO_CLEAN_THRESHOLD: f32 = 50.0;

/// Species-specific cleaning effectiveness (how much hygiene is restored per clean action).
pub mod clean {
    /// Moluun: groom (lick fur) — slow but thorough.
    pub const MOLUUN_GROOM: f32 = 25.0;
    /// Pylum: preen (arrange feathers) — medium.
    pub const PYLUM_PREEN: f32 = 30.0;
    /// Skael: shed (shed old scales) — fast burst.
    pub const SKAEL_SHED: f32 = 40.0;
    /// Nyxal: ink clean (expel ink cloud) — instant but costs energy.
    pub const NYXAL_INK: f32 = 50.0;
    /// Energy cost for Nyxal ink cleaning.
    pub const NYXAL_INK_ENERGY_COST: f32 = 10.0;
}
