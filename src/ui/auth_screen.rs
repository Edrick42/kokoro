//! Auth screen — Login / Register / Play as Guest.
//!
//! Connected to the same auth system as the Leptos web frontend.
//! Guest mode skips auth and plays offline (no web sync).

use bevy::prelude::*;

use crate::config::ui::{palette, fonts, buttons, PixelFont};
use crate::game::state::AppState;
use crate::ui::style::{AnimatedButton, ButtonRestColor};

#[derive(Component)]
struct AuthScreenUI;

#[derive(Component)]
enum AuthAction { Login, Register, Guest }

pub struct AuthScreenPlugin;

impl Plugin for AuthScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Auth), setup_auth_screen)
           .add_systems(OnExit(AppState::Auth), cleanup_auth_screen)
           .add_systems(Update, handle_auth_input.run_if(in_state(AppState::Auth)));
    }
}

fn setup_auth_screen(mut commands: Commands, pixel_font: Res<PixelFont>) {
    let font_lg = TextFont { font: pixel_font.0.clone(), font_size: fonts::LG, ..default() };
    let font_md = TextFont { font: pixel_font.0.clone(), font_size: fonts::MD, ..default() };
    let font_sm = TextFont { font: pixel_font.0.clone(), font_size: fonts::SM, ..default() };

    commands.spawn((
        AuthScreenUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(12.0),
            ..default()
        },
        BackgroundColor(palette::CREAM),
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("KOKORO"),
            font_lg.clone(),
            TextColor(palette::NEAR_BLACK),
        ));

        parent.spawn(Node { height: Val::Px(20.0), ..default() });

        // Login button
        spawn_auth_btn(parent, &font_md, "LOGIN", AuthAction::Login, palette::TEAL);

        // Register button
        spawn_auth_btn(parent, &font_md, "REGISTER", AuthAction::Register, palette::GOLD);

        parent.spawn(Node { height: Val::Px(8.0), ..default() });

        // Guest button (subtle)
        spawn_auth_btn(parent, &font_sm, "PLAY AS GUEST", AuthAction::Guest, palette::CREAM);
    });
}

fn spawn_auth_btn(parent: &mut ChildSpawnerCommands, font: &TextFont, label: &str, action: AuthAction, bg: Color) {
    let text_color = if bg == palette::CREAM { palette::NEAR_BLACK } else { palette::CREAM };

    parent.spawn((
        Button, AnimatedButton, ButtonRestColor(bg), action,
        Node {
            width: Val::Px(200.0),
            height: Val::Px(buttons::HEIGHT + 4.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(buttons::BORDER_WIDTH)),
            ..default()
        },
        BorderColor(palette::NEAR_BLACK),
        BorderRadius::all(Val::Px(0.0)),
        BackgroundColor(bg),
    )).with_child((
        Text::new(label.to_string()),
        font.clone(),
        TextColor(text_color),
    ));
}

fn cleanup_auth_screen(
    mut commands: Commands,
    query: Query<Entity, With<AuthScreenUI>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_auth_input(
    query: Query<(&Interaction, &AuthAction), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, action) in query.iter() {
        if *interaction != Interaction::Pressed { continue; }

        match action {
            AuthAction::Login => {
                // TODO: show login form, validate JWT
                // For now, go straight to gameplay
                info!("Login pressed — skipping to gameplay (auth not wired yet)");
                next_state.set(AppState::Gameplay);
            }
            AuthAction::Register => {
                info!("Register pressed — skipping to gameplay (auth not wired yet)");
                next_state.set(AppState::Gameplay);
            }
            AuthAction::Guest => {
                info!("Playing as guest (offline mode)");
                next_state.set(AppState::Gameplay);
            }
        }
    }
}
