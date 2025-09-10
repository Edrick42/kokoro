use bevy::prelude::*;

// Componentes de partes do pet
#[derive(Component)]
pub struct CreatureEyes;

#[derive(Component)]
pub struct CreatureMouth;

pub fn setup_creature(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Head
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::hsla(30.0, 0.8, 0.8, 1.0))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Left Eye
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(8.0))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(-15.0, 15.0, 1.0),
        CreatureEyes,
    ));

    // Right Eye
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(8.0))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(15.0, 15.0, 1.0),
        CreatureEyes,
    ));

    // Mouth
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(30.0, 6.0))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(0.0, -10.0, 1.0),
        CreatureMouth,
    ));
}
