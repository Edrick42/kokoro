//! Body part definitions and species templates.
//!
//! Each species is defined by a `SpeciesTemplate` containing:
//! - A **body rig** (proportional landmark system) that controls face/body shape
//! - A list of **body part definitions** with visual properties (fallback shape,
//!   tint, mood reactivity)
//!
//! The rig provides normalized positions that get resolved to pixel offsets
//! using the genome. This means each individual creature looks slightly
//! different, and different species can have radically different proportions.
//!
//! ## Adding a new species
//!
//! 1. Add a variant to `Species` in `genome/mod.rs`
//! 2. Write a rig function in `rig.rs` (e.g. `drakel_rig()`)
//! 3. Write a template function here (e.g. `drakel_template()`)
//! 4. Register both in `SpeciesRegistry::new()`
//! 5. Drop sprites into `assets/sprites/{species_dir}/`

use bevy::prelude::*;
use std::collections::HashMap;
use crate::genome::Species;
use super::rig::BodyRig;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marks the root entity of a composed creature.
/// All body parts are children of this entity, so moving/scaling the root
/// moves the entire creature.
#[derive(Component)]
pub struct CreatureRoot;

/// Identifies which body part slot an entity represents.
/// Uses a string so different species can have different part names
/// (e.g. "beak" for birds, "tail" for reptiles) without needing a
/// giant enum that covers every possible species.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BodyPartSlot(pub String);

/// Tags a body part as mood-reactive — its sprite will be swapped
/// whenever the creature's mood changes (e.g. eyes and mouth change
/// expression, but body and ears stay the same).
#[derive(Component)]
pub struct MoodReactive;

/// Tags a body part that should receive the genome's hue tint.
/// Body and ears are tinted; eyes and mouth are not.
#[derive(Component)]
pub struct Tinted;

// ---------------------------------------------------------------------------
// Template data structures
// ---------------------------------------------------------------------------

/// Procedural mesh fallback shape for when sprite PNGs are missing.
#[derive(Clone, Debug)]
pub enum FallbackShape {
    Circle { radius: f32 },
    Rect { width: f32, height: f32 },
}

/// Visual properties for a body part (everything except position,
/// which comes from the rig).
#[derive(Clone, Debug)]
pub struct BodyPartDef {
    /// Slot name, e.g. "body", "eye_left", "mouth"
    pub slot: String,
    /// Default scale
    pub base_scale: Vec2,
    /// Does this part change sprite when mood changes?
    pub mood_reactive: bool,
    /// Should the genome's hue tint be applied to this part?
    pub tinted: bool,
    /// Procedural mesh to use when no sprite PNG exists
    pub fallback_shape: FallbackShape,
    /// Color for the fallback mesh (None = use genome body color)
    pub fallback_color: Option<Color>,
}

/// Complete visual template for a species.
/// Combines the proportional rig with body part visual properties.
#[derive(Clone, Debug)]
pub struct SpeciesTemplate {
    /// Subdirectory under `assets/sprites/` for this species
    pub species_dir: String,
    /// Proportional landmark system — controls positioning
    pub rig: BodyRig,
    /// Visual properties for each body part
    pub parts: Vec<BodyPartDef>,
}

/// Registry mapping each `Species` to its visual template.
/// Inserted as a Bevy resource at startup.
#[derive(Resource)]
pub struct SpeciesRegistry {
    pub templates: HashMap<Species, SpeciesTemplate>,
}

impl SpeciesRegistry {
    /// Creates the registry with all known species templates.
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert(Species::Moluun, moluun_template());
        templates.insert(Species::Pylum, pylum_template());
        templates.insert(Species::Skael, skael_template());
        templates.insert(Species::Nyxal, nyxal_template());
        Self { templates }
    }

    /// Returns the template for the given species.
    pub fn get(&self, species: &Species) -> &SpeciesTemplate {
        self.templates
            .get(species)
            .expect("Species template not found in registry")
    }
}

// ---------------------------------------------------------------------------
// Species template definitions
// ---------------------------------------------------------------------------

/// The dark color used for eyes and mouth in the procedural mesh fallback.
const DARK: Color = Color::srgb(0.1, 0.1, 0.1);

/// Visual template for the Kobara species.
///
/// Positioning comes from `moluun_rig()` in `rig.rs`.
/// This function only defines visual properties (fallback shapes, tint, etc).
pub fn moluun_template() -> SpeciesTemplate {
    use super::rig::moluun_rig;

    SpeciesTemplate {
        species_dir: "moluun".into(),
        rig: moluun_rig(),
        parts: vec![
            BodyPartDef {
                slot: "body".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Circle { radius: 55.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "ear_left".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Circle { radius: 18.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "ear_right".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Circle { radius: 18.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "eye_left".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Circle { radius: 9.0 },
                fallback_color: Some(DARK),
            },
            BodyPartDef {
                slot: "eye_right".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Circle { radius: 9.0 },
                fallback_color: Some(DARK),
            },
            BodyPartDef {
                slot: "mouth".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Rect { width: 28.0, height: 7.0 },
                fallback_color: Some(DARK),
            },
        ],
    }
}

/// Visual template for the Pylum (bird) species.
///
/// Parts: body, wing_left, wing_right, eye_left, eye_right, beak, tail.
/// Positioning comes from `pylum_rig()` in `rig.rs`.
pub fn pylum_template() -> SpeciesTemplate {
    use super::rig::pylum_rig;

    SpeciesTemplate {
        species_dir: "pylum".into(),
        rig: pylum_rig(),
        parts: vec![
            BodyPartDef {
                slot: "body".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Circle { radius: 48.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "wing_left".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Rect { width: 40.0, height: 20.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "wing_right".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Rect { width: 40.0, height: 20.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "eye_left".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Circle { radius: 7.0 },
                fallback_color: Some(DARK),
            },
            BodyPartDef {
                slot: "eye_right".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Circle { radius: 7.0 },
                fallback_color: Some(DARK),
            },
            BodyPartDef {
                slot: "beak".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Rect { width: 16.0, height: 10.0 },
                fallback_color: Some(Color::srgb(0.95, 0.75, 0.2)),
            },
            BodyPartDef {
                slot: "tail".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Rect { width: 14.0, height: 22.0 },
                fallback_color: None,
            },
        ],
    }
}

/// Visual template for the Skael (reptile) species.
///
/// Parts: body, crest_left, crest_right, eye_left, eye_right, snout, tail.
/// Positioning comes from `skael_rig()` in `rig.rs`.
pub fn skael_template() -> SpeciesTemplate {
    use super::rig::skael_rig;

    SpeciesTemplate {
        species_dir: "skael".into(),
        rig: skael_rig(),
        parts: vec![
            BodyPartDef {
                slot: "body".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Circle { radius: 50.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "crest_left".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Rect { width: 12.0, height: 24.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "crest_right".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Rect { width: 12.0, height: 24.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "eye_left".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Circle { radius: 8.0 },
                fallback_color: Some(Color::srgb(0.9, 0.2, 0.1)),
            },
            BodyPartDef {
                slot: "eye_right".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Circle { radius: 8.0 },
                fallback_color: Some(Color::srgb(0.9, 0.2, 0.1)),
            },
            BodyPartDef {
                slot: "snout".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Rect { width: 30.0, height: 12.0 },
                fallback_color: Some(DARK),
            },
            BodyPartDef {
                slot: "tail".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Rect { width: 16.0, height: 35.0 },
                fallback_color: None,
            },
        ],
    }
}

/// Deep-sea bioluminescent color for Nyxal eye glow.
const BIOLUM: Color = Color::srgb(0.1, 0.5, 0.6);

pub fn nyxal_template() -> SpeciesTemplate {
    use super::rig::nyxal_rig;

    SpeciesTemplate {
        species_dir: "nyxal".into(),
        rig: nyxal_rig(),
        parts: vec![
            BodyPartDef {
                slot: "body".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Circle { radius: 45.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "mantle".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Circle { radius: 35.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "eye_left".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Circle { radius: 10.0 },
                fallback_color: Some(BIOLUM),
            },
            BodyPartDef {
                slot: "eye_right".into(),
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Circle { radius: 10.0 },
                fallback_color: Some(BIOLUM),
            },
            BodyPartDef {
                slot: "tentacle_front_left".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Rect { width: 10.0, height: 30.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "tentacle_front_right".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Rect { width: 10.0, height: 30.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "tentacle_back_left".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Rect { width: 10.0, height: 30.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "tentacle_back_right".into(),
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Rect { width: 10.0, height: 30.0 },
                fallback_color: None,
            },
        ],
    }
}
