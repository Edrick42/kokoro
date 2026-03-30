//! HUD plugin — displays the creature's vital stats and current mood on screen.
//!
//! Each stat row has a pixel art icon, a label, and a value. All rows sit
//! inside a semi-transparent dark panel for contrast against any background.

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

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = TextFont { font_size: 16.0, ..default() };

    // Dark semi-transparent panel behind all stats
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        BorderRadius::all(Val::Px(8.0)),
    )).with_children(|panel| {
        // Mood row
        spawn_stat_row(panel, &asset_server, &font, "sprites/shared/icons/mood.png", Color::WHITE, HudMood);
        // Hunger row
        spawn_stat_row(panel, &asset_server, &font, "sprites/shared/icons/hunger.png", Color::srgb(1.0, 0.6, 0.2), HudHunger);
        // Happiness row
        spawn_stat_row(panel, &asset_server, &font, "sprites/shared/icons/happiness.png", Color::srgb(0.4, 1.0, 0.5), HudHappiness);
        // Energy row
        spawn_stat_row(panel, &asset_server, &font, "sprites/shared/icons/energy.png", Color::srgb(0.4, 0.7, 1.0), HudEnergy);
    });
}

fn spawn_stat_row<M: Component>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &Res<AssetServer>,
    font: &TextFont,
    icon_path: &str,
    text_color: Color,
    marker: M,
) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(6.0),
        ..default()
    }).with_children(|row| {
        // Icon
        row.spawn((
            ImageNode::new(asset_server.load(icon_path)),
            Node {
                width: Val::Px(20.0),
                height: Val::Px(20.0),
                ..default()
            },
        ));

        // Text label + value
        row.spawn((
            Text::new(""),
            font.clone(),
            TextColor(text_color),
            marker,
        ));
    });
}

fn update_hud(
    mind: Res<Mind>,
    mut q_mood:      Query<&mut Text, (With<HudMood>,      Without<HudHunger>, Without<HudHappiness>, Without<HudEnergy>)>,
    mut q_hunger:    Query<&mut Text, (With<HudHunger>,    Without<HudMood>,   Without<HudHappiness>, Without<HudEnergy>)>,
    mut q_happiness: Query<&mut Text, (With<HudHappiness>, Without<HudMood>,   Without<HudHunger>,    Without<HudEnergy>)>,
    mut q_energy:    Query<&mut Text, (With<HudEnergy>,    Without<HudMood>,   Without<HudHunger>,    Without<HudHappiness>)>,
) {
    if let Ok(mut t) = q_mood.single_mut() {
        *t = Text::new(format!("{}", mind.mood.label()));
    }
    if let Ok(mut t) = q_hunger.single_mut() {
        *t = Text::new(format!("{:.0}%", mind.stats.hunger));
    }
    if let Ok(mut t) = q_happiness.single_mut() {
        *t = Text::new(format!("{:.0}%", mind.stats.happiness));
    }
    if let Ok(mut t) = q_energy.single_mut() {
        *t = Text::new(format!("{:.0}%", mind.stats.energy));
    }
}
