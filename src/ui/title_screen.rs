//! Title screen — Logo + "Press to Start" with retro aesthetic.

use bevy::prelude::*;

use crate::config::ui::{palette, fonts, PixelFont};
use crate::game::state::AppState;

#[derive(Component)]
struct TitleScreenUI;

pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::TitleScreen), setup_title_screen)
           .add_systems(OnExit(AppState::TitleScreen), cleanup_title_screen)
           .add_systems(Update, handle_title_input.run_if(in_state(AppState::TitleScreen)));
    }
}

fn setup_title_screen(mut commands: Commands, pixel_font: Res<PixelFont>) {
    // Full-screen container
    commands.spawn((
        TitleScreenUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(40.0),
            ..default()
        },
        BackgroundColor(palette::CREAM),
    )).with_children(|parent| {
        // Game title
        parent.spawn((
            Text::new("KOKORO"),
            TextFont {
                font: pixel_font.0.clone(),
                font_size: 32.0,
                ..default()
            },
            TextColor(palette::NEAR_BLACK),
        ));

        // Subtitle
        parent.spawn((
            Text::new("where the spirit lives"),
            TextFont {
                font: pixel_font.0.clone(),
                font_size: fonts::SM,
                ..default()
            },
            TextColor(palette::GRAY),
        ));

        // Spacer
        parent.spawn(Node { height: Val::Px(40.0), ..default() });

        // Press to start (blinks via system)
        parent.spawn((
            Text::new("PRESS ANY KEY"),
            TextFont {
                font: pixel_font.0.clone(),
                font_size: fonts::MD,
                ..default()
            },
            TextColor(palette::NEAR_BLACK),
            BlinkText,
        ));
    });
}

fn cleanup_title_screen(
    mut commands: Commands,
    query: Query<Entity, With<TitleScreenUI>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
struct BlinkText;

/// Any key or click transitions to Auth screen.
fn handle_title_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut blink_q: Query<&mut TextColor, With<BlinkText>>,
) {
    // Blink the "PRESS ANY KEY" text
    for mut color in blink_q.iter_mut() {
        let alpha = ((time.elapsed_secs() * 2.0).sin() * 0.5 + 0.5).max(0.2);
        color.0 = palette::NEAR_BLACK.with_alpha(alpha);
    }

    // Any input transitions to Auth
    if keys.get_just_pressed().len() > 0 || mouse.just_pressed(MouseButton::Left) {
        next_state.set(AppState::Auth);
    }
}
