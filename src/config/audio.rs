//! Audio volume constants — mixing hierarchy.
//!
//! Lower values = more subtle. These define the relative loudness
//! of each audio layer so they blend without overwhelming each other.

/// Mood-ambient background drone volume.
pub const AMBIENT_VOLUME: f32 = 0.15;

/// Breathing noise volume (barely perceptible).
pub const BREATHING_VOLUME: f32 = 0.10;

/// Heartbeat pulse volume (audible rhythm).
pub const HEARTBEAT_VOLUME: f32 = 0.25;

/// Creature vocalization volume (primary sounds).
pub const VOCAL_VOLUME: f32 = 0.60;

/// UI interaction sounds volume.
pub const UI_VOLUME: f32 = 0.40;
