//! Player action menu — hierarchical, animated, minimalist.
//!
//! Bottom-center "..." opens the main panel:
//!   [Species row]
//!   [Feed] [Play] [Sleep]
//!
//! Clicking "Feed" expands a sub-panel with food choices.
//! Other buttons may have sub-actions in the future.

use bevy::prelude::*;
use crate::config::nutrition::FoodType;
use crate::genome::{Genome, Species};
use crate::mind::Mind;
use crate::mind::training::build_event_payload;
use crate::persistence::plugin::DbConnection;
use crate::persistence::save;
use crate::creature::collection::{CreatureCollection, SelectSpeciesEvent};
use crate::creature::egg::EggTapEvent;
use super::style::*;

// ===================================================================
// Components
// ===================================================================

#[derive(Component, Clone, Copy)]
enum ActionKind { Feed, Play, Sleep }

#[derive(Component, Clone, Copy)]
struct FoodBtn(FoodType);

#[derive(Component, Clone)]
struct SpeciesBtn(Species);

#[derive(Component)]
struct MenuToggle;

#[derive(Component)]
struct MenuPanel;

/// Sub-panel that shows food options (child of MenuPanel).
#[derive(Component)]
struct FoodSubPanel;

// ===================================================================
// State
// ===================================================================

#[derive(Resource)]
struct MenuState {
    open: bool,
    food_expanded: bool,
}

impl Default for MenuState {
    fn default() -> Self {
        Self { open: false, food_expanded: false }
    }
}

// ===================================================================
// Plugin
// ===================================================================

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MenuState::default())
           .add_systems(Startup, setup_menu)
           .add_systems(Update, (
               handle_menu_toggle,
               handle_action_press,
               handle_food_press,
               handle_species_press,
               sync_visibility,
               animate_buttons,
               smooth_button_scale,
           ));
    }
}

// ===================================================================
// Setup
// ===================================================================

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Toggle "..."
    commands.spawn((
        Button, MenuToggle, AnimatedButton,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-28.0)),
            width: Val::Px(56.0), height: Val::Px(30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(BTN_BORDER_WIDTH)),
            ..default()
        },
        BorderColor(BTN_BORDER),
        BorderRadius::all(Val::Px(15.0)),
        BackgroundColor(TOGGLE_BG),
    )).with_child((
        Text::new("..."),
        TextFont { font_size: FONT_LG, ..default() },
        TextColor(TOGGLE_TEXT),
    ));

    // Main panel
    commands.spawn((
        MenuPanel,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Px(0.0), right: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        BorderRadius::all(Val::Px(PANEL_RADIUS)),
        Visibility::Hidden,
    )).with_children(|panel| {
        // Species row
        panel.spawn(Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(BTN_GAP),
            ..default()
        }).with_children(|row| {
            spawn_species_btn(row, Species::Moluun, "Moluun", Color::srgb(0.55, 0.75, 0.90));
            spawn_species_btn(row, Species::Pylum,  "Pylum",  Color::srgb(0.90, 0.78, 0.45));
            spawn_species_btn(row, Species::Skael,  "Skael",  Color::srgb(0.50, 0.75, 0.55));
            spawn_species_btn(row, Species::Nyxal,  "Nyxal",  Color::srgb(0.45, 0.30, 0.70));
        });

        // Action row: Feed, Play, Sleep
        panel.spawn(Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(BTN_GAP),
            ..default()
        }).with_children(|row| {
            spawn_action_btn(row, ActionKind::Feed,  "Feed",  Color::srgb(0.95, 0.65, 0.25));
            spawn_action_btn(row, ActionKind::Play,  "Play",  Color::srgb(0.40, 0.80, 0.45));
            spawn_action_btn(row, ActionKind::Sleep, "Sleep", Color::srgb(0.45, 0.55, 0.90));
        });

        // Food sub-panel (hidden until Feed is clicked)
        panel.spawn((
            FoodSubPanel,
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(3.0),
                flex_wrap: FlexWrap::Wrap,
                row_gap: Val::Px(3.0),
                ..default()
            },
            Visibility::Hidden,
        )).with_children(|row| {
            for food in FoodType::ALL {
                spawn_food_btn(row, *food, &asset_server);
            }
        });
    });
}

// ===================================================================
// Button spawners
// ===================================================================

fn spawn_species_btn(parent: &mut ChildSpawnerCommands, species: Species, label: &str, color: Color) {
    parent.spawn((
        Button, AnimatedButton, SpeciesBtn(species),
        Node {
            width: Val::Px(70.0), height: Val::Px(BTN_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(BTN_BORDER_WIDTH)),
            ..default()
        },
        BorderColor(BTN_BORDER), BorderRadius::all(Val::Px(BTN_RADIUS)),
        BackgroundColor(color),
    )).with_child((
        Text::new(label.to_string()),
        TextFont { font_size: FONT_SM, ..default() },
        TextColor(TEXT_PRIMARY),
    ));
}

fn spawn_action_btn(parent: &mut ChildSpawnerCommands, kind: ActionKind, label: &str, color: Color) {
    parent.spawn((
        Button, AnimatedButton, kind,
        Node {
            width: Val::Px(80.0), height: Val::Px(BTN_HEIGHT + 4.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(BTN_BORDER_WIDTH)),
            ..default()
        },
        BorderColor(BTN_BORDER), BorderRadius::all(Val::Px(BTN_RADIUS)),
        BackgroundColor(color),
    )).with_child((
        Text::new(label.to_string()),
        TextFont { font_size: FONT_MD, ..default() },
        TextColor(TEXT_PRIMARY),
    ));
}

fn spawn_food_btn(parent: &mut ChildSpawnerCommands, food: FoodType, asset_server: &AssetServer) {
    let icon_path = format!("sprites/shared/food/{}.png", food.event_key());
    let has_icon = std::path::Path::new(&format!("assets/{icon_path}")).exists();

    parent.spawn((
        Button, AnimatedButton, FoodBtn(food),
        Node {
            width: Val::Px(FOOD_BTN_SIZE), height: Val::Px(FOOD_BTN_SIZE),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(BTN_BORDER_WIDTH)),
            row_gap: Val::Px(1.0),
            ..default()
        },
        BorderColor(BTN_BORDER), BorderRadius::all(Val::Px(BTN_RADIUS)),
        BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 0.9)),
    )).with_children(|btn| {
        if has_icon {
            btn.spawn((
                ImageNode::new(asset_server.load(&icon_path)),
                Node { width: Val::Px(BTN_ICON_SIZE), height: Val::Px(BTN_ICON_SIZE), ..default() },
            ));
        } else {
            btn.spawn((
                Node { width: Val::Px(BTN_ICON_SIZE), height: Val::Px(BTN_ICON_SIZE), ..default() },
                BackgroundColor(food.color()),
                BorderRadius::all(Val::Px(4.0)),
            ));
        }
        btn.spawn((
            Text::new(food.label().to_string()),
            TextFont { font_size: 8.0, ..default() },
            TextColor(TEXT_SECONDARY),
        ));
    });
}

// ===================================================================
// Systems
// ===================================================================

fn handle_menu_toggle(
    query: Query<&Interaction, (Changed<Interaction>, With<MenuToggle>)>,
    mut state: ResMut<MenuState>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            state.open = !state.open;
            if !state.open { state.food_expanded = false; }
        }
    }
}

fn sync_visibility(
    state: Res<MenuState>,
    mut panel_q: Query<&mut Visibility, (With<MenuPanel>, Without<FoodSubPanel>)>,
    mut food_q: Query<&mut Visibility, (With<FoodSubPanel>, Without<MenuPanel>)>,
) {
    if !state.is_changed() { return; }
    for mut vis in panel_q.iter_mut() {
        *vis = if state.open { Visibility::Visible } else { Visibility::Hidden };
    }
    for mut vis in food_q.iter_mut() {
        *vis = if state.open && state.food_expanded { Visibility::Visible } else { Visibility::Hidden };
    }
}

fn handle_action_press(
    query: Query<(&Interaction, &ActionKind), Changed<Interaction>>,
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
    db: Res<DbConnection>,
    collection: Res<CreatureCollection>,
    mut egg_events: EventWriter<EggTapEvent>,
    mut state: ResMut<MenuState>,
) {
    for (interaction, kind) in query.iter() {
        if *interaction != Interaction::Pressed { continue; }

        let is_egg = collection.creatures.get(collection.active_index)
            .map(|c| !c.egg.hatched).unwrap_or(false);
        if is_egg { egg_events.write(EggTapEvent); continue; }

        match kind {
            ActionKind::Feed => {
                // Toggle food sub-panel
                state.food_expanded = !state.food_expanded;
            }
            ActionKind::Play => {
                mind.play(&genome);
                state.food_expanded = false;
                info!("Player action: Play");
                let payload = build_event_payload(&mind.stats, &mind.mood, "played");
                let conn = db.0.lock().expect("DB lock poisoned");
                let _ = save::log_event(&conn, mind.age_ticks, "played", Some(&payload));
            }
            ActionKind::Sleep => {
                mind.sleep(&genome);
                state.food_expanded = false;
                info!("Player action: Sleep");
                let payload = build_event_payload(&mind.stats, &mind.mood, "slept");
                let conn = db.0.lock().expect("DB lock poisoned");
                let _ = save::log_event(&conn, mind.age_ticks, "slept", Some(&payload));
            }
        }
    }
}

fn handle_food_press(
    query: Query<(&Interaction, &FoodBtn), Changed<Interaction>>,
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
    db: Res<DbConnection>,
    collection: Res<CreatureCollection>,
    mut egg_events: EventWriter<EggTapEvent>,
    mut nutrient_q: Query<&mut crate::mind::nutrition::NutrientState, With<crate::creature::species::CreatureRoot>>,
) {
    for (interaction, food_btn) in query.iter() {
        if *interaction != Interaction::Pressed { continue; }

        let is_egg = collection.creatures.get(collection.active_index)
            .map(|c| !c.egg.hatched).unwrap_or(false);
        if is_egg { egg_events.write(EggTapEvent); continue; }

        let food = food_btn.0;
        if let Ok(mut nutrients) = nutrient_q.single_mut() {
            nutrients.add_food(&food.nutrients());
        }
        mind.feed(&genome, &food);

        info!("Fed: {}", food.full_name());
        let payload = build_event_payload(&mind.stats, &mind.mood, &format!("fed:{}", food.event_key()));
        let conn = db.0.lock().expect("DB lock poisoned");
        let _ = save::log_event(&conn, mind.age_ticks, "fed", Some(&payload));
    }
}

fn handle_species_press(
    query: Query<(&Interaction, &SpeciesBtn), Changed<Interaction>>,
    mut events: EventWriter<SelectSpeciesEvent>,
    mut state: ResMut<MenuState>,
) {
    for (interaction, btn) in query.iter() {
        if *interaction == Interaction::Pressed {
            events.write(SelectSpeciesEvent { species: btn.0.clone() });
            state.food_expanded = false;
        }
    }
}
