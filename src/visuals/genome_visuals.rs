//! Genome-driven visual modifiers.
//!
//! Now that positioning is handled by the body rig (see `rig.rs`), this
//! system focuses on visual properties that the rig doesn't control:
//! - `hue` gene → body/ear tint color
//! - `appetite` gene → body width scale (rounder vs leaner)
//!
//! Eye spacing, ear spread, and mouth position are all handled by the rig's
//! gene offsets, so we don't need to duplicate that logic here.

use bevy::prelude::*;
use crate::genome::Genome;
use crate::creature::identity::species::{BodyPartSlot, CreatureRoot};

/// Applies genome-driven visual transforms to body parts.
///
/// Body scale varies with `appetite`:
/// - appetite = 0.0 → body slightly wider (1.1x) — slow metabolism, rounder
/// - appetite = 1.0 → body slightly narrower (0.9x) — fast metabolism, leaner
///
/// Tint color is applied to all parts marked as tinted (body, ears).
pub fn apply_genome_visuals(
    genome: Res<Genome>,
    root_q: Query<&Children, With<CreatureRoot>>,
    mut part_q: Query<(&BodyPartSlot, &mut Transform, Option<&mut Sprite>), Without<CreatureRoot>>,
) {
    if !genome.is_changed() {
        return;
    }

    let Ok(children) = root_q.single() else { return };

    // Appetite → body width multiplier (1.1 to 0.9, inverted)
    let body_scale_x = 1.1 - genome.appetite * 0.2;
    let tint = genome.tint_color();

    for child in children.iter() {
        let Ok((slot, mut transform, sprite)) = part_q.get_mut(child) else {
            continue;
        };

        // Scale body width by appetite
        if slot.0 == "body" {
            transform.scale.x = body_scale_x;
        }

        // Apply tint to tinted sprite parts
        if let Some(mut sprite) = sprite {
            if sprite.color != Color::WHITE {
                sprite.color = tint;
            }
        }
    }
}
