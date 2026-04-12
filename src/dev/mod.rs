//! Dev Mode — debug overlays and data panels for development.
//!
//! Enabled via `cargo run --features dev`. Toggle at runtime with **F12**.
//!
//! - **Rig Gizmos**: visualizes the creature's skeleton (anchors, connections,
//!   gene offsets, bounding box) using Bevy's built-in Gizmos.
//! - **Data Panels**: egui side panel showing vital stats, genome, and neural
//!   network state in real time.

mod panels;
mod rig_gizmos;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

/// Master state for dev mode — controls visibility of all overlays.
#[derive(Resource)]
pub struct DevModeState {
    pub active: bool,
    pub show_rig: bool,
    pub show_stats: bool,
    pub show_genome: bool,
    pub show_neural: bool,
    pub show_physics: bool,
    pub show_cheats: bool,
    /// Tick speed multiplier (1.0 = normal, 5.0 = 5x faster)
    pub tick_speed: f32,
}

impl Default for DevModeState {
    fn default() -> Self {
        Self {
            active: false,
            show_rig: true,
            show_stats: true,
            show_genome: true,
            show_neural: true,
            show_physics: true,
            show_cheats: true,
            tick_speed: 1.0,
        }
    }
}

pub struct DevPlugin;

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .insert_resource(DevModeState::default())
            .add_systems(Update, toggle_dev_mode)
            .add_systems(
                EguiPrimaryContextPass,
                panels::dev_panels_system
                    .run_if(|state: Res<DevModeState>| state.active),
            )
            .add_systems(
                Update,
                rig_gizmos::draw_rig_gizmos
                    .run_if(|state: Res<DevModeState>| state.active),
            );
    }
}

fn toggle_dev_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DevModeState>,
) {
    if keyboard.just_pressed(KeyCode::F12) {
        state.active = !state.active;
    }
}
