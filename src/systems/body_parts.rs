//! Body part definitions and species templates.
//!
//! Each species is defined by a `SpeciesTemplate` — a list of body parts with
//! their positions, z-depths, and visual properties. The creature spawn system
//! reads this template to assemble the creature from individual sprite/mesh
//! entities arranged in a parent-child hierarchy.
//!
//! ## Adding a new species
//!
//! 1. Add a variant to `Species` in `genome/mod.rs`
//! 2. Write a template function here (like `kobara_template()`)
//! 3. Register it in `SpeciesRegistry::new()`
//! 4. Drop sprites into `assets/sprites/{species_dir}/`

use bevy::prelude::*;
use std::collections::HashMap;
use crate::genome::Species;

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

/// Defines a single body part within a species template.
///
/// This is pure data — no Bevy components here. The spawn system reads
/// these definitions and creates the actual entities with the right
/// components attached.
#[derive(Clone, Debug)]
pub struct BodyPartDef {
    /// Slot name, e.g. "body", "eye_left", "mouth"
    pub slot: String,
    /// Position relative to the creature root
    pub offset: Vec2,
    /// Layering order (lower = behind, higher = in front)
    pub z_depth: f32,
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
/// One of these exists per species in the `SpeciesRegistry`.
#[derive(Clone, Debug)]
pub struct SpeciesTemplate {
    /// Subdirectory under `assets/sprites/` for this species
    pub species_dir: String,
    /// All body parts that make up this species
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
        templates.insert(Species::Kobara, kobara_template());
        // Future species go here:
        // templates.insert(Species::Lumini, lumini_template());
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
/// Part positions match the original `creature_form.rs` layout so the
/// procedural fallback looks identical to what was there before.
pub fn kobara_template() -> SpeciesTemplate {
    SpeciesTemplate {
        species_dir: "kobara".into(),
        parts: vec![
            BodyPartDef {
                slot: "ear_left".into(),
                offset: Vec2::new(-42.0, 45.0),
                z_depth: -0.1,
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Circle { radius: 18.0 },
                fallback_color: None, // uses genome body color
            },
            BodyPartDef {
                slot: "ear_right".into(),
                offset: Vec2::new(42.0, 45.0),
                z_depth: -0.1,
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Circle { radius: 18.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "body".into(),
                offset: Vec2::ZERO,
                z_depth: 0.0,
                base_scale: Vec2::ONE,
                mood_reactive: false,
                tinted: true,
                fallback_shape: FallbackShape::Circle { radius: 55.0 },
                fallback_color: None,
            },
            BodyPartDef {
                slot: "eye_left".into(),
                offset: Vec2::new(-18.0, 12.0),
                z_depth: 1.0,
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Circle { radius: 9.0 },
                fallback_color: Some(DARK),
            },
            BodyPartDef {
                slot: "eye_right".into(),
                offset: Vec2::new(18.0, 12.0),
                z_depth: 1.0,
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Circle { radius: 9.0 },
                fallback_color: Some(DARK),
            },
            BodyPartDef {
                slot: "mouth".into(),
                offset: Vec2::new(0.0, -14.0),
                z_depth: 1.0,
                base_scale: Vec2::ONE,
                mood_reactive: true,
                tinted: false,
                fallback_shape: FallbackShape::Rect { width: 28.0, height: 7.0 },
                fallback_color: Some(DARK),
            },
        ],
    }
}
