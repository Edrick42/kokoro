//! Bone health — slow-biology state of the skeletal frame.
//!
//! Distinct from `kokoro_rig::Skeleton` (per-frame physics): this struct
//! tracks density, integrity, and species archetype for damage cascades.
//! - **Standard** (Moluun) — normal mammalian skeleton
//! - **Hollow** (Pylum) — light, fragile, flight-optimized (1.5× damage)
//! - **Dense** (Skael) — heavy, durable, impact-resistant (0.6× damage)
//! - **Hydrostatic** (Nyxal) — no bones, water pressure provides structure

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneHealth {
    pub structure: BoneStructure,
    /// Overall bone density (0.0-1.0). Species base modified by resilience gene.
    pub density: f32,
    /// Overall bone health (0.0-1.0). Degrades from mineral/protein deficiency.
    pub overall: f32,
    /// Nyxal only: replaces bone density for boneless creatures.
    #[serde(default)]
    pub hydrostatic_pressure: f32,
    /// Per-bone integrity records.
    pub bones: Vec<BoneRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoneStructure {
    /// Moluun — elastic mammalian skeleton.
    Standard,
    /// Pylum — air-filled, lightweight, fragile.
    Hollow,
    /// Skael — mineral-reinforced, heavy, durable.
    Dense,
    /// Nyxal — no bones; pressurized water chambers.
    Hydrostatic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneRecord {
    /// Bone name (e.g. "skull", "spine", "ribcage").
    pub name: String,
    /// Maps to a body part slot in the rig system.
    pub slot: String,
    /// 0.0 = broken, 1.0 = perfect.
    pub integrity: f32,
    /// Relative to skeleton density (skull=1.2, limbs=0.8).
    pub density_modifier: f32,
}

/// Helper to create a bone record with full integrity.
pub fn bone_record(name: &str, slot: &str, density_modifier: f32) -> BoneRecord {
    BoneRecord {
        name: name.to_string(),
        slot: slot.to_string(),
        integrity: 1.0,
        density_modifier,
    }
}
