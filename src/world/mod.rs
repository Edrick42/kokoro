//! World setup — spawns the camera and any static scene elements.

pub mod daycycle;
pub mod time_tick;

use bevy::prelude::*;

pub fn setup_world(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}
