//! Basic creature animations.
//!
//! With the pixel art renderer, eye blink and mood sprite swaps are handled
//! directly by the pixel_creature system. This module is kept for future
//! animation systems (walk cycles, emotes, etc.).

use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, _app: &mut App) {
        // Blink and mood-reactive sprite swaps are now handled by PixelCreaturePlugin.
        // This plugin will host future animation systems (walk cycles, emotes).
    }
}
