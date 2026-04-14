//! Lifecycle system — aging, death, and care quality tracking.
//!
//! Every Kobara has a finite lifespan determined by species, genetics,
//! and quality of care. The creature ages naturally, and health degrades
//! in old age. Death occurs when health reaches zero.
//!
//! Causes of death:
//! - **Old age**: natural lifespan reached, health gradually declines
//! - **Starvation**: hunger at 100 for too long → health drops
//! - **Dehydration**: water nutrient at 0 → health drops rapidly
//! - **Neglect**: chronic low care quality shortens lifespan
//! - **Sickness**: health reaching 0 from any cause

use bevy::prelude::*;

use crate::game::state::AppState;
use crate::config::lifecycle as lc;
use crate::creature::identity::species::CreatureRoot;
use crate::genome::Genome;
use crate::mind::Mind;
use crate::mind::nutrition::NutrientState;

/// Tracks the creature's lifecycle state.
#[derive(Component, Debug, Clone)]
pub struct LifecycleState {
    /// Effective lifespan in ticks (adjusted by care quality).
    pub effective_lifespan: u64,
    /// Running care quality score (0.0 = neglected, 1.0 = perfect care).
    pub care_quality: f32,
    /// Consecutive ticks at health 0 (death grace period).
    pub zero_health_ticks: u32,
    /// Consecutive ticks at hunger 100 (starvation counter).
    pub starvation_ticks: u32,
    /// Consecutive ticks at thirst >= 99 (dehydration counter).
    pub dehydration_ticks: u32,
    /// Whether the creature is alive.
    pub alive: bool,
    /// Cause of death (if dead).
    pub cause_of_death: Option<DeathCause>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum DeathCause {
    OldAge,
    Starvation,
    Dehydration,
    Neglect,
    HealthFailure,
}

impl DeathCause {
    pub fn label(&self) -> &'static str {
        match self {
            DeathCause::OldAge         => "old age — a life well lived",
            DeathCause::Starvation     => "starvation",
            DeathCause::Dehydration    => "dehydration",
            DeathCause::Neglect        => "neglect",
            DeathCause::HealthFailure  => "health failure",
        }
    }
}

impl LifecycleState {
    pub fn new(species: &crate::genome::Species) -> Self {
        Self {
            effective_lifespan: lc::base_lifespan(species),
            care_quality: 0.5,
            zero_health_ticks: 0,
            starvation_ticks: 0,
            dehydration_ticks: 0,
            alive: true,
            cause_of_death: None,
        }
    }

    /// How far through its lifespan the creature is (0.0–1.0+).
    pub fn age_fraction(&self, age_ticks: u64) -> f32 {
        age_ticks as f32 / self.effective_lifespan as f32
    }

    /// Is the creature in old age? (aging effects active)
    pub fn is_elderly(&self, age_ticks: u64) -> bool {
        self.age_fraction(age_ticks) >= lc::AGING_START
    }
}

pub struct LifecyclePlugin;

impl Plugin for LifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DeathEvent>()
           .add_systems(Update, (
               aging_system,
               starvation_system,
               death_check_system,
               care_quality_system,
           ).chain().run_if(in_state(AppState::Gameplay)));
    }
}

/// Event fired when the creature dies.
#[derive(Event)]
#[allow(dead_code)]
pub struct DeathEvent {
    pub cause: DeathCause,
    pub age_ticks: u64,
    pub care_quality: f32,
}

/// Applies aging effects: health decay and energy cap reduction in old age.
fn aging_system(
    mut mind: ResMut<Mind>,
    lifecycle_q: Query<&LifecycleState, With<CreatureRoot>>,
) {
    let Ok(lifecycle) = lifecycle_q.single() else { return };
    if !lifecycle.alive { return; }

    if lifecycle.is_elderly(mind.age_ticks) {
        let age_frac = lifecycle.age_fraction(mind.age_ticks);
        let severity = (age_frac - lc::AGING_START) / (1.0 - lc::AGING_START);

        mind.stats.health = (mind.stats.health - lc::AGING_HEALTH_DECAY * severity).max(0.0);

        let energy_cap = 100.0 * (1.0 - lc::AGING_ENERGY_CAP_REDUCTION * severity.min(1.0));
        if mind.stats.energy > energy_cap {
            mind.stats.energy = energy_cap;
        }
    }
}

/// Tracks starvation and dehydration → health damage.
fn starvation_system(
    mut mind: ResMut<Mind>,
    mut lifecycle_q: Query<&mut LifecycleState, With<CreatureRoot>>,
    nutrient_q: Query<&NutrientState, With<CreatureRoot>>,
) {
    let Ok(mut lifecycle) = lifecycle_q.single_mut() else { return };
    if !lifecycle.alive { return; }

    // Starvation: hunger at max for too long
    if mind.stats.hunger >= 99.0 {
        lifecycle.starvation_ticks += 1;
        if lifecycle.starvation_ticks > lc::STARVATION_THRESHOLD_TICKS {
            mind.stats.health = (mind.stats.health - lc::STARVATION_HEALTH_DECAY).max(0.0);
        }
    } else {
        lifecycle.starvation_ticks = 0;
    }

    // Dehydration: thirst at max for too long
    if mind.stats.thirst >= 99.0 {
        lifecycle.dehydration_ticks += 1;
        if lifecycle.dehydration_ticks > lc::STARVATION_THRESHOLD_TICKS {
            mind.stats.health = (mind.stats.health - lc::DEHYDRATION_HEALTH_DECAY).max(0.0);
        }
    } else {
        lifecycle.dehydration_ticks = 0;
    }

    // Dehydration: water nutrient at 0
    if let Ok(nutrients) = nutrient_q.single() {
        if nutrients.water < 1.0 {
            mind.stats.health = (mind.stats.health - lc::DEHYDRATION_HEALTH_DECAY).max(0.0);
        }
    }
}

/// Checks if the creature should die.
fn death_check_system(
    mind: Res<Mind>,
    _genome: Res<Genome>,
    mut lifecycle_q: Query<&mut LifecycleState, With<CreatureRoot>>,
    mut death_events: EventWriter<DeathEvent>,
) {
    let Ok(mut lifecycle) = lifecycle_q.single_mut() else { return };
    if !lifecycle.alive { return; }

    // Health at zero → grace period then death
    if mind.stats.health <= lc::DEATH_HEALTH_THRESHOLD {
        lifecycle.zero_health_ticks += 1;

        if lifecycle.zero_health_ticks >= lc::DEATH_GRACE_TICKS {
            // Determine cause of death
            let cause = if lifecycle.age_fraction(mind.age_ticks) >= 1.0 {
                DeathCause::OldAge
            } else if lifecycle.starvation_ticks > lc::STARVATION_THRESHOLD_TICKS {
                DeathCause::Starvation
            } else if lifecycle.dehydration_ticks > lc::STARVATION_THRESHOLD_TICKS {
                DeathCause::Dehydration
            } else if lifecycle.care_quality < 0.2 {
                DeathCause::Neglect
            } else {
                DeathCause::HealthFailure
            };

            lifecycle.alive = false;
            lifecycle.cause_of_death = Some(cause.clone());

            info!(
                "Kobara has died. Cause: {}. Age: {} ticks. Care quality: {:.0}%",
                cause.label(), mind.age_ticks, lifecycle.care_quality * 100.0
            );

            death_events.write(DeathEvent {
                cause,
                age_ticks: mind.age_ticks,
                care_quality: lifecycle.care_quality,
            });
        }
    } else {
        lifecycle.zero_health_ticks = 0;
    }

    // Natural death from extreme old age (150% of lifespan = guaranteed death)
    if lifecycle.age_fraction(mind.age_ticks) >= 1.5 {
        lifecycle.alive = false;
        lifecycle.cause_of_death = Some(DeathCause::OldAge);

        info!("Kobara has passed peacefully of old age. {} ticks lived.", mind.age_ticks);
        death_events.write(DeathEvent {
            cause: DeathCause::OldAge,
            age_ticks: mind.age_ticks,
            care_quality: lifecycle.care_quality,
        });
    }
}

/// Updates care quality score based on creature well-being.
fn care_quality_system(
    mind: Res<Mind>,
    nutrient_q: Query<&NutrientState, With<CreatureRoot>>,
    mut lifecycle_q: Query<&mut LifecycleState, With<CreatureRoot>>,
    genome: Res<Genome>,
) {
    if mind.age_ticks % lc::CARE_CHECK_INTERVAL != 0 { return; }

    let Ok(mut lifecycle) = lifecycle_q.single_mut() else { return };
    if !lifecycle.alive { return; }

    // Calculate care quality from current stats
    let health_score = mind.stats.health / 100.0;
    let happiness_score = mind.stats.happiness / 100.0;
    let energy_score = mind.stats.energy / 100.0;
    let hunger_score = 1.0 - (mind.stats.hunger / 100.0); // lower hunger = better

    let nutrient_score = if let Ok(nutrients) = nutrient_q.single() {
        nutrients.average_fullness() / 100.0
    } else {
        0.5
    };

    let current_quality = (health_score + happiness_score + energy_score + hunger_score + nutrient_score) / 5.0;

    // Exponential moving average
    lifecycle.care_quality = lifecycle.care_quality * 0.95 + current_quality * 0.05;

    // Adjust effective lifespan based on care quality
    let base = lc::base_lifespan(&genome.species) as f32;
    let bonus = if lifecycle.care_quality > 0.7 {
        (lifecycle.care_quality - 0.7) / 0.3 * lc::MAX_CARE_BONUS
    } else if lifecycle.care_quality < 0.3 {
        -((0.3 - lifecycle.care_quality) / 0.3 * lc::MAX_NEGLECT_PENALTY)
    } else {
        0.0
    };
    lifecycle.effective_lifespan = (base * (1.0 + bonus)) as u64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::Species;

    #[test]
    fn lifecycle_starts_alive() {
        let lc = LifecycleState::new(&Species::Moluun);
        assert!(lc.alive);
        assert!(lc.cause_of_death.is_none());
    }

    #[test]
    fn elderly_detection() {
        let lc = LifecycleState::new(&Species::Moluun);
        let lifespan = lc.effective_lifespan;
        assert!(!lc.is_elderly(0));
        assert!(!lc.is_elderly(lifespan / 2));
        assert!(lc.is_elderly((lifespan as f32 * 0.75) as u64));
    }

    #[test]
    fn age_fraction_calculation() {
        let lc = LifecycleState::new(&Species::Moluun);
        let lifespan = lc.effective_lifespan;
        assert!((lc.age_fraction(0) - 0.0).abs() < 0.01);
        assert!((lc.age_fraction(lifespan) - 1.0).abs() < 0.01);
        assert!((lc.age_fraction(lifespan / 2) - 0.5).abs() < 0.01);
    }

    #[test]
    fn death_cause_labels() {
        assert_eq!(DeathCause::OldAge.label(), "old age — a life well lived");
        assert_eq!(DeathCause::Starvation.label(), "starvation");
    }
}
