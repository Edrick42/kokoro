//! World setup — spawns the camera and any static scene elements.

use bevy::prelude::*;

pub fn setup_world(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}
