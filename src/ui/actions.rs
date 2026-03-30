//! Player action buttons — Feed, Play, Sleep, Breed.
//!
//! Buttons are pinned to the bottom of the screen. Clicking (or tapping
//! on mobile) any button updates the creature's Mind and logs the event to
//! the SQLite database for neural network training data.
//!
//! Bevy's built-in `Interaction` component handles both mouse clicks and
//! touch events, so no extra input handling is needed.

use bevy::prelude::*;
use crate::mind::Mind;
use crate::mind::training::build_event_payload;
use crate::persistence::plugin::DbConnection;
use crate::persistence::save;
use crate::creature::reproduction::{BreedRequestEvent, BreedingState, can_breed};

/// Identifies which action a button triggers.
#[derive(Component, Clone, Copy)]
enum ActionKind {
    Feed,
    Play,
    Sleep,
    Breed,
}

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_buttons)
           .add_systems(Update, (handle_button_press, update_breed_button));
    }
}

/// Marker for the Breed button so we can update its appearance.
#[derive(Component)]
struct BreedButton;

/// Spawns action buttons in a horizontal row at the bottom of the screen.
fn setup_buttons(mut commands: Commands) {
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(20.0),
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        justify_content: JustifyContent::SpaceEvenly,
        align_items: AlignItems::Center,
        ..default()
    }).with_children(|parent| {
        spawn_button(parent, ActionKind::Feed,  "Feed",  Color::srgb(0.95, 0.65, 0.25));
        spawn_button(parent, ActionKind::Play,  "Play",  Color::srgb(0.40, 0.80, 0.45));
        spawn_button(parent, ActionKind::Sleep, "Sleep", Color::srgb(0.45, 0.55, 0.90));
        // Breed button (starts dimmed until conditions are met)
        parent.spawn((
            Button,
            Node {
                width: Val::Px(90.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor(Color::srgb(0.2, 0.2, 0.2)),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(Color::srgb(0.85, 0.4, 0.55)),
            ActionKind::Breed,
            BreedButton,
        )).with_child((
            Text::new("Breed".to_string()),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

fn spawn_button(parent: &mut ChildSpawnerCommands, kind: ActionKind, label: &str, color: Color) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(90.0),
            height: Val::Px(40.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor(Color::srgb(0.2, 0.2, 0.2)),
        BorderRadius::all(Val::Px(8.0)),
        BackgroundColor(color),
        kind,
    )).with_child((
        Text::new(label.to_string()),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::WHITE),
    ));
}

/// Updates the Breed button appearance based on whether breeding is possible.
fn update_breed_button(
    mind: Res<Mind>,
    breed_state: Res<BreedingState>,
    mut query: Query<&mut BackgroundColor, With<BreedButton>>,
) {
    let ready = can_breed(&mind, &breed_state);
    for mut bg in query.iter_mut() {
        *bg = if ready {
            BackgroundColor(Color::srgb(0.85, 0.4, 0.55))
        } else {
            BackgroundColor(Color::srgb(0.4, 0.3, 0.3))
        };
    }
}

/// Responds to button clicks/taps.
fn handle_button_press(
    query: Query<(&Interaction, &ActionKind), Changed<Interaction>>,
    mut mind: ResMut<Mind>,
    db: Res<DbConnection>,
    mut breed_events: EventWriter<BreedRequestEvent>,
) {
    for (interaction, kind) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let (event_type, action_label) = match kind {
            ActionKind::Feed => {
                mind.feed();
                ("fed", "Feed")
            }
            ActionKind::Play => {
                mind.play();
                ("played", "Play")
            }
            ActionKind::Sleep => {
                mind.sleep();
                ("slept", "Sleep")
            }
            ActionKind::Breed => {
                breed_events.write(BreedRequestEvent);
                ("breed_requested", "Breed")
            }
        };

        info!("Player action: {action_label}");

        // Log the event with full state snapshot for neural network training
        let payload = build_event_payload(&mind.stats, &mind.mood, event_type);
        let conn = db.0.lock().expect("DB lock poisoned");
        if let Err(e) = save::log_event(&conn, mind.age_ticks, event_type, Some(&payload)) {
            error!("Failed to log event: {e}");
        }
    }
}
