//! # Mind
//!
//! The creature's AI engine, built in two layers:
//!
//! 1. **Finite State Machine (FSM)** — defines the current mood state
//!    (hungry, tired, playful, etc.) and drives transitions between states.
//!
//! 2. **Emergent behaviour** — transitions depend on the genome, vital stats,
//!    and a random component. The result looks like personality: same stats,
//!    different genes → different behaviour.
//!
//! ## Future — Phase 4
//! A small local neural network (via `candle-core`) will replace part of the
//! hardcoded rules, learning the specific interaction patterns of each owner.

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// The creature's current emotional and behavioural state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MoodState {
    Happy,
    Hungry,
    Tired,
    Lonely,
    Playful,
    Sick,
    Sleeping,
}

impl MoodState {
    pub fn label(&self) -> &str {
        match self {
            MoodState::Happy    => "Happy",
            MoodState::Hungry   => "Hungry",
            MoodState::Tired    => "Tired",
            MoodState::Lonely   => "Lonely",
            MoodState::Playful  => "Playful",
            MoodState::Sick     => "Sick",
            MoodState::Sleeping => "Sleeping",
        }
    }

    /// Returns the mood key used for building sprite asset paths.
    ///
    /// The spawn system combines this with the body part slot to form
    /// a filename: `{slot}_{mood_key}.png` (e.g. `eye_left_hungry.png`).
    /// Happy maps to "idle" because the idle pose is the default state.
    pub fn mood_key(&self) -> &str {
        match self {
            MoodState::Happy    => "idle",
            MoodState::Hungry   => "hungry",
            MoodState::Tired    => "tired",
            MoodState::Lonely   => "lonely",
            MoodState::Playful  => "playful",
            MoodState::Sick     => "sick",
            MoodState::Sleeping => "sleeping",
        }
    }
}

/// Core vital stats of the creature (all values are 0.0–100.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalStats {
    /// 0 = full, 100 = starving
    pub hunger: f32,
    /// 0 = miserable, 100 = euphoric
    pub happiness: f32,
    /// 0 = exhausted, 100 = fully rested
    pub energy: f32,
    /// 0 = critical, 100 = perfect health
    pub health: f32,
}

impl VitalStats {
    pub fn new() -> Self {
        Self {
            hunger:    30.0,
            happiness: 70.0,
            energy:    80.0,
            health:    100.0,
        }
    }
}

/// The creature's mind: holds the current mood state and vital stats.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Mind {
    pub mood:      MoodState,
    pub stats:     VitalStats,
    /// Age in game ticks (1 tick = 1 real-world second by default)
    pub age_ticks: u64,
}

impl Mind {
    pub fn new() -> Self {
        Self {
            mood:      MoodState::Happy,
            stats:     VitalStats::new(),
            age_ticks: 0,
        }
    }

    /// Feed the creature: reduces hunger, slightly boosts happiness.
    pub fn feed(&mut self) {
        self.stats.hunger    = (self.stats.hunger - 25.0).max(0.0);
        self.stats.happiness = (self.stats.happiness + 8.0).min(100.0);
    }

    /// Play with the creature: boosts happiness, costs energy, increases hunger slightly.
    pub fn play(&mut self) {
        self.stats.happiness = (self.stats.happiness + 15.0).min(100.0);
        self.stats.energy    = (self.stats.energy - 10.0).max(0.0);
        self.stats.hunger    = (self.stats.hunger + 5.0).min(100.0);
    }

    /// Put the creature to sleep: restores energy, sets mood to Sleeping.
    pub fn sleep(&mut self) {
        self.stats.energy = (self.stats.energy + 30.0).min(100.0);
        self.mood = MoodState::Sleeping;
    }

    /// Updates the mood state based on vital stats and the creature's genome.
    ///
    /// Called every tick by [`TimeTickPlugin`]. This is where emergent behaviour
    /// happens: the same stats produce different moods depending on the genome.
    pub fn update_mood(&mut self, genome: &crate::genome::Genome) {
        use rand::Rng;
        let mut rng = rand::rng();

        // Natural stat decay — modulated by genome genes
        let hunger_rate = 0.05 + genome.appetite * 0.1;
        self.stats.hunger    = (self.stats.hunger + hunger_rate).min(100.0);
        self.stats.energy    = (self.stats.energy - 0.03).max(0.0);
        self.stats.happiness = (self.stats.happiness - 0.02).max(0.0);

        // Random mood noise — fragile creatures (low resilience) swing more
        let mood_noise: f32 = rng.random_range(-1.0..1.0) * (1.0 - genome.resilience) * 2.0;
        self.stats.happiness = (self.stats.happiness + mood_noise).clamp(0.0, 100.0);

        // FSM transition: priority order matters
        self.mood = if self.stats.energy < 15.0 {
            MoodState::Sleeping

        } else if self.stats.health < 30.0 {
            MoodState::Sick

        } else if self.stats.hunger > 75.0 {
            MoodState::Hungry

        } else if self.stats.happiness < 25.0 {
            // Emergent behaviour: lonely vs just tired depends on the genome
            if genome.loneliness_sensitivity > 0.6 {
                MoodState::Lonely
            } else {
                MoodState::Tired
            }

        } else if self.stats.happiness > 80.0 && self.stats.energy > 60.0 {
            // Curious creatures are more likely to become Playful spontaneously
            if genome.curiosity > 0.6 || rng.random_range(0.0f32..1.0) < genome.curiosity {
                MoodState::Playful
            } else {
                MoodState::Happy
            }

        } else {
            MoodState::Happy
        };

        self.age_ticks += 1;
    }
}
