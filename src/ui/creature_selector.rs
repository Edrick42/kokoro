//! Creature selector UI — lets the player switch between their Kobaras.
//!
//! Displays a row of small numbered buttons in the top-right corner.
//! The active creature's button is highlighted. Only appears when
//! the player has more than one creature.

use bevy::prelude::*;
use crate::creature::collection::{CreatureCollection, SwitchCreatureEvent};

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
    mut switch_events: EventWriter<SwitchCreatureEvent>,
) {
    // Handle button clicks
    for (interaction, btn) in button_q.iter() {
        if *interaction == Interaction::Pressed {
            switch_events.write(SwitchCreatureEvent { index: btn.0 });
        }
    }

    // Only rebuild UI when collection changes
    if !collection.is_changed() {
        return;
    }

    // Don't show selector for single creature
    if collection.count() <= 1 {
        for entity in existing.iter() {
            commands.entity(entity).despawn();
        }
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
        for i in 0..collection.count() {
            let is_active = i == collection.active_index;
            let bg_color = if is_active {
                Color::srgb(0.3, 0.7, 0.4)
            } else {
                Color::srgb(0.3, 0.3, 0.3)
            };

            parent.spawn((
                Button,
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(bg_color),
                SelectorButton(i),
            )).with_child((
                Text::new(format!("{}", i + 1)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
        }
    });
}
