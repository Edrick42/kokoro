//! Genome-driven visual modifiers.
//!
//! Applies the creature's genetic traits to its visual appearance:
//! - `hue` gene → body/ear tint color
//! - `curiosity` gene → eye spacing (wider apart = more curious)
//! - `appetite` gene → body roundness (higher appetite = slimmer)
//!
//! This system runs once at startup (after the creature is spawned)
//! and also reacts to any changes in the `Genome` resource.

use bevy::prelude::*;
use crate::genome::Genome;
use super::body_parts::{BodyPartSlot, CreatureRoot};

/// Applies genome-driven visual transforms to body parts.
///
/// Eye spacing scales with the `curiosity` gene:
/// - curiosity = 0.0 → eyes at 70% of base distance (close together)
/// - curiosity = 1.0 → eyes at 130% of base distance (wide apart)
///
/// Body scale varies with `appetite`:
/// - appetite = 0.0 → body slightly wider (1.1x) — slow metabolism, rounder
/// - appetite = 1.0 → body slightly narrower (0.9x) — fast metabolism, leaner
pub fn apply_genome_visuals(
    genome: Res<Genome>,
    root_q: Query<&Children, With<CreatureRoot>>,
    mut part_q: Query<(&BodyPartSlot, &mut Transform, Option<&mut Sprite>), Without<CreatureRoot>>,
) {
    // Only run when the genome changes (or on first run)
    if !genome.is_changed() {
        return;
    }

    let Ok(children) = root_q.single() else { return };

    // Curiosity → eye spacing multiplier (0.7 to 1.3)
    let eye_spread = 0.7 + genome.curiosity * 0.6;

    // Appetite → body width multiplier (1.1 to 0.9, inverted)
    let body_scale_x = 1.1 - genome.appetite * 0.2;

    let tint = genome.tint_color();

    for child in children.iter() {
        let Ok((slot, mut transform, sprite)) = part_q.get_mut(child) else {
            continue;
        };

        match slot.0.as_str() {
            // Scale eye positions by curiosity
            "eye_left" => {
                transform.translation.x = -18.0 * eye_spread;
            }
            "eye_right" => {
                transform.translation.x = 18.0 * eye_spread;
            }
            // Scale body by appetite
            "body" => {
                transform.scale.x = body_scale_x;
            }
            _ => {}
        }

        // Apply tint to sprite parts that were spawned with a non-white color
        // (tinted parts receive the genome color at spawn time)
        if let Some(mut sprite) = sprite {
            if sprite.color != Color::WHITE {
                sprite.color = tint;
            }
        }
    }
}
