//! Creature selector UI — lets the player switch between their Kobaras.
//!
//! Displays a row of small named buttons in the top-right corner.
//! The active creature's button is highlighted.

use bevy::prelude::*;
use crate::creature::collection::{CreatureCollection, SelectSpeciesEvent};

pub struct CreatureSelectorPlugin;

impl Plugin for CreatureSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_selector);
    }
}

/// Root container for the selector UI.
#[derive(Component)]
struct SelectorRoot;

/// A selector button for a specific creature index.
#[derive(Component)]
struct SelectorButton(usize);

/// Rebuilds the selector UI when the collection changes, and handles clicks.
fn update_selector(
    collection: Res<CreatureCollection>,
    mut commands: Commands,
    existing: Query<Entity, With<SelectorRoot>>,
    button_q: Query<(&Interaction, &SelectorButton), Changed<Interaction>>,
    mut select_events: EventWriter<SelectSpeciesEvent>,
) {
    // Handle button clicks — look up the species at that index
    for (interaction, btn) in button_q.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(creature) = collection.creatures.get(btn.0) {
                select_events.write(SelectSpeciesEvent {
                    species: creature.genome.species.clone(),
                });
            }
        }
    }

    // Only rebuild UI when collection changes
    if !collection.is_changed() {
        return;
    }

    // Despawn old selector
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    // Build new selector
    commands.spawn((
        SelectorRoot,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(8.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        BorderRadius::all(Val::Px(6.0)),
    )).with_children(|parent| {
        for (i, creature) in collection.creatures.iter().enumerate() {
            let is_active = i == collection.active_index;
            let bg_color = if is_active {
                Color::srgb(0.3, 0.7, 0.4)
            } else {
                Color::srgb(0.3, 0.3, 0.3)
            };

            let label = &creature.name;

            parent.spawn((
                Button,
                Node {
                    height: Val::Px(28.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(bg_color),
                SelectorButton(i),
            )).with_child((
                Text::new(label.clone()),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::WHITE),
            ));
        }
    });
}
