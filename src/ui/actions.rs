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

/// Side panel that shows food options as a vertical list.
#[derive(Component)]
struct FoodSubPanel;

/// Marker for the description text of a food item (toggled on hover).
#[derive(Component)]
struct FoodDescription(#[allow(dead_code)] FoodType);

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
               toggle_food_description,
               sync_visibility,
               animate_buttons,
               smooth_button_scale,
           ));
    }
}

// ===================================================================
// Setup
// ===================================================================

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>, pixel_font: Res<crate::config::ui::PixelFont>) {
    let font_lg = TextFont { font: pixel_font.0.clone(), font_size: fonts::LG, ..default() };
    let font_sm = TextFont { font: pixel_font.0.clone(), font_size: fonts::SM, ..default() };
    let font_md = TextFont { font: pixel_font.0.clone(), font_size: fonts::MD, ..default() };

    // Toggle "..." — flat retro rectangle
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
            border: UiRect::all(Val::Px(buttons::BORDER_WIDTH)),
            ..default()
        },
        BorderColor(NEAR_BLACK),
        BorderRadius::all(Val::Px(0.0)),
        BackgroundColor(CREAM),
    )).with_child((
        Text::new("..."),
        font_lg.clone(),
        TextColor(NEAR_BLACK),
    ));

    // Main panel — flat, dark
    commands.spawn((
        MenuPanel,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(44.0),
            // Center horizontally: auto margins on both sides
            margin: UiRect::horizontal(Val::Auto),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            // Shrink to content — don't stretch
            width: Val::Auto,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(4.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(crate::config::ui::palette::PANEL_BG),
        BorderColor(NEAR_BLACK),
        BorderRadius::all(Val::Px(0.0)),
        Visibility::Hidden,
    )).with_children(|panel| {
        // Species row — each species gets its palette color
        panel.spawn(Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(buttons::GAP),
            ..default()
        }).with_children(|row| {
            spawn_species_btn(row, &font_sm, Species::Moluun, "Moluun", GOLD);
            spawn_species_btn(row, &font_sm, Species::Pylum,  "Pylum",  ORANGE);
            spawn_species_btn(row, &font_sm, Species::Skael,  "Skael",  TEAL);
            spawn_species_btn(row, &font_sm, Species::Nyxal,  "Nyxal",  RED);
        });

        // Action row — uniform cream buttons (Game Boy style)
        panel.spawn(Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(buttons::GAP),
            ..default()
        }).with_children(|row| {
            spawn_action_btn(row, &font_md, ActionKind::Feed,  "Feed");
            spawn_action_btn(row, &font_md, ActionKind::Play,  "Play");
            spawn_action_btn(row, &font_md, ActionKind::Sleep, "Sleep");
        });

    });

    // Food side panel — separate from the menu, positioned on the right side
    commands.spawn((
        FoodSubPanel,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            top: Val::Px(80.0),
            bottom: Val::Px(80.0),
            width: Val::Px(180.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            padding: UiRect::all(Val::Px(4.0)),
            border: UiRect::all(Val::Px(2.0)),
            overflow: Overflow::scroll_y(),
            display: Display::None,
            ..default()
        },
        BackgroundColor(crate::config::ui::palette::PANEL_BG),
        BorderColor(NEAR_BLACK),
        BorderRadius::all(Val::Px(0.0)),
    )).with_children(|panel| {
        for food in FoodType::ALL {
            spawn_food_row(panel, *food, &asset_server, &font_sm);
        }
    });
}

// ===================================================================
// Button spawners
// ===================================================================

fn spawn_species_btn(parent: &mut ChildSpawnerCommands, font: &TextFont, species: Species, label: &str, color: Color) {
    // Species buttons use their species color as background
    let text_color = if color == TEAL || color == RED { CREAM } else { NEAR_BLACK };
    parent.spawn((
        Button, AnimatedButton, ButtonRestColor(color), SpeciesBtn(species),
        Node {
            width: Val::Px(70.0), height: Val::Px(buttons::HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(buttons::BORDER_WIDTH)),
            ..default()
        },
        BorderColor(NEAR_BLACK), BorderRadius::all(Val::Px(0.0)),
        BackgroundColor(color),
    )).with_child((
        Text::new(label.to_string()),
        font.clone(),
        TextColor(text_color),
    ));
}

fn spawn_action_btn(parent: &mut ChildSpawnerCommands, font: &TextFont, kind: ActionKind, label: &str) {
    // Action buttons: cream bg, dark text (uniform Game Boy style)
    parent.spawn((
        Button, AnimatedButton, kind,
        Node {
            width: Val::Px(80.0), height: Val::Px(buttons::HEIGHT + 4.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(buttons::BORDER_WIDTH)),
            ..default()
        },
        BorderColor(NEAR_BLACK), BorderRadius::all(Val::Px(0.0)),
        BackgroundColor(CREAM),
    )).with_child((
        Text::new(label.to_string()),
        font.clone(),
        TextColor(NEAR_BLACK),
    ));
}

/// Spawns a food row: [icon] [full name] — click to feed, acts as expandable item.
fn spawn_food_row(parent: &mut ChildSpawnerCommands, food: FoodType, asset_server: &AssetServer, font: &TextFont) {
    let icon_path = format!("sprites/shared/food/{}.png", food.event_key());
    let has_icon = std::path::Path::new(&format!("assets/{icon_path}")).exists();

    // Row container (button)
    parent.spawn((
        Button, FoodBtn(food),
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BorderColor(NEAR_BLACK), BorderRadius::all(Val::Px(0.0)),
        BackgroundColor(CREAM),
    )).with_children(|row| {
        // Top part: icon + name
        row.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            ..default()
        }).with_children(|top| {
            // Icon
            if has_icon {
                top.spawn((
                    ImageNode::new(asset_server.load(&icon_path)),
                    Node { width: Val::Px(20.0), height: Val::Px(20.0), ..default() },
                ));
            } else {
                top.spawn((
                    Node { width: Val::Px(20.0), height: Val::Px(20.0), ..default() },
                    BackgroundColor(food.color()),
                    BorderRadius::all(Val::Px(0.0)),
                ));
            }
            // Full name
            top.spawn((
                Text::new(food.full_name().to_string()),
                font.clone(),
                TextColor(NEAR_BLACK),
            ));
        });

        // Description (hidden by default, toggled on hover)
        row.spawn((
            Text::new(food.description().to_string()),
            font.clone(),
            TextColor(NEAR_BLACK),
            TextLayout::new_with_linebreak(LineBreak::WordBoundary),
            FoodDescription(food),
            Node {
                display: Display::None,
                margin: UiRect::top(Val::Px(4.0)),
                padding: UiRect::bottom(Val::Px(4.0)),
                ..default()
            },
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
    mut food_q: Query<&mut Node, (With<FoodSubPanel>, Without<MenuPanel>)>,
) {
    if !state.is_changed() { return; }
    for mut vis in panel_q.iter_mut() {
        *vis = if state.open { Visibility::Visible } else { Visibility::Hidden };
    }
    // Food sub-panel uses Display::None/Flex so it doesn't take layout space when hidden
    for mut node in food_q.iter_mut() {
        node.display = if state.open && state.food_expanded { Display::Flex } else { Display::None };
    }
}

/// Toggles food description visibility on hover over food rows.
fn toggle_food_description(
    food_q: Query<(&Interaction, &FoodBtn, &Children), Changed<Interaction>>,
    mut desc_q: Query<&mut Node, With<FoodDescription>>,
) {
    for (interaction, _food_btn, children) in food_q.iter() {
        for child in children.iter() {
            if let Ok(mut node) = desc_q.get_mut(child) {
                node.display = match interaction {
                    Interaction::Hovered | Interaction::Pressed => Display::Flex,
                    Interaction::None => Display::None,
                };
            }
        }
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
    bank: Res<crate::audio::SoundBank>,
    mut commands: Commands,
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
                play_action_sound(&bank, crate::audio::ActionSound::Play, &mut commands);
                let payload = build_event_payload(&mind.stats, &mind.mood, "played");
                let conn = db.0.lock().expect("DB lock poisoned");
                let _ = save::log_event(&conn, mind.age_ticks, "played", Some(&payload));
            }
            ActionKind::Sleep => {
                mind.sleep(&genome);
                state.food_expanded = false;
                play_action_sound(&bank, crate::audio::ActionSound::Sleep, &mut commands);
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
    bank: Res<crate::audio::SoundBank>,
    mut commands: Commands,
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
        play_action_sound(&bank, crate::audio::ActionSound::Eat, &mut commands);

        let payload = build_event_payload(&mind.stats, &mind.mood, &format!("fed:{}", food.event_key()));
        let conn = db.0.lock().expect("DB lock poisoned");
        let _ = save::log_event(&conn, mind.age_ticks, "fed", Some(&payload));
    }
}

/// Helper: plays an action sound from the SoundBank (silent if not loaded).
fn play_action_sound(bank: &crate::audio::SoundBank, action: crate::audio::ActionSound, commands: &mut Commands) {
    if let Some(handle) = bank.get(&crate::audio::SoundKey::Action(action)) {
        commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(
                crate::config::audio::VOCAL_VOLUME,
            )),
        ));
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
