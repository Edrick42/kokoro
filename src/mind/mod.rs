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
//!
//! ## Gradual transitions
//!
//! Actions (feed, play, sleep) don't change stats instantly. Instead, they
//! queue **pending** changes that drain gradually over several ticks. This
//! makes the creature feel organic rather than robotic.
//!
//! Mood changes have a **cooldown** — the creature won't flicker between
//! moods every tick. After a mood transition, it stays for at least 5 ticks.

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

pub mod absence;
pub mod lifecycle;
pub mod neural;
pub mod nutrition;
pub mod plugin;
pub mod preferences;
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

    #[allow(dead_code)]
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

    pub fn is_critical(&self) -> bool {
        matches!(self, MoodState::Sick | MoodState::Sleeping)
    }
}

/// Core vital stats of the creature (all values are 0.0–100.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalStats {
    pub hunger: f32,
    pub happiness: f32,
    pub energy: f32,
    pub health: f32,
}

impl VitalStats {
    pub fn new() -> Self {
        use crate::config::initial_stats;
        Self {
            hunger:    initial_stats::HUNGER,
            happiness: initial_stats::HAPPINESS,
            energy:    initial_stats::ENERGY,
            health:    initial_stats::HEALTH,
        }
    }
}

/// The creature's mind: holds the current mood state, vital stats,
/// pending action effects, and mood transition cooldown.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Mind {
    pub mood:      MoodState,
    pub stats:     VitalStats,
    pub age_ticks: u64,

    /// Pending stat changes that drain gradually (from feed/play/sleep).
    #[serde(default)]
    pub pending_hunger: f32,
    #[serde(default)]
    pub pending_happiness: f32,
    #[serde(default)]
    pub pending_energy: f32,

    /// Ticks remaining before the FSM can change the mood again.
    /// Prevents rapid flickering between mood states.
    #[serde(default)]
    pub mood_cooldown: u32,
}

use crate::config;

impl Mind {
    pub fn new() -> Self {
        Self {
            mood:              MoodState::Happy,
            stats:             VitalStats::new(),
            age_ticks:         0,
            pending_hunger:    0.0,
            pending_happiness: 0.0,
            pending_energy:    0.0,
            mood_cooldown:     0,
        }
    }

    /// Feed the creature with a specific food type.
    /// Nutrients are applied to the NutrientState component separately.
    /// This method handles the happiness boost.
    pub fn feed(&mut self, genome: &crate::genome::Genome, food: &crate::config::nutrition::FoodType) {
        use crate::config::nutrition as nutr;
        use crate::mind::nutrition::is_preferred_food;

        let happiness = if is_preferred_food(&genome.species, food) {
            nutr::PREFERRED_FOOD_HAPPINESS + nutr::BASE_FEED_HAPPINESS
        } else {
            nutr::BASE_FEED_HAPPINESS
        };

        self.pending_happiness += happiness;
    }

    /// Play with the creature. Queues gradual happiness boost and energy/hunger costs.
    pub fn play(&mut self, genome: &crate::genome::Genome) {
        use crate::genome::Species;
        let (happiness_boost, energy_cost, hunger_cost) = match genome.species {
            Species::Moluun => config::play::MOLUUN,
            Species::Pylum  => config::play::PYLUM,
            Species::Skael  => config::play::SKAEL,
            Species::Nyxal  => config::play::NYXAL,
        };
        self.pending_happiness += happiness_boost;
        self.pending_energy    -= energy_cost;
        self.pending_hunger    += hunger_cost;
    }

    /// Put the creature to sleep. Queues gradual energy recovery.
    /// The FSM will transition to Sleeping naturally when energy is low.
    pub fn sleep(&mut self, genome: &crate::genome::Genome) {
        use crate::genome::Species;
        let energy_restore = match genome.species {
            Species::Moluun => config::sleep::MOLUUN,
            Species::Pylum  => config::sleep::PYLUM,
            Species::Skael  => config::sleep::SKAEL,
            Species::Nyxal  => config::sleep::NYXAL,
        };
        self.pending_energy += energy_restore;
        self.mood = MoodState::Sleeping;
        self.mood_cooldown = config::mood::SLEEP_COOLDOWN_TICKS;
    }

    /// Pure FSM mood calculation — returns what the mood should be based on stats.
    pub fn fsm_mood(&self, genome: &crate::genome::Genome) -> MoodState {
        use rand::Rng;
        let mut rng = rand::rng();

        use config::mood_thresholds as mt;

        let hunger_threshold = match genome.species {
            crate::genome::Species::Skael  => mt::HUNGER_SKAEL,
            crate::genome::Species::Pylum  => mt::HUNGER_PYLUM,
            _ => mt::HUNGER_DEFAULT,
        };
        let playful_threshold = match genome.species {
            crate::genome::Species::Pylum  => mt::PLAYFUL_PYLUM,
            crate::genome::Species::Skael  => mt::PLAYFUL_SKAEL,
            _ => mt::PLAYFUL_DEFAULT,
        };

        if self.stats.energy < mt::ENERGY_SLEEP {
            MoodState::Sleeping
        } else if self.stats.health < mt::HEALTH_SICK {
            MoodState::Sick
        } else if self.stats.hunger > hunger_threshold {
            MoodState::Hungry
        } else if self.stats.happiness < mt::HAPPINESS_SAD {
            if genome.loneliness_sensitivity > mt::LONELINESS_GENE_THRESHOLD {
                MoodState::Lonely
            } else {
                MoodState::Tired
            }
        } else if self.stats.happiness > playful_threshold && self.stats.energy > mt::ENERGY_PLAYFUL {
            if genome.curiosity > mt::CURIOSITY_GENE_THRESHOLD || rng.random_range(0.0f32..1.0) < genome.curiosity {
                MoodState::Playful
            } else {
                MoodState::Happy
            }
        } else {
            MoodState::Happy
        }
    }

    /// Updates the mood state each tick. Drains pending stat changes gradually
    /// and applies mood transitions with cooldown.
    pub fn update_mood(&mut self, genome: &crate::genome::Genome) {
        use rand::Rng;
        let mut rng = rand::rng();

        // --- Drain pending stat changes gradually ---
        self.drain_pending();

        // --- Natural stat decay ---
        let hunger_rate = config::stat_decay::HUNGER_BASE + genome.appetite * config::stat_decay::HUNGER_APPETITE_MULTIPLIER;
        self.stats.hunger    = (self.stats.hunger + hunger_rate).min(100.0);
        self.stats.energy    = (self.stats.energy - config::stat_decay::ENERGY_DECAY).max(0.0);
        self.stats.happiness = (self.stats.happiness - config::stat_decay::HAPPINESS_DECAY).max(0.0);

        // --- Mood noise ---
        let mood_noise: f32 = rng.random_range(-1.0..1.0) * (1.0 - genome.resilience) * 2.0;
        self.stats.happiness = (self.stats.happiness + mood_noise).clamp(0.0, 100.0);

        // --- Mood transition with cooldown ---
        if self.mood_cooldown > 0 {
            self.mood_cooldown -= 1;
        } else {
            let new_mood = self.fsm_mood(genome);
            // Critical states (Sick, Sleeping) bypass cooldown from non-critical
            let force_critical = new_mood.is_critical() && !self.mood.is_critical();
            if new_mood != self.mood || force_critical {
                self.mood = new_mood;
                self.mood_cooldown = config::mood::COOLDOWN_TICKS;
            }
        }

        self.age_ticks += 1;
    }

    /// Drains pending stat changes at DRAIN_RATE per tick.
    fn drain_pending(&mut self) {
        let rate = config::mood::DRAIN_RATE;
        let eps = config::mood::PENDING_EPSILON;

        // Hunger
        if self.pending_hunger.abs() > eps {
            let delta = self.pending_hunger.signum() * rate.min(self.pending_hunger.abs());
            self.stats.hunger = (self.stats.hunger + delta).clamp(0.0, 100.0);
            self.pending_hunger -= delta;
        } else {
            self.pending_hunger = 0.0;
        }

        // Happiness
        if self.pending_happiness.abs() > eps {
            let delta = self.pending_happiness.signum() * rate.min(self.pending_happiness.abs());
            self.stats.happiness = (self.stats.happiness + delta).clamp(0.0, 100.0);
            self.pending_happiness -= delta;
        } else {
            self.pending_happiness = 0.0;
        }

        // Energy
        if self.pending_energy.abs() > eps {
            let delta = self.pending_energy.signum() * rate.min(self.pending_energy.abs());
            self.stats.energy = (self.stats.energy + delta).clamp(0.0, 100.0);
            self.pending_energy -= delta;
        } else {
            self.pending_energy = 0.0;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::{Genome, Species};

    #[test]
    fn feed_queues_happiness_pending() {
        let genome = Genome::random_for(Species::Moluun);
        let mut mind = Mind::new();

        mind.feed(&genome, &crate::config::nutrition::FoodType::LatticeFruit);

        // Happiness should be queued as pending, not applied instantly
        assert!(mind.pending_happiness > 0.0);
    }

    #[test]
    fn preferred_food_gives_more_happiness() {
        let genome = Genome::random_for(Species::Moluun);
        let mut mind_pref = Mind::new();
        let mut mind_other = Mind::new();

        // Moluun prefers VerdanceBerry
        mind_pref.feed(&genome, &crate::config::nutrition::FoodType::VerdanceBerry);
        mind_other.feed(&genome, &crate::config::nutrition::FoodType::CaveCrustacean);

        assert!(mind_pref.pending_happiness > mind_other.pending_happiness);
    }

    #[test]
    fn mood_cooldown_prevents_flickering() {
        let genome = Genome::random_for(Species::Moluun);
        let mut mind = Mind::new();
        mind.mood = MoodState::Happy;
        mind.mood_cooldown = 3;

        // Even if FSM says different mood, cooldown blocks it
        mind.stats.hunger = 90.0; // would trigger Hungry
        mind.update_mood(&genome);
        // Mood might still be Happy because cooldown was active
        assert!(mind.mood_cooldown <= 3);
    }

    #[test]
    fn sleep_forces_sleeping_with_extended_cooldown() {
        let genome = Genome::random_for(Species::Moluun);
        let mut mind = Mind::new();
        mind.sleep(&genome);

        assert_eq!(mind.mood, MoodState::Sleeping);
        assert_eq!(mind.mood_cooldown, 10);
        assert!(mind.pending_energy > 0.0);
    }

    #[test]
    fn species_preferred_food_match() {
        use crate::mind::nutrition::is_preferred_food;
        use crate::config::nutrition::FoodType;

        // Each species has a preferred food
        assert!(is_preferred_food(&Species::Moluun, &FoodType::VerdanceBerry));
        assert!(is_preferred_food(&Species::Pylum,  &FoodType::ThermalSeed));
        assert!(is_preferred_food(&Species::Skael,  &FoodType::CaveCrustacean));
        assert!(is_preferred_food(&Species::Nyxal,  &FoodType::BiolumPlankton));

        // Non-preferred
        assert!(!is_preferred_food(&Species::Moluun, &FoodType::CaveCrustacean));
    }

    #[test]
    fn fsm_sleeping_when_exhausted() {
        let genome = Genome::random_for(Species::Moluun);
        let mind = Mind {
            mood: MoodState::Happy,
            stats: VitalStats { hunger: 30.0, happiness: 70.0, energy: 5.0, health: 100.0 },
            age_ticks: 0,
            pending_hunger: 0.0, pending_happiness: 0.0, pending_energy: 0.0,
            mood_cooldown: 0,
        };
        assert_eq!(mind.fsm_mood(&genome), MoodState::Sleeping);
    }

    #[test]
    fn fsm_sick_when_low_health() {
        let genome = Genome::random_for(Species::Moluun);
        let mind = Mind {
            mood: MoodState::Happy,
            stats: VitalStats { hunger: 30.0, happiness: 70.0, energy: 80.0, health: 10.0 },
            age_ticks: 0,
            pending_hunger: 0.0, pending_happiness: 0.0, pending_energy: 0.0,
            mood_cooldown: 0,
        };
        assert_eq!(mind.fsm_mood(&genome), MoodState::Sick);
    }

    #[test]
    fn fsm_hungry_when_starving() {
        let genome = Genome::random_for(Species::Moluun);
        let mind = Mind {
            mood: MoodState::Happy,
            stats: VitalStats { hunger: 90.0, happiness: 70.0, energy: 80.0, health: 100.0 },
            age_ticks: 0,
            pending_hunger: 0.0, pending_happiness: 0.0, pending_energy: 0.0,
            mood_cooldown: 0,
        };
        assert_eq!(mind.fsm_mood(&genome), MoodState::Hungry);
    }
}
