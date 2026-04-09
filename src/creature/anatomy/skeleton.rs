//! Skeleton — the structural frame of every Kobara.
//!
//! Bone density and integrity determine the creature's maximum health ceiling.
//! Species archetype matters:
//! - **Standard** (Moluun) — normal mammalian skeleton
//! - **Hollow** (Pylum) — light, fragile, flight-optimized (1.5× damage)
//! - **Dense** (Skael) — heavy, durable, impact-resistant (0.6× damage)
//! - **Hydrostatic** (Nyxal) — no bones, water pressure provides structure

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skeleton {
    pub structure_type: SkeletonType,
    /// Overall bone density (0.0-1.0). Species base modified by resilience gene.
    pub bone_density: f32,
    /// Overall bone health (0.0-1.0). Degrades from mineral/protein deficiency.
    pub bone_health: f32,
    /// Nyxal only: replaces bone density for boneless creatures.
    #[serde(default)]
    pub hydrostatic_pressure: f32,
    /// Individual bones with per-bone integrity.
    pub bones: Vec<Bone>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkeletonType {
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
pub struct Bone {
    /// Bone name (e.g. "skull", "spine", "ribcage").
    pub name: String,
    /// Maps to a body part slot in the rig system.
    pub slot: String,
    /// 0.0 = broken, 1.0 = perfect.
    pub integrity: f32,
    /// Relative to skeleton density (skull=1.2, limbs=0.8).
    pub density_modifier: f32,
}

/// Helper to create a bone with full integrity.
pub fn bone(name: &str, slot: &str, density_modifier: f32) -> Bone {
    Bone {
        name: name.to_string(),
        slot: slot.to_string(),
        integrity: 1.0,
        density_modifier,
    }
}
