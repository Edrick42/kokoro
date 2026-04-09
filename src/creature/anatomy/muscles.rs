//! Muscles — force and movement.
//!
//! Muscle mass determines nutrient demand and force output.
//! Well-conditioned muscles are energy-efficient. Atrophied muscles
//! from neglect or malnutrition drain energy rapidly.
//! Muscles fatigue during waking hours and recover during sleep.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscleSystem {
    /// Overall muscle mass (0.0-1.0). Affects nutrient demand and strength.
    pub mass: f32,
    /// Current condition (0.0 = atrophied, 1.0 = peak).
    pub condition: f32,
    /// Energy efficiency factor — slowly converges toward condition.
    pub tone: f32,
    /// Individual muscle groups, each actuating a specific joint.
    pub groups: Vec<MuscleGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscleGroup {
    /// Muscle group name (e.g. "core", "pectorals", "legs_left").
    pub name: String,
    /// Which joint this muscle actuates.
    pub joint: String,
    /// 0.0 = no force, 1.0 = full strength.
    pub strength: f32,
    /// 0.0 = fresh, 1.0 = exhausted. Recovers during sleep.
    pub fatigue: f32,
}

/// Helper to create a muscle group at full strength, no fatigue.
pub fn muscle(name: &str, joint: &str) -> MuscleGroup {
    MuscleGroup {
        name: name.to_string(),
        joint: joint.to_string(),
        strength: 1.0,
        fatigue: 0.0,
    }
}
