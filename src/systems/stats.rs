//! HUD plugin — displays the creature's vital stats and current mood on screen.
//!
//! Each stat label is its own entity with a marker component, so they can be
//! queried and updated independently without conflicting borrows.

use bevy::prelude::*;
use crate::mind::Mind;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
           .add_systems(Update, update_hud);
    }
}

// Marker components — one per HUD line to allow independent queries
#[derive(Component)] struct HudMood;
#[derive(Component)] struct HudHunger;
#[derive(Component)] struct HudHappiness;
#[derive(Component)] struct HudEnergy;

fn setup_hud(mut commands: Commands) {
    let font = TextFont { font_size: 16.0, ..default() };

    // Mood state label
    commands.spawn((
        Text::new(""),
        font.clone(),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top:  Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        HudMood,
    ));

    // Hunger bar — orange tint
    commands.spawn((
        Text::new(""),
        font.clone(),
        TextColor(Color::srgb(1.0, 0.6, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top:  Val::Px(36.0),
            left: Val::Px(12.0),
            ..default()
        },
        HudHunger,
    ));

    // Happiness bar — green tint
    commands.spawn((
        Text::new(""),
        font.clone(),
        TextColor(Color::srgb(0.4, 1.0, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            top:  Val::Px(60.0),
            left: Val::Px(12.0),
            ..default()
        },
        HudHappiness,
    ));

    // Energy bar — blue tint
    commands.spawn((
        Text::new(""),
        font,
        TextColor(Color::srgb(0.4, 0.7, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top:  Val::Px(84.0),
            left: Val::Px(12.0),
            ..default()
        },
        HudEnergy,
    ));
}

fn update_hud(
    mind: Res<Mind>,
    mut q_mood:      Query<&mut Text, (With<HudMood>,      Without<HudHunger>, Without<HudHappiness>, Without<HudEnergy>)>,
    mut q_hunger:    Query<&mut Text, (With<HudHunger>,    Without<HudMood>,   Without<HudHappiness>, Without<HudEnergy>)>,
    mut q_happiness: Query<&mut Text, (With<HudHappiness>, Without<HudMood>,   Without<HudHunger>,    Without<HudEnergy>)>,
    mut q_energy:    Query<&mut Text, (With<HudEnergy>,    Without<HudMood>,   Without<HudHunger>,    Without<HudHappiness>)>,
) {
    if let Ok(mut t) = q_mood.single_mut() {
        *t = Text::new(format!("Mood:      {}", mind.mood.label()));
    }
    if let Ok(mut t) = q_hunger.single_mut() {
        *t = Text::new(format!("Hunger:    {:.0}%", mind.stats.hunger));
    }
    if let Ok(mut t) = q_happiness.single_mut() {
        *t = Text::new(format!("Happiness: {:.0}%", mind.stats.happiness));
    }
    if let Ok(mut t) = q_energy.single_mut() {
        *t = Text::new(format!("Energy:    {:.0}%", mind.stats.energy));
    }
}
