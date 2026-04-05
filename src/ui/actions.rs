//! Player action menu — a collapsible bottom menu triggered by a toggle button.
//!
//! A small "..." button sits at the bottom-center of the screen. Tapping it
//! opens a panel with species selector and action buttons (Feed, Play, Sleep).
//! Tapping again (or pressing an action) closes the menu.

use bevy::prelude::*;
use crate::genome::{Genome, Species};
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

/// Identifies a species selection button.
#[derive(Component, Clone)]
struct SpeciesButton(Species);

/// Marker for the toggle button.
#[derive(Component)]
struct MenuToggle;

/// Marker for the menu panel (shown/hidden).
#[derive(Component)]
struct MenuPanel;

/// Tracks whether the menu is open.
#[derive(Resource)]
struct MenuOpen(bool);

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MenuOpen(false))
           .add_systems(Startup, setup_menu)
           .add_systems(Update, (
               handle_menu_toggle,
               handle_action_press,
               handle_species_press,
               sync_menu_visibility,
           ));
    }
}

fn setup_menu(mut commands: Commands) {
    // Toggle button — always visible at bottom center
    commands.spawn((
        Button,
        MenuToggle,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-24.0)), // center the 48px button
            width: Val::Px(48.0),
            height: Val::Px(28.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        BorderRadius::all(Val::Px(14.0)),
        BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.85)),
    )).with_child((
        Text::new("..."),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
    ));

    // Menu panel — starts hidden
    commands.spawn((
        MenuPanel,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(48.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.9)),
        BorderRadius::all(Val::Px(12.0)),
        Visibility::Hidden,
    )).with_children(|panel| {
        // Species row
        panel.spawn(Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        }).with_children(|row| {
            spawn_species_button(row, Species::Moluun, "Moluun", Color::srgb(0.55, 0.75, 0.90));
            spawn_species_button(row, Species::Pylum,  "Pylum",  Color::srgb(0.90, 0.78, 0.45));
            spawn_species_button(row, Species::Skael,  "Skael",  Color::srgb(0.50, 0.75, 0.55));
            spawn_species_button(row, Species::Nyxal,  "Nyxal",  Color::srgb(0.45, 0.30, 0.70));
        });

        // Action row
        panel.spawn(Node {
            justify_content: JustifyContent::Center,
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
            width: Val::Px(80.0),
            height: Val::Px(36.0),
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
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::WHITE),
    ));
}

fn spawn_species_button(parent: &mut ChildSpawnerCommands, species: Species, label: &str, color: Color) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(80.0),
            height: Val::Px(28.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor(Color::srgb(0.15, 0.15, 0.15)),
        BorderRadius::all(Val::Px(6.0)),
        BackgroundColor(color),
        SpeciesButton(species),
    )).with_child((
        Text::new(label.to_string()),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::WHITE),
    ));
}

/// Toggles the menu open/closed when the "..." button is pressed.
fn handle_menu_toggle(
    query: Query<&Interaction, (Changed<Interaction>, With<MenuToggle>)>,
    mut menu_open: ResMut<MenuOpen>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            menu_open.0 = !menu_open.0;
        }
    }
}

/// Syncs the menu panel visibility with the MenuOpen resource.
fn sync_menu_visibility(
    menu_open: Res<MenuOpen>,
    mut panel_q: Query<&mut Visibility, With<MenuPanel>>,
) {
    if !menu_open.is_changed() {
        return;
    }
    for mut vis in panel_q.iter_mut() {
        *vis = if menu_open.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Handles Feed/Play/Sleep button presses.
fn handle_action_press(
    query: Query<(&Interaction, &ActionKind), Changed<Interaction>>,
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
    db: Res<DbConnection>,
) {
    for (interaction, kind) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let (event_type, action_label) = match kind {
            ActionKind::Feed => {
                mind.feed(&genome);
                ("fed", "Feed")
            }
            ActionKind::Play => {
                mind.play(&genome);
                ("played", "Play")
            }
            ActionKind::Sleep => {
                mind.sleep(&genome);
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
    query: Query<(&Interaction, &SpeciesButton), Changed<Interaction>>,
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
