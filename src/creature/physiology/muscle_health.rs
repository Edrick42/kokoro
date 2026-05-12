//! Muscle health — slow-biology state of muscle force and conditioning.
//!
//! Distinct from `kokoro_body::Muscle` (per-frame force producer): this
//! struct tracks long-term mass, conditioning, fatigue, and recovery.
//! Atrophied muscles drain energy; well-conditioned ones are efficient.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscleHealth {
    /// Overall muscle mass (0.0-1.0). Affects nutrient demand and strength.
    pub mass: f32,
    /// Current condition (0.0 = atrophied, 1.0 = peak).
    pub condition: f32,
    /// Energy efficiency factor — slowly converges toward condition.
    pub tone: f32,
    /// Per-group conditioning records, each actuating a specific joint.
    pub groups: Vec<MuscleRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscleRecord {
    /// Muscle group name (e.g. "core", "pectorals", "legs_left").
    pub name: String,
    /// Which joint this muscle actuates.
    pub joint: String,
    /// 0.0 = no force, 1.0 = full strength.
    pub strength: f32,
    /// 0.0 = fresh, 1.0 = exhausted. Recovers during sleep.
    pub fatigue: f32,
}

/// Helper to create a muscle record at full strength, no fatigue.
pub fn muscle_record(name: &str, joint: &str) -> MuscleRecord {
    MuscleRecord {
        name: name.to_string(),
        joint: joint.to_string(),
        strength: 1.0,
        fatigue: 0.0,
    }
}
