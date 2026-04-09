//! Joints — connections between bones (or flex points for boneless Nyxal).
//!
//! Flexibility determines range of motion and whether the creature can play.
//! Lubrication determines movement smoothness — dry joints cost extra energy.
//! Elders lose flexibility irreversibly as joint tissue calcifies.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointSystem {
    pub joints: Vec<Joint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    /// Joint name (e.g. "neck", "shoulder_left", "hip_right").
    pub name: String,
    /// First connected bone (or body segment for Nyxal).
    pub bone_a: String,
    /// Second connected bone.
    pub bone_b: String,
    /// 0.0 = locked, 1.0 = fully mobile.
    pub flexibility: f32,
    /// 0.0 = dry/painful, 1.0 = smooth.
    pub lubrication: f32,
    /// 0.0 = destroyed, 1.0 = perfect.
    pub integrity: f32,
}

/// Helper to create a joint at full lubrication and integrity.
pub fn joint(name: &str, bone_a: &str, bone_b: &str, flexibility: f32) -> Joint {
    Joint {
        name: name.to_string(),
        bone_a: bone_a.to_string(),
        bone_b: bone_b.to_string(),
        flexibility: flexibility.clamp(0.0, 1.0),
        lubrication: 1.0,
        integrity: 1.0,
    }
}
