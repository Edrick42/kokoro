//! Skeletal Pose System — joint angles driven by anatomy.
//!
//! The pose layer sits between anatomy and skin rendering:
//! ```
//! AnatomyState → PoseState → SkinRenderer
//! ```
//!
//! Each joint has a current angle that blends toward a target angle.
//! Animation speed is driven by muscle strength, fatigue, and joint lubrication.
//! Joint flexibility constrains the maximum angle (elder creatures move less).
//! Broken bones lock their joints at 0°.

use std::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(dead_code)] // APIs will be used when reactions are wired
use crate::creature::anatomy::AnatomyState;
use crate::game::state::AppState;

/// Current pose — one angle per joint, blending toward targets.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct PoseState {
    /// Current joint angles in degrees (joint_name → angle).
    pub angles: HashMap<String, f32>,
    /// Target angles to animate toward.
    pub targets: HashMap<String, f32>,
}

impl Default for PoseState {
    fn default() -> Self {
        Self {
            angles: HashMap::new(),
            targets: HashMap::new(),
        }
    }
}

impl PoseState {
    /// Initialize pose from anatomy joints (all angles start at 0).
    pub fn from_anatomy(anatomy: &AnatomyState) -> Self {
        let mut angles = HashMap::new();
        for joint in &anatomy.joints.joints {
            angles.insert(joint.name.clone(), 0.0);
        }
        Self {
            targets: angles.clone(),
            angles,
        }
    }

    /// Set target pose (all joints). Unspecified joints target 0 (neutral).
    pub fn set_targets(&mut self, targets: HashMap<String, f32>) {
        // Reset all targets to neutral first
        for val in self.targets.values_mut() {
            *val = 0.0;
        }
        // Apply specific targets
        for (joint, angle) in targets {
            self.targets.insert(joint, angle);
        }
    }

    /// Reset all targets to neutral (resting pose).
    pub fn reset_to_neutral(&mut self) {
        for val in self.targets.values_mut() {
            *val = 0.0;
        }
    }

    /// Get current angle for a joint (0.0 if not found).
    pub fn angle(&self, joint: &str) -> f32 {
        self.angles.get(joint).copied().unwrap_or(0.0)
    }
}

/// A keyframed animation — sequence of target poses with hold durations.
#[derive(Debug, Clone)]
pub struct PoseAnimation {
    /// (target_angles, hold_ticks) per keyframe.
    pub keyframes: Vec<(HashMap<String, f32>, u32)>,
    /// Which keyframe we're on.
    pub current_frame: usize,
    /// Ticks remaining on current keyframe's hold.
    pub hold_remaining: u32,
}

impl PoseAnimation {
    pub fn new(keyframes: Vec<(HashMap<String, f32>, u32)>) -> Self {
        let hold = keyframes.first().map(|(_, h)| *h).unwrap_or(0);
        Self {
            keyframes,
            current_frame: 0,
            hold_remaining: hold,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current_frame >= self.keyframes.len()
    }

    /// Advance and return current target angles (or None if finished).
    pub fn tick(&mut self) -> Option<&HashMap<String, f32>> {
        if self.is_finished() { return None; }

        if self.hold_remaining > 0 {
            self.hold_remaining -= 1;
            return Some(&self.keyframes[self.current_frame].0);
        }

        // Move to next keyframe
        self.current_frame += 1;
        if self.is_finished() { return None; }

        self.hold_remaining = self.keyframes[self.current_frame].1;
        Some(&self.keyframes[self.current_frame].0)
    }
}

/// Resource tracking the active animation (if any).
#[derive(Resource, Default)]
pub struct ActiveAnimation {
    pub animation: Option<PoseAnimation>,
}

pub struct PosePlugin;

impl Plugin for PosePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PoseState::default())
            .insert_resource(ActiveAnimation::default())
            .add_systems(Update, (
                init_pose_system,
                animation_tick_system,
                pose_blend_system,
            ).chain().run_if(in_state(AppState::Gameplay)));
    }
}

/// Initialize pose state from anatomy when it first becomes available.
fn init_pose_system(
    anatomy: Option<Res<AnatomyState>>,
    mut pose: ResMut<PoseState>,
) {
    if pose.angles.is_empty() {
        if let Some(ref anat) = anatomy {
            *pose = PoseState::from_anatomy(anat);
        }
    }
}

/// Advance active animation and push target angles to PoseState.
fn animation_tick_system(
    mut active: ResMut<ActiveAnimation>,
    mut pose: ResMut<PoseState>,
) {
    let Some(ref mut anim) = active.animation else { return };

    if let Some(targets) = anim.tick() {
        pose.set_targets(targets.clone());
    } else {
        // Animation finished — return to neutral
        pose.reset_to_neutral();
        active.animation = None;
    }
}

/// Blend current angles toward targets, constrained by anatomy.
fn pose_blend_system(
    anatomy: Option<Res<AnatomyState>>,
    mut pose: ResMut<PoseState>,
) {
    let Some(ref anat) = anatomy else { return };

    for joint_data in &anat.joints.joints {
        let name = &joint_data.name;
        let current = pose.angles.get(name).copied().unwrap_or(0.0);
        let target = pose.targets.get(name).copied().unwrap_or(0.0);

        if (current - target).abs() < 0.1 {
            pose.angles.insert(name.clone(), target);
            continue;
        }

        // Max angle constrained by flexibility
        let max_angle = 90.0 * joint_data.flexibility;
        let clamped_target = target.clamp(-max_angle, max_angle);

        // Locked joint (broken bone) — force to 0
        if joint_data.integrity < 0.1 {
            pose.angles.insert(name.clone(), 0.0);
            continue;
        }

        // Blend speed from muscle strength + lubrication
        let muscle = anat.muscles.groups.iter()
            .find(|m| m.joint == *name);
        let strength = muscle.map(|m| m.strength).unwrap_or(1.0);
        let fatigue = muscle.map(|m| m.fatigue).unwrap_or(0.0);
        let lubrication = joint_data.lubrication;

        let speed = 5.0 * strength * (1.0 - fatigue * 0.5) * lubrication;

        // Blend toward target
        let diff = clamped_target - current;
        let step = diff.signum() * speed.min(diff.abs());
        pose.angles.insert(name.clone(), current + step);
    }
}

/// Helper: create a target angle map from pairs.
pub fn pose_from(pairs: &[(&str, f32)]) -> HashMap<String, f32> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}
