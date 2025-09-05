use bevy::{prelude::*, scene::ron::de, transform};
mod systems;
use systems::pet::Pet;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Pet::new()) 
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());

    commands.spawn(Sprite::from_image(
        asset_server.load("../assets/kokoro.png")
    ));
}