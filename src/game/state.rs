//! Game state machine — controls the flow between screens.
//!
//! ```text
//! Loading → TitleScreen → Auth → Onboarding → Gameplay ↔ Paused
//!                                                 ↓
//!                                           DeathScreen
//! ```

use bevy::prelude::*;

/// The main game state. Determines which UI is visible and which systems run.
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    /// Loading assets (font, database). Transitions to TitleScreen automatically.
    #[default]
    Loading,
    /// Logo + "Press to Start". Plays opening track.
    TitleScreen,
    /// Login / Register / Skip (guest mode).
    Auth,
    /// First-time tutorial. Skipped if save exists.
    #[allow(dead_code)]
    Onboarding,
    /// Main gameplay — creature alive, HUD visible, all systems active.
    Gameplay,
    /// Gameplay paused — creature frozen, menu accessible.
    #[allow(dead_code)]
    Paused,
    /// Creature died — memorial screen with cause of death.
    DeathScreen,
}

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
           .add_systems(Update, auto_transition_from_loading.run_if(in_state(AppState::Loading)));
    }
}

/// After one frame in Loading (assets queued), transition to TitleScreen.
fn auto_transition_from_loading(
    mut next_state: ResMut<NextState<AppState>>,
    mut frame_count: Local<u32>,
) {
    *frame_count += 1;
    // Wait 2 frames for assets to start loading
    if *frame_count >= 2 {
        next_state.set(AppState::TitleScreen);
    }
}
