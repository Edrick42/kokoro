//! Disease system — specific conditions with causes, durations, and cures.
//!
//! Replaces the generic "Sick" mood with targeted conditions:
//! - Cold: prolonged exposure to low temperature
//! - Parasite: low hygiene over time
//! - Malnutrition: sustained nutrient deficiency
//! - Exhaustion: prolonged low energy
//! - Infection: injury (broken bone) + low hygiene

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::disease as cfg;
use crate::creature::anatomy::AnatomyState;
use crate::game::state::AppState;
use crate::genome::Genome;
use crate::mind::hygiene::HygieneState;
use crate::mind::nutrition::NutrientState;
use crate::mind::{Mind, MoodState};
use crate::creature::species::CreatureRoot;

/// A specific disease condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Condition {
    Cold,
    Parasite,
    Malnutrition,
    Exhaustion,
    Infection,
}

impl Condition {
    pub fn label(&self) -> &'static str {
        match self {
            Condition::Cold         => "Cold",
            Condition::Parasite     => "Parasite",
            Condition::Malnutrition => "Malnutrition",
            Condition::Exhaustion   => "Exhaustion",
            Condition::Infection    => "Infection",
        }
    }

    fn base_duration(&self) -> u32 {
        match self {
            Condition::Cold         => cfg::duration::COLD,
            Condition::Parasite     => cfg::duration::PARASITE,
            Condition::Malnutrition => cfg::duration::MALNUTRITION,
            Condition::Exhaustion   => cfg::duration::EXHAUSTION,
            Condition::Infection    => cfg::duration::INFECTION,
        }
    }

    fn health_drain(&self) -> f32 {
        match self {
            Condition::Cold         => cfg::health_drain::COLD,
            Condition::Parasite     => cfg::health_drain::PARASITE,
            Condition::Malnutrition => cfg::health_drain::MALNUTRITION,
            Condition::Exhaustion   => cfg::health_drain::EXHAUSTION,
            Condition::Infection    => cfg::health_drain::INFECTION,
        }
    }
}

/// Active condition with remaining duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCondition {
    pub condition: Condition,
    pub remaining_ticks: u32,
}

/// Tracks all active disease conditions.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiseaseState {
    pub conditions: Vec<ActiveCondition>,
    /// Ticks of consecutive low energy (for exhaustion trigger).
    pub low_energy_ticks: u32,
}

impl DiseaseState {
    pub fn has(&self, condition: Condition) -> bool {
        self.conditions.iter().any(|c| c.condition == condition)
    }

    pub fn add(&mut self, condition: Condition, genome: &Genome) {
        if self.has(condition) { return; }
        // Resilience reduces duration
        let base = condition.base_duration();
        let reduction = (genome.resilience * cfg::RESILIENCE_RECOVERY_FACTOR * base as f32) as u32;
        self.conditions.push(ActiveCondition {
            condition,
            remaining_ticks: base.saturating_sub(reduction),
        });
    }

    pub fn is_sick(&self) -> bool {
        !self.conditions.is_empty()
    }

    /// Reduce duration of a specific condition (e.g., from healing herbs).
    pub fn heal(&mut self, condition: Condition, ticks: f32) {
        if let Some(c) = self.conditions.iter_mut().find(|c| c.condition == condition) {
            c.remaining_ticks = c.remaining_ticks.saturating_sub(ticks as u32);
        }
    }
}

pub struct DiseasePlugin;

impl Plugin for DiseasePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DiseaseState::default())
            .add_systems(Update, (
                disease_trigger_system,
                disease_tick_system,
            ).chain().run_if(in_state(AppState::Gameplay)));
    }
}

/// Checks conditions and triggers new diseases.
fn disease_trigger_system(
    mind: Res<Mind>,
    genome: Res<Genome>,
    hygiene: Option<Res<HygieneState>>,
    anatomy: Option<Res<AnatomyState>>,
    nutrient_q: Query<&NutrientState, With<CreatureRoot>>,
    mut disease: ResMut<DiseaseState>,
    mut reaction_events: EventWriter<crate::creature::reactions::CreatureReaction>,
) {
    let had_conditions = disease.conditions.len();

    // Parasite: low hygiene
    if let Some(ref hyg) = hygiene {
        if hyg.level < cfg::trigger::PARASITE_HYGIENE && !disease.has(Condition::Parasite) {
            disease.add(Condition::Parasite, &genome);
        }
    }

    // Exhaustion: prolonged low energy
    if mind.stats.energy < cfg::trigger::EXHAUSTION_ENERGY {
        disease.low_energy_ticks += 1;
        if disease.low_energy_ticks > cfg::trigger::EXHAUSTION_TICKS && !disease.has(Condition::Exhaustion) {
            disease.add(Condition::Exhaustion, &genome);
        }
    } else {
        disease.low_energy_ticks = 0;
    }

    // Malnutrition: sustained low nutrients
    if let Ok(nutrients) = nutrient_q.single() {
        if nutrients.average_fullness() < cfg::trigger::MALNUTRITION_FULLNESS && !disease.has(Condition::Malnutrition) {
            disease.add(Condition::Malnutrition, &genome);
        }
    }

    // Infection: broken bone + dirty
    if let (Some(ref anat), Some(ref hyg)) = (anatomy, hygiene) {
        let has_break = anat.skeleton.bones.iter().any(|b| b.integrity < 0.1);
        if has_break && hyg.level < 40.0 && !disease.has(Condition::Infection) {
            disease.add(Condition::Infection, &genome);
        }
    }

    // Fire visual reaction if new conditions appeared
    if disease.conditions.len() > had_conditions {
        reaction_events.write(crate::creature::reactions::CreatureReaction::GotSick);
    }
}

/// Ticks active diseases — applies health drain, removes expired conditions.
fn disease_tick_system(
    mut disease: ResMut<DiseaseState>,
    mut mind: ResMut<Mind>,
    ans: Option<Res<crate::mind::autonomic::AutonomicState>>,
    mut reaction_events: EventWriter<crate::creature::reactions::CreatureReaction>,
) {
    // ANS: parasympathetic rest speeds recovery; sympathetic stress slows it
    let calm = ans.as_ref().map(|a| a.calm_multiplier()).unwrap_or(1.0);

    // Apply health drain from each active condition
    for active in &disease.conditions {
        let drain = active.condition.health_drain();
        mind.stats.health = (mind.stats.health - drain).max(0.0);
    }

    // Tick down durations — parasympathetic accelerates recovery
    let recovery_ticks = if calm > 1.2 { 2 } else { 1 }; // heal 2x ticks when very calm
    let before = disease.conditions.len();
    disease.conditions.retain_mut(|c| {
        c.remaining_ticks = c.remaining_ticks.saturating_sub(recovery_ticks);
        c.remaining_ticks > 0
    });

    // Fire recovery reaction if conditions cleared
    if before > 0 && disease.conditions.is_empty() {
        reaction_events.write(crate::creature::reactions::CreatureReaction::Recovered);
    }

    // Override mood to Sick when conditions are active
    if disease.is_sick() && mind.mood != MoodState::Sleeping {
        mind.mood = MoodState::Sick;
    }
}
