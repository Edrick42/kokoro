//! Death screen — memorial for the fallen Kobara.

use bevy::prelude::*;

use crate::config::ui::{palette, fonts, buttons, PixelFont};
use crate::game::state::AppState;
use crate::mind::Mind;
use crate::mind::lifecycle::LifecycleState;
use crate::genome::Genome;
use crate::creature::species::CreatureRoot;
use crate::ui::style::{AnimatedButton, ButtonRestColor};

#[derive(Component)]
struct DeathScreenUI;

#[derive(Component)]
enum DeathAction { NewEgg }

pub struct DeathScreenPlugin;

impl Plugin for DeathScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::DeathScreen), setup_death_screen)
           .add_systems(OnExit(AppState::DeathScreen), cleanup_death_screen)
           .add_systems(Update, handle_death_input.run_if(in_state(AppState::DeathScreen)));
    }
}

fn setup_death_screen(
    mut commands: Commands,
    pixel_font: Res<PixelFont>,
    mind: Res<Mind>,
    genome: Res<Genome>,
    lifecycle_q: Query<&LifecycleState, With<CreatureRoot>>,
) {
    let font_lg = TextFont { font: pixel_font.0.clone(), font_size: fonts::LG, ..default() };
    let font_md = TextFont { font: pixel_font.0.clone(), font_size: fonts::MD, ..default() };
    let font_sm = TextFont { font: pixel_font.0.clone(), font_size: fonts::SM, ..default() };

    let cause = lifecycle_q.single()
        .ok()
        .and_then(|lc| lc.cause_of_death.as_ref())
        .map(|c| c.label())
        .unwrap_or("Unknown");

    let species_name = match &genome.species {
        crate::genome::Species::Moluun => "Moluun",
        crate::genome::Species::Pylum  => "Pylum",
        crate::genome::Species::Skael  => "Skael",
        crate::genome::Species::Nyxal  => "Nyxal",
    };

    let age_days = mind.age_ticks / 86400;

    commands.spawn((
        DeathScreenUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(palette::NEAR_BLACK),
    )).with_children(|parent| {
        parent.spawn((Text::new("REST IN PEACE"), font_lg.clone(), TextColor(palette::CREAM)));
        parent.spawn(Node { height: Val::Px(12.0), ..default() });
        parent.spawn((Text::new(format!("Your {} has passed.", species_name)), font_md.clone(), TextColor(palette::CREAM)));
        parent.spawn((Text::new(format!("Cause: {}", cause)), font_sm.clone(), TextColor(palette::RED)));
        parent.spawn((Text::new(format!("Lived {} days", age_days)), font_sm.clone(), TextColor(palette::GOLD)));
        parent.spawn(Node { height: Val::Px(30.0), ..default() });

        parent.spawn((
            Button, AnimatedButton, ButtonRestColor(palette::TEAL), DeathAction::NewEgg,
            Node {
                width: Val::Px(180.0), height: Val::Px(buttons::HEIGHT + 4.0),
                justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(buttons::BORDER_WIDTH)),
                ..default()
            },
            BorderColor(palette::CREAM), BorderRadius::all(Val::Px(0.0)), BackgroundColor(palette::TEAL),
        )).with_child((Text::new("NEW EGG"), font_md.clone(), TextColor(palette::CREAM)));
    });
}

fn cleanup_death_screen(mut commands: Commands, query: Query<Entity, With<DeathScreenUI>>) {
    for entity in query.iter() { commands.entity(entity).despawn(); }
}

fn handle_death_input(
    query: Query<(&Interaction, &DeathAction), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, _action) in query.iter() {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Gameplay);
        }
    }
}
