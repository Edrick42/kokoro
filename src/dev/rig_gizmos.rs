//! Rig skeleton visualization using Bevy Gizmos.
//!
//! Draws anchor points, connections to body center, gene offset arrows,
//! and the species bounding box directly in world space — aligned with
//! the creature's sprites.

use bevy::prelude::*;

use crate::creature::rig::BodyRig;
use crate::creature::spawn::ResolvedRig;
use crate::creature::species::{CreatureRoot, SpeciesRegistry};
use crate::genome::Genome;

use super::DevModeState;

/// Color palette for anchor types.
fn slot_color(slot: &str) -> Color {
    if slot == "body" {
        Color::WHITE
    } else if slot.contains("eye") {
        Color::srgb(0.0, 0.9, 0.9) // cyan
    } else if slot.contains("mouth") || slot.contains("beak") || slot.contains("snout") {
        Color::srgb(1.0, 0.9, 0.2) // yellow
    } else if slot.contains("ear") || slot.contains("wing") || slot.contains("crest") {
        Color::srgb(0.2, 0.9, 0.3) // green
    } else if slot.contains("tail") {
        Color::srgb(0.9, 0.2, 0.9) // magenta
    } else {
        Color::srgb(0.7, 0.7, 0.7) // gray fallback
    }
}

pub fn draw_rig_gizmos(
    mut gizmos: Gizmos,
    resolved_rig: Option<Res<ResolvedRig>>,
    genome: Option<Res<Genome>>,
    registry: Option<Res<SpeciesRegistry>>,
    root_q: Query<&Transform, With<CreatureRoot>>,
    dev_state: Res<DevModeState>,
) {
    if !dev_state.show_rig {
        return;
    }

    let (Some(resolved_rig), Some(genome), Some(registry)) =
        (resolved_rig, genome, registry) else { return };

    let root_offset = root_q
        .iter()
        .next()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    let template = match registry.templates.get(&genome.species) {
        Some(t) => t,
        None => return,
    };
    let rig: &BodyRig = &template.rig;
    let half = rig.base_size * 0.5;

    // --- Bounding box ---
    gizmos.rect_2d(
        Isometry2d::from_translation(root_offset),
        rig.base_size,
        Color::srgba(1.0, 1.0, 1.0, 0.25),
    );

    // --- Find body center for connection lines ---
    let body_world = resolved_rig
        .anchors
        .iter()
        .find(|a| a.slot == "body")
        .map(|a| root_offset + a.position)
        .unwrap_or(root_offset);

    // --- Anchor dots + connection lines ---
    for anchor in &resolved_rig.anchors {
        let world_pos = root_offset + anchor.position;
        let color = slot_color(&anchor.slot);

        // Anchor dot
        gizmos.circle_2d(Isometry2d::from_translation(world_pos), 4.0, color);

        // Connection line to body center (skip body itself)
        if anchor.slot != "body" {
            gizmos.line_2d(body_world, world_pos, color.with_alpha(0.4));
        }
    }

    // --- Gene offset arrows ---
    // Show displacement from base position (no genes) to resolved position (with genes)
    for rig_anchor in &rig.anchors {
        if rig_anchor.gene_offsets.is_empty() || rig_anchor.mirror_of.is_some() {
            continue;
        }

        let base_world = root_offset + rig_anchor.base_pos * half;

        if let Some(resolved) = resolved_rig.anchors.iter().find(|a| a.slot == rig_anchor.slot) {
            let resolved_world = root_offset + resolved.position;
            let delta = resolved_world - base_world;

            // Only draw if there's a visible displacement
            if delta.length() > 0.5 {
                let arrow_color = Color::srgb(1.0, 0.6, 0.0); // orange
                gizmos.arrow_2d(base_world, resolved_world, arrow_color);
            }
        }
    }
}
