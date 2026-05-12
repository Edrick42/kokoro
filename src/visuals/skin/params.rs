//! SkinParams — bridge between physiology data and visual rendering.
//!
//! Translates bone density, muscle mass, fat level, joint stiffness,
//! and skin integrity into visual parameters that the drawing functions use.

use crate::creature::physiology::PhysiologyState;
use crate::creature::physiology::bone_health::BoneStructure;
use crate::genome::Species;
use crate::visuals::evolution::GrowthStage;

/// Visual parameters derived from physiology. Passed to every draw function.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SkinParams {
    /// Body radius multiplier (1.0 = baseline). Driven by density × mass × fat.
    pub bulk: f32,
    /// Belly roundness (0.0 = flat, 1.0 = very round). Driven by fat level.
    pub belly: f32,
    /// Limb thickness multiplier (1.0 = normal). Driven by muscle strength.
    pub limb_bulk: f32,
    /// Posture sag (0.0 = upright, 1.0 = drooping). Driven by fatigue + stiffness.
    pub sag: f32,
    /// Armor coverage (0.0 = smooth, 1.0 = fully plated). Skael only, age-driven.
    pub armor: f32,
    /// Bioluminescence intensity (0.0 = dim, 1.0 = bright). Nyxal, health-driven.
    pub glow: f32,
    /// Skin damage (0.0 = pristine, 1.0 = heavily scarred).
    pub damage: f32,
    /// Fur/plumage/scale density (0.0 = patchy, 1.0 = full). Health + hydration.
    pub covering_density: f32,
    /// Overall health color modifier (1.0 = vibrant, 0.0 = pale/washed out).
    pub vitality: f32,
}

impl SkinParams {
    /// Compute visual parameters from live physiology data.
    pub fn from_anatomy(physiology: &PhysiologyState, species: &Species, stage: &GrowthStage) -> Self {
        let avg_strength = if physiology.muscle_health.groups.is_empty() {
            1.0
        } else {
            physiology.muscle_health.groups.iter().map(|m| m.strength).sum::<f32>()
                / physiology.muscle_health.groups.len() as f32
        };

        let avg_fatigue = if physiology.muscle_health.groups.is_empty() {
            0.0
        } else {
            physiology.muscle_health.groups.iter().map(|m| m.fatigue).sum::<f32>()
                / physiology.muscle_health.groups.len() as f32
        };

        let stiffness = 1.0 - physiology.avg_flexibility();

        // Bulk: bone density provides the frame, muscle mass fills it, fat rounds it
        let density_factor = match physiology.bone_health.structure {
            BoneStructure::Hydrostatic => physiology.bone_health.hydrostatic_pressure,
            _ => physiology.bone_health.density,
        };
        let bulk = 0.7 + density_factor * 0.15 + physiology.muscle_health.mass * 0.1 + physiology.fat.level * 0.15;

        // Belly: primarily fat, slightly affected by muscle condition (low condition = sagging)
        let belly = (physiology.fat.level * 0.7 + (1.0 - physiology.muscle_health.condition) * 0.3).clamp(0.0, 1.0);

        // Limb thickness: muscle strength and mass
        let limb_bulk = 0.6 + avg_strength * 0.25 + physiology.muscle_health.mass * 0.15;

        // Posture sag: fatigue + joint stiffness + low energy
        let sag = (avg_fatigue * 0.5 + stiffness * 0.3 + (1.0 - physiology.muscle_health.condition) * 0.2).clamp(0.0, 1.0);

        // Armor: Skael-specific, grows with age. Other species = 0.
        let armor = match species {
            Species::Skael => match stage {
                GrowthStage::Egg   => 0.0,
                GrowthStage::Cub   => 0.0,
                GrowthStage::Young => 0.3,
                GrowthStage::Adult => 0.8,
                GrowthStage::Elder => 1.0,
            },
            _ => 0.0,
        };

        // Glow: Nyxal bioluminescence, proportional to health and vitality.
        let glow = match species {
            Species::Nyxal => {
                let bone_health = physiology.avg_bone_integrity();
                (bone_health * 0.5 + physiology.skin_health.integrity * 0.3 + physiology.muscle_health.condition * 0.2).clamp(0.0, 1.0)
            }
            _ => 0.0,
        };

        // Skin damage: inverse of skin integrity
        let damage = (1.0 - physiology.skin_health.integrity).clamp(0.0, 1.0);

        // Covering density: skin hydration + integrity
        let covering_density = (physiology.skin_health.hydration * 0.5 + physiology.skin_health.integrity * 0.5).clamp(0.0, 1.0);

        // Vitality: overall "aliveness" look — pale when sick, vibrant when healthy
        let vitality = (physiology.bone_health.overall * 0.3
            + physiology.muscle_health.condition * 0.3
            + physiology.skin_health.integrity * 0.2
            + (1.0 - avg_fatigue) * 0.2)
            .clamp(0.0, 1.0);

        Self {
            bulk,
            belly,
            limb_bulk,
            sag,
            armor,
            glow,
            damage,
            covering_density,
            vitality,
        }
    }

    /// Default params when no physiology data is available (fallback).
    pub fn healthy_default() -> Self {
        Self {
            bulk: 1.0,
            belly: 0.5,
            limb_bulk: 1.0,
            sag: 0.0,
            armor: 0.0,
            glow: 0.8,
            damage: 0.0,
            covering_density: 1.0,
            vitality: 1.0,
        }
    }
}
