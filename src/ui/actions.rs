//! Player action buttons — Feed, Play, Sleep + species creation.
//!
//! Two rows of buttons:
//! - **Top row**: New Marumi / New Tsubasa / New Uroko — create a new creature
//! - **Bottom row**: Feed / Play / Sleep — interact with the active creature
//!
//! Bevy's built-in `Interaction` component handles both mouse clicks and
//! touch events, so no extra input handling is needed.

use bevy::prelude::*;
use crate::genome::Species;
use crate::mind::Mind;
use crate::mind::training::build_event_payload;
use crate::persistence::plugin::DbConnection;
use crate::persistence::save;
use crate::creature::collection::SelectSpeciesEvent;

/// Identifies which action a button triggers.
#[derive(Component, Clone, Copy)]
enum ActionKind {
    Feed,
    Play,
    Sleep,
}

/// Identifies a species creation button.
#[derive(Component, Clone)]
struct NewSpeciesButton(Species);

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_buttons)
           .add_systems(Update, (handle_action_press, handle_species_press));
    }
}

/// Spawns action buttons at the bottom of the screen.
fn setup_buttons(mut commands: Commands) {
    // Bottom container with two rows
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(10.0),
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: Val::Px(8.0),
        ..default()
    }).with_children(|parent| {
        // Top row — species creation
        parent.spawn(Node {
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        }).with_children(|row| {
            spawn_species_button(row, Species::Marumi,  "Marumi",   Color::srgb(0.55, 0.75, 0.90));
            spawn_species_button(row, Species::Tsubasa, "Tsubasa",  Color::srgb(0.90, 0.78, 0.45));
            spawn_species_button(row, Species::Uroko,   "Uroko",    Color::srgb(0.50, 0.75, 0.55));
        });

        // Bottom row — actions
        parent.spawn(Node {
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        }).with_children(|row| {
            spawn_action_button(row, ActionKind::Feed,  "Feed",  Color::srgb(0.95, 0.65, 0.25));
            spawn_action_button(row, ActionKind::Play,  "Play",  Color::srgb(0.40, 0.80, 0.45));
            spawn_action_button(row, ActionKind::Sleep, "Sleep", Color::srgb(0.45, 0.55, 0.90));
        });
    });
}

fn spawn_action_button(parent: &mut ChildSpawnerCommands, kind: ActionKind, label: &str, color: Color) {
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
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::WHITE),
    ));
}

fn spawn_species_button(parent: &mut ChildSpawnerCommands, species: Species, label: &str, color: Color) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(90.0),
            height: Val::Px(32.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor(Color::srgb(0.15, 0.15, 0.15)),
        BorderRadius::all(Val::Px(6.0)),
        BackgroundColor(color),
        NewSpeciesButton(species),
    )).with_child((
        Text::new(label.to_string()),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::WHITE),
    ));
}

/// Handles Feed/Play/Sleep button presses.
fn handle_action_press(
    query: Query<(&Interaction, &ActionKind), Changed<Interaction>>,
    mut mind: ResMut<Mind>,
    db: Res<DbConnection>,
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
        };

        info!("Player action: {action_label}");

        let payload = build_event_payload(&mind.stats, &mind.mood, event_type);
        let conn = db.0.lock().expect("DB lock poisoned");
        if let Err(e) = save::log_event(&conn, mind.age_ticks, event_type, Some(&payload)) {
            error!("Failed to log event: {e}");
        }
    }
}

/// Handles species button presses — switches to that species.
fn handle_species_press(
    query: Query<(&Interaction, &NewSpeciesButton), Changed<Interaction>>,
    mut events: EventWriter<SelectSpeciesEvent>,
) {
    for (interaction, btn) in query.iter() {
        if *interaction == Interaction::Pressed {
            events.write(SelectSpeciesEvent {
                species: btn.0.clone(),
            });
        }
    }
}
