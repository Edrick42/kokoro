//! Species-specific vocal sound definitions.
//!
//! Currently unused — vocalizations come from .ogg files loaded by SoundBank,
//! composed at runtime by voice_composer. This module is kept as a reference
//! for the synthesis approach if needed in the future.

use crate::genome::Species;

/// Placeholder — returns empty samples. Real sounds come from .ogg atoms.
pub fn vocalize_happy(_species: &Species) -> Vec<f32> { vec![0.0; 100] }
pub fn vocalize_hungry(_species: &Species) -> Vec<f32> { vec![0.0; 100] }
pub fn vocalize_sleepy(_species: &Species) -> Vec<f32> { vec![0.0; 100] }
