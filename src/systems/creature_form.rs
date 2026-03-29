//! Visual rendering of the Kobara.
//!
//! The body color is derived from the `hue` gene — every Kobara has its own unique color.
//! In Phase 3, these procedural meshes will be replaced by sprite sheets.

use bevy::prelude::*;
use crate::genome::Genome;

/// Marker component for the creature's main body mesh.
#[derive(Component)]
pub struct CreatureBody;

/// Marker component for the creature's eye meshes.
#[derive(Component)]
pub struct CreatureEyes;

/// Marker component for the creature's mouth mesh.
#[derive(Component)]
pub struct CreatureMouth;

/// Spawns the visual representation of the Kobara using procedural meshes.
///
/// Layout:
/// - Two ear circles rendered behind the body (z = -0.1)
/// - One large circle for the body (z = 0.0)
/// - Two eye circles (z = 1.0)
/// - One rectangle for the mouth (z = 1.0)
pub fn setup_creature(
    mut commands: Commands,
    mut meshes:   ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    genome: Res<Genome>,
) {
    let body_color = genome.body_color();
    let dark       = Color::srgb(0.1, 0.1, 0.1);

    // Ears — rendered behind the body
    for side in [-1.0_f32, 1.0_f32] {
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(18.0))),
            MeshMaterial2d(materials.add(body_color)),
            Transform::from_xyz(side * 42.0, 45.0, -0.1),
        ));
    }

    // Body
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(55.0))),
        MeshMaterial2d(materials.add(body_color)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        CreatureBody,
    ));

    // Left eye
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(9.0))),
        MeshMaterial2d(materials.add(dark)),
        Transform::from_xyz(-18.0, 12.0, 1.0),
        CreatureEyes,
    ));

    // Right eye
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(9.0))),
        MeshMaterial2d(materials.add(dark)),
        Transform::from_xyz(18.0, 12.0, 1.0),
        CreatureEyes,
    ));

    // Mouth — shape will reflect mood in a future system
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(28.0, 7.0))),
        MeshMaterial2d(materials.add(dark)),
        Transform::from_xyz(0.0, -14.0, 1.0),
        CreatureMouth,
    ));
}
