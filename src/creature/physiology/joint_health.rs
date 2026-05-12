//! Joint health — slow-biology state of articulation surfaces.
//!
//! Distinct from `kokoro_rig::Joint` (per-frame articulator): this struct
//! tracks flexibility, lubrication, and integrity that decay over time.
//! Elders lose flexibility irreversibly as joint tissue calcifies.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointHealth {
    pub joints: Vec<JointRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointRecord {
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

/// Helper to create a joint record at full lubrication and integrity.
pub fn joint_record(name: &str, bone_a: &str, bone_b: &str, flexibility: f32) -> JointRecord {
    JointRecord {
        name: name.to_string(),
        bone_a: bone_a.to_string(),
        bone_b: bone_b.to_string(),
        flexibility: flexibility.clamp(0.0, 1.0),
        lubrication: 1.0,
        integrity: 1.0,
    }
}
