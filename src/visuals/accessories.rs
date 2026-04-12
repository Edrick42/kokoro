//! Accessories system — visual marks and accessories earned at age milestones.
//!
//! As a Kobara ages, it earns visual accessories that appear on its body:
//! - **Baby → Child** (500 ticks): small bow/ribbon
//! - **Child → Adult** (2000 ticks): scarf/collar
//! - **Adult → Elder** (10000 ticks): crown/halo + wisdom marks
//!
//! Accessories are spawned as child entities of `CreatureRoot`, positioned
//! relative to the rig's anchor points.

use bevy::prelude::*;
use crate::game::state::AppState;
use crate::mind::Mind;
use crate::creature::species::CreatureRoot;

pub struct AccessoriesPlugin;

impl Plugin for AccessoriesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AccessoryState::default())
           .add_systems(Update, check_milestones.run_if(in_state(AppState::Gameplay)));
    }
}

/// Tracks which accessories have already been spawned.
#[derive(Resource, Default)]
struct AccessoryState {
    child_accessory: bool,
    adult_accessory: bool,
    elder_accessory: bool,
}

/// Marks an accessory entity for cleanup if needed.
#[derive(Component)]
#[allow(dead_code)]
pub struct Accessory {
    pub kind: AccessoryKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessoryKind {
    Ribbon,
    Scarf,
    Crown,
}

/// Age thresholds from config — single source of truth for growth stages.
const CHILD_AGE: u64 = crate::config::growth::CUB_MAX;
const ADULT_AGE: u64 = crate::config::growth::YOUNG_MAX;
const ELDER_AGE: u64 = crate::config::growth::ADULT_MAX;

fn check_milestones(
    mind: Res<Mind>,
    mut state: ResMut<AccessoryState>,
    root_q: Query<Entity, With<CreatureRoot>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let age = mind.age_ticks;
    let Ok(root) = root_q.single() else { return };

    // Child milestone: small ribbon on top
    if age >= CHILD_AGE && !state.child_accessory {
        state.child_accessory = true;
        info!("Milestone: Child! Ribbon earned at tick {age}");

        commands.entity(root).with_children(|parent| {
            // Small pink bow
            parent.spawn((
                Mesh2d(meshes.add(Circle::new(6.0))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.7))),
                Transform::from_xyz(25.0, 55.0, 2.0),
                Accessory { kind: AccessoryKind::Ribbon },
            ));
        });
    }

    // Adult milestone: scarf/collar
    if age >= ADULT_AGE && !state.adult_accessory {
        state.adult_accessory = true;
        info!("Milestone: Adult! Scarf earned at tick {age}");

        commands.entity(root).with_children(|parent| {
            // Horizontal scarf under the face
            parent.spawn((
                Mesh2d(meshes.add(Rectangle::new(50.0, 8.0))),
                MeshMaterial2d(materials.add(Color::srgb(0.9, 0.3, 0.2))),
                Transform::from_xyz(0.0, -30.0, 2.0),
                Accessory { kind: AccessoryKind::Scarf },
            ));
        });
    }

    // Elder milestone: golden crown
    if age >= ELDER_AGE && !state.elder_accessory {
        state.elder_accessory = true;
        info!("Milestone: Elder! Crown earned at tick {age}");

        commands.entity(root).with_children(|parent| {
            // Golden crown on top
            parent.spawn((
                Mesh2d(meshes.add(Rectangle::new(35.0, 10.0))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 0.85, 0.1))),
                Transform::from_xyz(0.0, 75.0, 2.0),
                Accessory { kind: AccessoryKind::Crown },
            ));
            // Crown points (3 triangles approximated by small circles)
            for x_off in [-10.0, 0.0, 10.0] {
                parent.spawn((
                    Mesh2d(meshes.add(Circle::new(4.0))),
                    MeshMaterial2d(materials.add(Color::srgb(1.0, 0.85, 0.1))),
                    Transform::from_xyz(x_off, 83.0, 2.0),
                    Accessory { kind: AccessoryKind::Crown },
                ));
            }
        });
    }
}
