//! # Mind
//!
//! The creature's AI engine, built in three layers:
//!
//! 1. **Finite State Machine (FSM)** — defines the current mood state
//!    (hungry, tired, playful, etc.) and drives transitions between states.
//!    The FSM has **veto power** on critical states (Sick, Sleeping).
//!
//! 2. **Emergent behaviour** — transitions depend on the genome, vital stats,
//!    and a random component. The result looks like personality: same stats,
//!    different genes → different behaviour.
//!
//! 3. **Neural network (Phase 4)** — a small MLP trained locally on the
//!    owner's interaction history. It *suggests* mood transitions that the
//!    FSM can accept or override. Each Kobara's network is unique.

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

pub mod absence;
pub mod neural;
pub mod plugin;
pub mod training;

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

    /// Returns true for moods where the FSM has absolute authority
    /// and the neural network cannot override.
    pub fn is_critical(&self) -> bool {
        matches!(self, MoodState::Sick | MoodState::Sleeping)
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

    /// Feed the creature. Each species reacts differently:
    /// - Moluun: loves food, gets very happy
    /// - Pylum: picky eater, mild reaction
    /// - Skael: eats a lot but stays stoic
    /// - Nyxal: nibbles, moderate enjoyment
    pub fn feed(&mut self, genome: &crate::genome::Genome) {
        use crate::genome::Species;
        let (hunger_relief, happiness_boost) = match genome.species {
            Species::Moluun => (25.0, 12.0),
            Species::Pylum  => (18.0, 5.0),
            Species::Skael  => (35.0, 4.0),
            Species::Nyxal  => (15.0, 8.0),
        };
        self.stats.hunger    = (self.stats.hunger - hunger_relief).max(0.0);
        self.stats.happiness = (self.stats.happiness + happiness_boost).min(100.0);
    }

    /// Play with the creature. Species-specific reactions:
    /// - Moluun: very playful, loves it
    /// - Pylum: gets excited, burns lots of energy
    /// - Skael: barely participates
    /// - Nyxal: intellectually engaged, moderate energy
    pub fn play(&mut self, genome: &crate::genome::Genome) {
        use crate::genome::Species;
        let (happiness_boost, energy_cost, hunger_cost) = match genome.species {
            Species::Moluun => (18.0, 8.0, 5.0),
            Species::Pylum  => (15.0, 12.0, 8.0),
            Species::Skael  => (8.0, 5.0, 3.0),
            Species::Nyxal  => (12.0, 6.0, 4.0),
        };
        self.stats.happiness = (self.stats.happiness + happiness_boost).min(100.0);
        self.stats.energy    = (self.stats.energy - energy_cost).max(0.0);
        self.stats.hunger    = (self.stats.hunger + hunger_cost).min(100.0);
    }

    /// Put the creature to sleep. Species-specific recovery:
    /// - Moluun: sleeps well, good recovery
    /// - Pylum: light sleeper, less recovery
    /// - Skael: deep sleeper, great recovery
    /// - Nyxal: floats asleep, moderate recovery
    pub fn sleep(&mut self, genome: &crate::genome::Genome) {
        use crate::genome::Species;
        let energy_restore = match genome.species {
            Species::Moluun => 30.0,
            Species::Pylum  => 22.0,
            Species::Skael  => 38.0,
            Species::Nyxal  => 28.0,
        };
        self.stats.energy = (self.stats.energy + energy_restore).min(100.0);
        self.mood = MoodState::Sleeping;
    }

    /// Pure FSM mood update — returns what the FSM thinks the mood should be.
    ///
    /// This is separated from `update_mood` so the neural network can
    /// compare its suggestion against the FSM's decision.
    pub fn fsm_mood(&self, genome: &crate::genome::Genome) -> MoodState {
        use rand::Rng;
        let mut rng = rand::rng();

        // Species-specific thresholds
        let hunger_threshold = match genome.species {
            crate::genome::Species::Skael  => 65.0,  // gets hungry sooner (big appetite)
            crate::genome::Species::Pylum  => 85.0,  // tolerates hunger longer
            _ => 75.0,
        };
        let playful_threshold = match genome.species {
            crate::genome::Species::Pylum  => 70.0,  // gets playful easily (curious)
            crate::genome::Species::Skael  => 90.0,  // rarely playful (stoic)
            _ => 80.0,
        };

        if self.stats.energy < 15.0 {
            MoodState::Sleeping
        } else if self.stats.health < 30.0 {
            MoodState::Sick
        } else if self.stats.hunger > hunger_threshold {
            MoodState::Hungry
        } else if self.stats.happiness < 25.0 {
            if genome.loneliness_sensitivity > 0.6 {
                MoodState::Lonely
            } else {
                MoodState::Tired
            }
        } else if self.stats.happiness > playful_threshold && self.stats.energy > 60.0 {
            if genome.curiosity > 0.6 || rng.random_range(0.0f32..1.0) < genome.curiosity {
                MoodState::Playful
            } else {
                MoodState::Happy
            }
        } else {
            MoodState::Happy
        }
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

        // FSM transition
        self.mood = self.fsm_mood(genome);

        self.age_ticks += 1;
    }
}
