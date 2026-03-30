//! Body rig system — proportional landmark-based positioning.
//!
//! Instead of hardcoded pixel offsets, each species defines a set of **anchor
//! points** (landmarks) in a normalized coordinate space. The genome then
//! nudges these anchors to produce unique face shapes within the same species.
//!
//! ## Coordinate space
//!
//! All anchor positions use a normalized `[-1, 1]` system relative to the
//! creature's bounding box:
//!
//! ```text
//!        (-1, 1) ────────── (1, 1)
//!           │                  │
//!           │     (0, 0)       │    ← center
//!           │                  │
//!       (-1, -1) ────────── (1, -1)
//! ```
//!
//! `(0, 0.3)` means "center horizontally, 30% above center vertically".
//!
//! ## Species vs genome
//!
//! - **Species** defines the base rig: anchor positions, which genes affect
//!   which anchors, and how much variation is allowed. A Marumi (round, cute)
//!   has very different base positions than a Drakel (sharp, predatory).
//!
//! - **Genome** applies per-individual variation within the species' allowed
//!   ranges. Two Marumis will look similar but not identical.
//!
//! Think of it like facial landmark polygons — same topology, different
//! proportions per individual.

use bevy::prelude::*;
use crate::genome::Genome;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// Which genome gene drives a particular displacement.
#[derive(Debug, Clone, Copy)]
pub enum GeneTrait {
    Curiosity,
    Appetite,
    Resilience,
    LonelinessSensitivity,
    Hue,
}

impl GeneTrait {
    /// Reads the gene value (0.0–1.0) from a genome.
    pub fn read(&self, genome: &Genome) -> f32 {
        match self {
            GeneTrait::Curiosity              => genome.curiosity,
            GeneTrait::Appetite               => genome.appetite,
            GeneTrait::Resilience             => genome.resilience,
            GeneTrait::LonelinessSensitivity  => genome.loneliness_sensitivity,
            GeneTrait::Hue                    => genome.hue / 360.0,
        }
    }
}

/// A gene-driven displacement on one axis of an anchor point.
///
/// When the gene is at 0.0, the offset is `range.0`.
/// When the gene is at 1.0, the offset is `range.1`.
/// Values in between are linearly interpolated.
#[derive(Debug, Clone)]
pub struct GeneOffset {
    pub gene: GeneTrait,
    /// Which axis this offset affects.
    pub axis: Axis,
    /// Min/max displacement in normalized coords.
    /// Example: `(-0.05, 0.05)` means the anchor can shift ±5% of the bounding box.
    pub range: (f32, f32),
}

#[derive(Debug, Clone, Copy)]
pub enum Axis {
    X,
    Y,
    /// Both axes are displaced equally (uniform scale effect).
    Both,
}

/// A named anchor point on the creature's body.
///
/// Each anchor maps to a body part slot and defines where that part
/// sits relative to the creature's center, plus how the genome can
/// shift it.
#[derive(Debug, Clone)]
pub struct AnchorPoint {
    /// Body part slot this anchor controls (e.g. "eye_left").
    pub slot: String,

    /// Base position in normalized coordinates `[-1, 1]`.
    pub base_pos: Vec2,

    /// Z-depth for layering (lower = behind, higher = in front).
    pub z_depth: f32,

    /// Gene-driven displacements applied on top of `base_pos`.
    pub gene_offsets: Vec<GeneOffset>,

    /// If set, this anchor mirrors another anchor's X position.
    /// The mirror target's resolved X is negated for this anchor.
    /// Useful for left/right eye symmetry.
    pub mirror_of: Option<String>,
}

/// The complete proportional skeleton for a species.
///
/// Defines the base shape, size, and all anchor points. Two creatures
/// of the same species share the same `BodyRig` but get different
/// resolved positions because their genomes differ.
#[derive(Debug, Clone)]
pub struct BodyRig {
    /// Bounding box size in pixels at scale 1.0.
    /// Anchors are scaled from normalized `[-1,1]` to this size.
    /// Example: `Vec2::new(140.0, 160.0)` means the creature's
    /// visual space is 140px wide × 160px tall.
    pub base_size: Vec2,

    /// All anchor points that define the creature's proportions.
    pub anchors: Vec<AnchorPoint>,
}

/// A resolved anchor — the final pixel position after applying genome offsets.
#[derive(Debug, Clone)]
pub struct ResolvedAnchor {
    pub slot: String,
    pub position: Vec2,
    pub z_depth: f32,
}

// ---------------------------------------------------------------------------
// Rig resolution
// ---------------------------------------------------------------------------

impl BodyRig {
    /// Resolves all anchors to pixel positions using the given genome.
    ///
    /// This is the core function: it takes normalized proportional positions,
    /// applies genome-driven offsets, handles symmetry mirrors, and converts
    /// to pixel coordinates.
    pub fn resolve(&self, genome: &Genome) -> Vec<ResolvedAnchor> {
        let half = self.base_size * 0.5;

        // First pass: resolve all non-mirrored anchors
        let mut resolved: Vec<ResolvedAnchor> = Vec::new();
        let mut deferred_mirrors: Vec<&AnchorPoint> = Vec::new();

        for anchor in &self.anchors {
            if anchor.mirror_of.is_some() {
                deferred_mirrors.push(anchor);
                continue;
            }

            let pos = self.resolve_anchor(anchor, genome, half);
            resolved.push(ResolvedAnchor {
                slot: anchor.slot.clone(),
                position: pos,
                z_depth: anchor.z_depth,
            });
        }

        // Second pass: resolve mirrored anchors by copying + negating X
        for anchor in deferred_mirrors {
            let mirror_target = anchor.mirror_of.as_ref().unwrap();
            if let Some(source) = resolved.iter().find(|r| &r.slot == mirror_target) {
                let mut pos = source.position;
                pos.x = -pos.x; // Mirror horizontally
                resolved.push(ResolvedAnchor {
                    slot: anchor.slot.clone(),
                    position: pos,
                    z_depth: anchor.z_depth,
                });
            } else {
                // Fallback: resolve independently if mirror target not found
                let pos = self.resolve_anchor(anchor, genome, half);
                resolved.push(ResolvedAnchor {
                    slot: anchor.slot.clone(),
                    position: pos,
                    z_depth: anchor.z_depth,
                });
            }
        }

        resolved
    }

    /// Resolves a single anchor point to pixel coordinates.
    fn resolve_anchor(&self, anchor: &AnchorPoint, genome: &Genome, half_size: Vec2) -> Vec2 {
        let mut pos = anchor.base_pos;

        // Apply gene-driven offsets
        for offset in &anchor.gene_offsets {
            let gene_val = offset.gene.read(genome);
            let displacement = offset.range.0 + gene_val * (offset.range.1 - offset.range.0);

            match offset.axis {
                Axis::X    => pos.x += displacement,
                Axis::Y    => pos.y += displacement,
                Axis::Both => { pos.x += displacement; pos.y += displacement; }
            }
        }

        // Clamp to [-1, 1] so parts don't escape the bounding box
        pos = pos.clamp(Vec2::splat(-1.0), Vec2::splat(1.0));

        // Convert from normalized [-1, 1] to pixel coordinates
        Vec2::new(pos.x * half_size.x, pos.y * half_size.y)
    }
}

// ---------------------------------------------------------------------------
// Species rig definitions
// ---------------------------------------------------------------------------

/// Creates the body rig for the Marumi species.
///
/// Proportions are tuned to match the original idle_00.png reference:
/// round, chunky body with eyes slightly above center, small ears on top,
/// and a tiny mouth below center.
///
/// ```text
///          ear_L    ear_R
///            ┌────────┐
///           ╱          ╲
///         ╱  eye_L eye_R ╲
///        │                │
///        │     mouth      │
///         ╲              ╱
///           ╲__________╱
/// ```
pub fn marumi_rig() -> BodyRig {
    BodyRig {
        // Bounding box: 140×160 pixels — slightly taller than wide
        // to accommodate ears on top and feet on bottom.
        base_size: Vec2::new(140.0, 160.0),

        anchors: vec![
            // --- Body (center, behind everything) ---
            AnchorPoint {
                slot: "body".into(),
                base_pos: Vec2::new(0.0, -0.05),
                z_depth: 0.0,
                gene_offsets: vec![],
                mirror_of: None,
            },

            // --- Left ear (top-left, behind body) ---
            AnchorPoint {
                slot: "ear_left".into(),
                base_pos: Vec2::new(-0.35, 0.55),
                z_depth: -0.1,
                gene_offsets: vec![
                    // Curious creatures have ears slightly more spread apart
                    GeneOffset {
                        gene: GeneTrait::Curiosity,
                        axis: Axis::X,
                        range: (0.0, -0.08),
                    },
                ],
                mirror_of: None,
            },

            // --- Right ear (mirrors left ear) ---
            AnchorPoint {
                slot: "ear_right".into(),
                base_pos: Vec2::new(0.35, 0.55),
                z_depth: -0.1,
                gene_offsets: vec![],
                mirror_of: Some("ear_left".into()),
            },

            // --- Left eye (above center, in front) ---
            AnchorPoint {
                slot: "eye_left".into(),
                base_pos: Vec2::new(-0.22, 0.15),
                z_depth: 1.0,
                gene_offsets: vec![
                    // Curiosity widens eye spacing
                    GeneOffset {
                        gene: GeneTrait::Curiosity,
                        axis: Axis::X,
                        range: (0.05, -0.08),
                    },
                    // Resilience raises eyes slightly (confident look)
                    GeneOffset {
                        gene: GeneTrait::Resilience,
                        axis: Axis::Y,
                        range: (-0.02, 0.04),
                    },
                ],
                mirror_of: None,
            },

            // --- Right eye (mirrors left eye) ---
            AnchorPoint {
                slot: "eye_right".into(),
                base_pos: Vec2::new(0.22, 0.15),
                z_depth: 1.0,
                gene_offsets: vec![],
                mirror_of: Some("eye_left".into()),
            },

            // --- Mouth (below center, in front) ---
            AnchorPoint {
                slot: "mouth".into(),
                base_pos: Vec2::new(0.0, -0.15),
                z_depth: 1.0,
                gene_offsets: vec![
                    // Appetite shifts mouth down slightly (bigger mouth for hungry genes)
                    GeneOffset {
                        gene: GeneTrait::Appetite,
                        axis: Axis::Y,
                        range: (0.02, -0.05),
                    },
                ],
                mirror_of: None,
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Future species rigs — showing how different they can be
// ---------------------------------------------------------------------------

// Example of what a predator rig would look like (not implemented yet):
//
// pub fn drakel_rig() -> BodyRig {
//     BodyRig {
//         base_size: Vec2::new(120.0, 200.0),  // taller, narrower
//         anchors: vec![
//             // Eyes higher and closer together (forward-facing predator)
//             AnchorPoint {
//                 slot: "eye_left".into(),
//                 base_pos: Vec2::new(-0.15, 0.35),  // closer together, higher
//                 ...
//             },
//             // Snout/mouth further from eyes (elongated face)
//             AnchorPoint {
//                 slot: "mouth".into(),
//                 base_pos: Vec2::new(0.0, -0.3),  // much lower
//                 ...
//             },
//             // Pointed ears, higher and more angled
//             AnchorPoint {
//                 slot: "ear_left".into(),
//                 base_pos: Vec2::new(-0.45, 0.70),  // more spread, higher
//                 ...
//             },
//         ],
//     }
// }
//
// pub fn lumini_rig() -> BodyRig {
//     // Herbivore: wide face, eyes on sides (panoramic vision)
//     BodyRig {
//         base_size: Vec2::new(180.0, 140.0),  // wider than tall
//         anchors: vec![
//             AnchorPoint {
//                 slot: "eye_left".into(),
//                 base_pos: Vec2::new(-0.40, 0.20),  // far apart (side-facing)
//                 ...
//             },
//         ],
//     }
// }
