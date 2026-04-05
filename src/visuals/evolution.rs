//! Visual evolution — the creature changes appearance as it ages.
//!
//! ## Growth stages
//!
//! | Stage    | Age (ticks) | Scale | Visual changes           |
//! |----------|-------------|-------|--------------------------|
//! | Baby     | 0–500       | 0.6   | Small, extra round       |
//! | Child    | 500–2000    | 0.8   | Growing, slight stretch  |
//! | Adult    | 2000–10000  | 1.0   | Full size                |
//! | Elder    | 10000+      | 0.95  | Slightly smaller, wisdom |
//!
//! The transition between stages is smoothly interpolated over ~100 ticks
//! so the creature doesn't suddenly jump in size.
//!
//! ## Future additions
//! - Accessories that appear at milestones (scarf at 5000 ticks, crown at 20000)
//! - Battle scars or marks from sickness events
//! - Color shifts as the creature matures

use bevy::prelude::*;
use crate::mind::Mind;
use crate::creature::species::CreatureRoot;

/// Tracks the creature's current growth stage for visual evolution.
#[derive(Resource)]
pub struct GrowthState {
    pub stage: GrowthStage,
    /// Current visual scale (smoothly interpolated)
    pub current_scale: f32,
    /// Target scale for the current stage
    pub target_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GrowthStage {
    Baby,
    Child,
    Adult,
    Elder,
}

impl GrowthStage {
    pub fn from_age_pub(age: u64) -> Self {
        Self::from_age(age)
    }

    fn from_age(age: u64) -> Self {
        match age {
            0..500      => GrowthStage::Baby,
            500..2000   => GrowthStage::Child,
            2000..10000 => GrowthStage::Adult,
            _           => GrowthStage::Elder,
        }
    }

    fn target_scale(&self) -> f32 {
        match self {
            GrowthStage::Baby  => 0.6,
            GrowthStage::Child => 0.8,
            GrowthStage::Adult => 1.0,
            GrowthStage::Elder => 0.95,
        }
    }

    /// Returns the sprite subdirectory for this growth stage.
    /// Falls back to the base species directory if no stage-specific sprites exist.
    pub fn sprite_subdir(&self) -> Option<&'static str> {
        match self {
            GrowthStage::Baby  => Some("cub"),
            GrowthStage::Child => Some("young"),
            GrowthStage::Adult => Some("adult"),
            GrowthStage::Elder => Some("elder"),
        }
    }

    #[allow(dead_code)]
    pub fn label(&self) -> &str {
        match self {
            GrowthStage::Baby  => "Baby",
            GrowthStage::Child => "Child",
            GrowthStage::Adult => "Adult",
            GrowthStage::Elder => "Elder",
        }
    }
}

pub struct EvolutionPlugin;

impl Plugin for EvolutionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GrowthState {
                stage: GrowthStage::Baby,
                current_scale: 0.6,
                target_scale: 0.6,
            })
           .add_systems(Update, evolution_system);
    }
}

/// Updates the creature's growth stage and smoothly scales the root entity.
fn evolution_system(
    time: Res<Time>,
    mind: Res<Mind>,
    mut growth: ResMut<GrowthState>,
    mut root_q: Query<&mut Transform, With<CreatureRoot>>,
) {
    let new_stage = GrowthStage::from_age(mind.age_ticks);

    if new_stage != growth.stage {
        info!(
            "Growth stage changed: {:?} → {:?} (age: {} ticks)",
            growth.stage, new_stage, mind.age_ticks
        );
        growth.stage = new_stage;
        growth.target_scale = new_stage.target_scale();
    }

    // Smoothly interpolate toward target scale
    let speed = 0.5; // scale units per second
    let diff = growth.target_scale - growth.current_scale;
    if diff.abs() > 0.001 {
        let step = diff.signum() * speed * time.delta_secs();
        // Don't overshoot
        if step.abs() > diff.abs() {
            growth.current_scale = growth.target_scale;
        } else {
            growth.current_scale += step;
        }
    }

    // Apply scale to the creature root (only when still interpolating)
    if (growth.current_scale - growth.target_scale).abs() > 0.001
        || growth.stage != GrowthStage::from_age(mind.age_ticks.saturating_sub(1))
    {
        for mut transform in root_q.iter_mut() {
            let s = growth.current_scale;
            transform.scale.x = s;
            transform.scale.y = s;
        }
    }
}
