//! Absence awareness — the Mirror Bond.
//!
//! Tracks how long the player was away and applies effects when they return.
//! The creature remembers being left alone. Resilient creatures handle it
//! better; sensitive ones suffer more.

use bevy::prelude::*;

use crate::config;
use crate::genome::Genome;
use super::{Mind, MoodState};

/// Tracks player absence and reunion state.
#[derive(Resource)]
pub struct AbsenceState {
    /// How many seconds the player was away before this session.
    pub seconds_away: u64,
    /// Countdown ticks for the reunion animation (starts at ~5 seconds worth).
    pub reunion_ticks: u32,
    /// Whether the reunion reaction has fully played out.
    pub acknowledged: bool,
}

impl AbsenceState {
    pub fn new(seconds_away: u64) -> Self {
        let reunion_ticks = if seconds_away > config::absence::TRIVIAL {
            config::absence::REUNION_TICKS
        } else {
            0
        };
        Self {
            seconds_away,
            reunion_ticks,
            acknowledged: seconds_away <= config::absence::TRIVIAL,
        }
    }

    /// Human-readable absence description.
    pub fn description(&self) -> &'static str {
        match self.seconds_away {
            s if s < config::absence::TRIVIAL => "just now",
            s if s < config::absence::SHORT   => "a few minutes",
            s if s < config::absence::MEDIUM  => "a while",
            s if s < config::absence::LONG    => "many hours",
            _ => "a long time",
        }
    }
}

/// Applies stat effects from the player's absence. Runs once at startup.
pub fn apply_absence_effects(
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
    absence: Res<AbsenceState>,
) {
    let secs = absence.seconds_away;
    if secs < config::absence::TRIVIAL {
        return; // Less than a minute — no effect
    }

    // Modifiers from genome: resilient creatures handle absence better
    let resilience_factor = 1.0 - genome.resilience * 0.5; // 0.5 to 1.0
    let loneliness_factor = 0.5 + genome.loneliness_sensitivity * 0.5; // 0.5 to 1.0

    if secs < config::absence::SHORT {
        // 1-30 minutes: slight effects
        mind.stats.hunger = (mind.stats.hunger + 10.0 * resilience_factor).min(100.0);
        mind.stats.happiness = (mind.stats.happiness - 8.0 * loneliness_factor).max(0.0);
    } else if secs < config::absence::MEDIUM {
        // 30 min - 4 hours: significant effects
        mind.stats.hunger = (mind.stats.hunger + 25.0 * resilience_factor).min(100.0);
        mind.stats.happiness = (mind.stats.happiness - 20.0 * loneliness_factor).max(0.0);
        mind.stats.energy = (mind.stats.energy - 10.0 * resilience_factor).max(0.0);
    } else if secs < config::absence::LONG {
        // 4-24 hours: severe effects
        mind.stats.hunger = (mind.stats.hunger + 40.0 * resilience_factor).min(100.0);
        mind.stats.happiness = (mind.stats.happiness - 35.0 * loneliness_factor).max(0.0);
        mind.stats.energy = (mind.stats.energy - 20.0 * resilience_factor).max(0.0);
        mind.mood = MoodState::Lonely;
    } else {
        // 24+ hours: critical neglect
        mind.stats.hunger = (mind.stats.hunger + 55.0 * resilience_factor).min(100.0);
        mind.stats.happiness = (mind.stats.happiness - 50.0 * loneliness_factor).max(0.0);
        mind.stats.energy = (mind.stats.energy - 30.0 * resilience_factor).max(0.0);
        mind.stats.health = (mind.stats.health - 10.0 * resilience_factor).max(0.0);
        mind.mood = MoodState::Sick;
    }

    // Survival floor: the creature always comes back alive with minimal stats.
    // No matter how long the absence, the player gets a chance to care for it.
    // Health floor prevents instant death from accumulated decay systems.
    mind.stats.health = mind.stats.health.max(15.0);
    mind.stats.energy = mind.stats.energy.max(5.0);

    if secs >= config::absence::TRIVIAL {
        info!(
            "Absence: {} seconds ({}) — hunger +{:.0}, happiness -{:.0}",
            secs,
            absence.description(),
            mind.stats.hunger,
            mind.stats.happiness,
        );
    }
}

/// Counts down the reunion animation ticks. Species-specific reactions
/// are handled by the species_behavior system checking this resource.
pub fn reunion_countdown(
    mut absence: ResMut<AbsenceState>,
) {
    if absence.acknowledged {
        return;
    }

    if absence.reunion_ticks > 0 {
        absence.reunion_ticks -= 1;
    } else {
        absence.acknowledged = true;
    }
}
