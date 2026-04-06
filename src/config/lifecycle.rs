//! Lifecycle configuration — lifespan, aging, death conditions, legacy.
//!
//! Every Kobara has a finite life. How long it lives depends on care,
//! genetics, nutrition, and luck. Death is permanent — but in the future,
//! essence can be passed to descendants.

use crate::genome::Species;

// ===================================================================
// LIFESPAN — natural age limits
// ===================================================================

/// Base lifespan in ticks per species. Can be extended by good care.
pub fn base_lifespan(species: &Species) -> u64 {
    match species {
        Species::Moluun => 25_000,  // ~7 hours of real play
        Species::Pylum  => 20_000,  // shorter, faster metabolism
        Species::Skael  => 35_000,  // longest, slow metabolism
        Species::Nyxal  => 28_000,  // moderate
    }
}

/// Maximum lifespan bonus from excellent care (percentage of base).
pub const MAX_CARE_BONUS: f32 = 0.3; // up to 30% longer life

/// Maximum lifespan penalty from neglect (percentage of base).
pub const MAX_NEGLECT_PENALTY: f32 = 0.4; // up to 40% shorter life

// ===================================================================
// AGING — how health degrades over time
// ===================================================================

/// Age at which aging effects begin (fraction of lifespan).
pub const AGING_START: f32 = 0.7; // effects start at 70% of lifespan

/// Health decay rate per tick during old age.
pub const AGING_HEALTH_DECAY: f32 = 0.01;

/// Energy max reduction during old age (energy cap drops).
pub const AGING_ENERGY_CAP_REDUCTION: f32 = 0.3; // max energy drops by 30%

// ===================================================================
// DEATH CONDITIONS
// ===================================================================

/// Health below this = death.
pub const DEATH_HEALTH_THRESHOLD: f32 = 0.0;

/// Consecutive ticks at health 0 before death (grace period).
pub const DEATH_GRACE_TICKS: u32 = 10;

/// Chronic starvation: ticks at hunger 100 before health starts dropping.
pub const STARVATION_THRESHOLD_TICKS: u32 = 60;

/// Health loss per tick during starvation.
pub const STARVATION_HEALTH_DECAY: f32 = 0.5;

/// Chronic dehydration (water nutrient at 0): health loss per tick.
pub const DEHYDRATION_HEALTH_DECAY: f32 = 0.8;

// ===================================================================
// CARE QUALITY — affects lifespan
// ===================================================================

/// How care quality is calculated (running average of these factors):
/// - Average nutrient balance (0-1)
/// - Touch frequency (0-1, based on recent touches)
/// - Absence frequency (0-1, inverse of time away)
/// - Mood variety (0-1, how many different moods experienced)

/// Ticks between care quality recalculations.
pub const CARE_CHECK_INTERVAL: u64 = 100;

// ===================================================================
// LEGACY (Future — documented for lore, not yet implemented)
// ===================================================================

/// In the future, when a Kobara dies after fulfilling its purpose
/// (having descendants, reaching elder stage, forming bonds), its
/// kokoro-sac essence can transfer to a descendant. This grants:
///
/// - Partial gene preservation (some traits carry over)
/// - Neural network seed (descendant starts with parent's learned patterns)
/// - Bonus stats at birth (higher starting values)
/// - Unique "ancestral mark" visual (visible heritage)
///
/// The amount transferred depends on:
/// - Age at death (elder = more essence)
/// - Care quality during life (well-cared = more essence)
/// - Number of descendants (more = diluted per child)
/// - Bond strength with player (stronger = more preserved)
///
/// This system requires breeding (Phase 9 — mobile P2P) to function.
/// For now, death is final.
pub const _LEGACY_PLACEHOLDER: bool = false;
