//! Anatomy system — skeleton, muscles, joints, skin.
//!
//! Four interconnected layers that work like real biology:
//! - **Skeleton** → structural frame, health ceiling
//! - **Joints** → connections, mobility, playfulness
//! - **Muscles** → force, efficiency, fatigue/recovery
//! - **Skin** → protection, sensation, species covering
//!
//! Damage cascades through layers: broken bone → joint locks →
//! muscle weakens → energy costs rise → creature suffers.
//! Good care reverses the cycle gradually.

pub mod fat;
pub mod skeleton;
pub mod muscles;
pub mod joints;
pub mod skin;
mod species;
mod tick;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::state::AppState;

use crate::config::anatomy as cfg;
pub use self::fat::FatReserve;
use self::skeleton::{Skeleton, SkeletonType};
use self::muscles::MuscleSystem;
use self::joints::JointSystem;
use self::skin::SkinLayer;

/// Complete anatomy state for one creature.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct AnatomyState {
    pub skeleton: Skeleton,
    pub muscles: MuscleSystem,
    pub joints: JointSystem,
    pub skin: SkinLayer,
    pub fat: FatReserve,
}

/// Bevy plugin — registers the anatomy tick system.
pub struct AnatomyPlugin;

impl Plugin for AnatomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tick::anatomy_tick_system.run_if(in_state(AppState::Gameplay)));
    }
}

// ---------------------------------------------------------------------------
// Aggregate helpers
// ---------------------------------------------------------------------------

impl AnatomyState {
    /// Average bone integrity across all bones (hydrostatic pressure for Nyxal).
    pub fn avg_bone_integrity(&self) -> f32 {
        if self.skeleton.bones.is_empty() {
            return self.skeleton.hydrostatic_pressure;
        }
        let sum: f32 = self.skeleton.bones.iter().map(|b| b.integrity).sum();
        sum / self.skeleton.bones.len() as f32
    }

    /// Maximum health ceiling based on skeletal health.
    pub fn health_ceiling(&self) -> f32 {
        let structural = if self.skeleton.structure_type == SkeletonType::Hydrostatic {
            self.skeleton.hydrostatic_pressure
        } else {
            self.skeleton.bone_health * self.avg_bone_integrity()
        };
        structural * cfg::skeleton::HEALTH_CEILING_MULTIPLIER
    }

    /// Average joint lubrication.
    pub fn avg_lubrication(&self) -> f32 {
        if self.joints.joints.is_empty() { return 1.0; }
        let sum: f32 = self.joints.joints.iter().map(|j| j.lubrication).sum();
        sum / self.joints.joints.len() as f32
    }

    /// Average joint flexibility.
    pub fn avg_flexibility(&self) -> f32 {
        if self.joints.joints.is_empty() { return 1.0; }
        let sum: f32 = self.joints.joints.iter().map(|j| j.flexibility).sum();
        sum / self.joints.joints.len() as f32
    }

    /// Average muscle fatigue.
    #[allow(dead_code)]
    pub fn avg_fatigue(&self) -> f32 {
        if self.muscles.groups.is_empty() { return 0.0; }
        let sum: f32 = self.muscles.groups.iter().map(|m| m.fatigue).sum();
        sum / self.muscles.groups.len() as f32
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::anatomy as cfg;
    use crate::genome::{Genome, Species};
    use crate::mind::Mind;
    use crate::mind::MoodState;

    fn test_genome(species: Species) -> Genome {
        Genome::random_for(species)
    }

    #[test]
    fn moluun_anatomy_counts() {
        let g = test_genome(Species::Moluun);
        let a = AnatomyState::new_for(&Species::Moluun, &g);
        assert_eq!(a.skeleton.bones.len(), 6);
        assert_eq!(a.joints.joints.len(), 5);
        assert_eq!(a.muscles.groups.len(), 4);
        assert_eq!(a.skeleton.structure_type, SkeletonType::Standard);
        assert_eq!(a.skin.covering, skin::SkinCovering::Fur);
    }

    #[test]
    fn pylum_anatomy_counts() {
        let g = test_genome(Species::Pylum);
        let a = AnatomyState::new_for(&Species::Pylum, &g);
        assert_eq!(a.skeleton.bones.len(), 7);
        assert_eq!(a.joints.joints.len(), 5);
        assert_eq!(a.muscles.groups.len(), 4);
        assert_eq!(a.skeleton.structure_type, SkeletonType::Hollow);
        assert_eq!(a.skin.covering, skin::SkinCovering::Plumage);
    }

    #[test]
    fn skael_anatomy_counts() {
        let g = test_genome(Species::Skael);
        let a = AnatomyState::new_for(&Species::Skael, &g);
        assert_eq!(a.skeleton.bones.len(), 7);
        assert_eq!(a.joints.joints.len(), 6);
        assert_eq!(a.muscles.groups.len(), 5);
        assert_eq!(a.skeleton.structure_type, SkeletonType::Dense);
        assert_eq!(a.skin.covering, skin::SkinCovering::Scales);
    }

    #[test]
    fn nyxal_has_no_bones() {
        let g = test_genome(Species::Nyxal);
        let a = AnatomyState::new_for(&Species::Nyxal, &g);
        assert_eq!(a.skeleton.bones.len(), 0);
        assert_eq!(a.skeleton.structure_type, SkeletonType::Hydrostatic);
        assert!(a.skeleton.hydrostatic_pressure > 0.0);
        assert_eq!(a.skin.covering, skin::SkinCovering::Membrane);
    }

    #[test]
    fn resilience_increases_bone_density() {
        let mut g_low = Genome::random_for(Species::Moluun);
        g_low.resilience = 0.0;
        let mut g_high = Genome::random_for(Species::Moluun);
        g_high.resilience = 1.0;

        let a_low = AnatomyState::new_for(&Species::Moluun, &g_low);
        let a_high = AnatomyState::new_for(&Species::Moluun, &g_high);

        assert!(a_high.skeleton.bone_density > a_low.skeleton.bone_density,
            "high resilience should produce higher bone density");
    }

    #[test]
    fn health_ceiling_based_on_bones() {
        let g = test_genome(Species::Moluun);
        let mut a = AnatomyState::new_for(&Species::Moluun, &g);

        let ceiling_full = a.health_ceiling();
        assert!(ceiling_full > 90.0, "full bone health ceiling = {}", ceiling_full);

        a.skeleton.bone_health = 0.5;
        let ceiling_half = a.health_ceiling();
        assert!(ceiling_half < 60.0, "half bone health ceiling = {}", ceiling_half);
    }

    #[test]
    fn nyxal_health_ceiling_based_on_pressure() {
        let g = test_genome(Species::Nyxal);
        let mut a = AnatomyState::new_for(&Species::Nyxal, &g);

        let ceiling_full = a.health_ceiling();
        assert!(ceiling_full > 70.0);

        a.skeleton.hydrostatic_pressure = 0.3;
        let ceiling_low = a.health_ceiling();
        assert!(ceiling_low < 40.0, "low pressure ceiling = {}", ceiling_low);
    }

    #[test]
    fn bone_break_cascade() {
        let g = test_genome(Species::Moluun);
        let mut a = AnatomyState::new_for(&Species::Moluun, &g);
        let mut mind = Mind::new();

        if let Some(spine) = a.skeleton.bones.iter_mut().find(|b| b.name == "spine") {
            spine.integrity = 0.0;
        }

        tick::check_bone_breaks(&mut a, &mut mind);

        let neck = a.joints.joints.iter().find(|j| j.name == "neck").unwrap();
        assert!(neck.flexibility <= cfg::skeleton::BREAK_JOINT_FLEX_MIN + 0.01);

        let core = a.muscles.groups.iter().find(|m| m.name == "core").unwrap();
        assert!(core.strength < 1.0);

        assert_eq!(mind.mood, MoodState::Sick);
        assert!(mind.stats.health < 100.0);
    }

    #[test]
    fn all_species_start_healthy() {
        let species = [Species::Moluun, Species::Pylum, Species::Skael, Species::Nyxal];
        for sp in &species {
            let g = Genome::random_for(sp.clone());
            let a = AnatomyState::new_for(sp, &g);
            assert_eq!(a.skeleton.bone_health, 1.0);
            assert_eq!(a.muscles.condition, 1.0);
            assert_eq!(a.skin.integrity, 1.0);
            assert_eq!(a.skin.hydration, 1.0);
            for bone in &a.skeleton.bones {
                assert_eq!(bone.integrity, 1.0);
            }
            for joint in &a.joints.joints {
                assert_eq!(joint.lubrication, 1.0);
                assert_eq!(joint.integrity, 1.0);
            }
            for group in &a.muscles.groups {
                assert_eq!(group.strength, 1.0);
                assert_eq!(group.fatigue, 0.0);
            }
        }
    }

    #[test]
    fn avg_helpers_work() {
        let g = test_genome(Species::Moluun);
        let a = AnatomyState::new_for(&Species::Moluun, &g);
        assert!((a.avg_bone_integrity() - 1.0).abs() < 0.01);
        assert!((a.avg_lubrication() - 1.0).abs() < 0.01);
        assert!(a.avg_flexibility() > 0.0 && a.avg_flexibility() <= 1.0);
        assert!((a.avg_fatigue() - 0.0).abs() < 0.01);
    }
}
