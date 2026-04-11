//! Game state machine — controls the flow between screens.
//!
//! ```text
//! Loading → TitleScreen → Auth → Onboarding → Gameplay ↔ Paused
//!                                                 ↓
//!                                           DeathScreen
//! ```
//!
//! ## Entity cleanup
//!
//! All entities spawned during Gameplay are marked with `GameplayEntity`.
//! When leaving Gameplay (death, pause, etc.), all `GameplayEntity` entities
//! are despawned — clean slate for the next state.

use bevy::prelude::*;

/// The main game state.
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Loading,
    TitleScreen,
    Auth,
    Onboarding,
    Gameplay,
    #[allow(dead_code)]
    Paused,
    DeathScreen,
}

/// Marker component for all entities that belong to the Gameplay state.
/// Everything with this marker is despawned when leaving Gameplay.
#[derive(Component)]
pub struct GameplayEntity;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
           .add_systems(Update, auto_transition_from_loading.run_if(in_state(AppState::Loading)))
           .add_systems(OnExit(AppState::Gameplay), cleanup_gameplay_entities);
    }
}

fn auto_transition_from_loading(
    mut next_state: ResMut<NextState<AppState>>,
    mut frame_count: Local<u32>,
) {
    *frame_count += 1;
    if *frame_count >= 2 {
        next_state.set(AppState::TitleScreen);
    }
}

/// Despawns ALL entities marked with `GameplayEntity` when leaving Gameplay.
fn cleanup_gameplay_entities(
    mut commands: Commands,
    query: Query<Entity, With<GameplayEntity>>,
) {
    let count = query.iter().count();
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    if count > 0 {
        info!("Cleaned up {} gameplay entities", count);
    }
}
