//! HUD plugin — displays the creature's vital stats and mood.
//!
//! Icons-first display with pixel font values. Retro Game Boy style.

use bevy::prelude::*;
use crate::config::ui::{palette, fonts, PixelFont};
use crate::config::ui::stats as stat_colors;
use crate::mind::Mind;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
           .add_systems(Update, update_hud);
    }
}

#[derive(Component)] struct HudMood;
#[derive(Component)] struct HudHunger;
#[derive(Component)] struct HudHappiness;
#[derive(Component)] struct HudEnergy;

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>, pixel_font: Res<PixelFont>) {
    let font = TextFont {
        font: pixel_font.0.clone(),
        font_size: fonts::MD,
        ..default()
    };

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            padding: UiRect::all(Val::Px(6.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(palette::PANEL_BG),
        BorderColor(palette::NEAR_BLACK),
        BorderRadius::all(Val::Px(0.0)),
    )).with_children(|panel| {
        // Mood — text only (no icon)
        panel.spawn((
            Text::new(""),
            font.clone(),
            TextColor(palette::NEAR_BLACK),
            HudMood,
        ));

        // Stats — icon + value
        spawn_icon_row(panel, &asset_server, &font, "sprites/shared/icons/hunger.png", stat_colors::HUNGER, HudHunger);
        spawn_icon_row(panel, &asset_server, &font, "sprites/shared/icons/happiness.png", stat_colors::HAPPINESS, HudHappiness);
        spawn_icon_row(panel, &asset_server, &font, "sprites/shared/icons/energy.png", stat_colors::ENERGY, HudEnergy);
    });
}

fn spawn_icon_row<M: Component>(
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
        column_gap: Val::Px(4.0),
        ..default()
    }).with_children(|row| {
        // Icon (small, retro)
        row.spawn((
            ImageNode::new(asset_server.load(icon_path)),
            Node {
                width: Val::Px(16.0),
                height: Val::Px(16.0),
                ..default()
            },
        ));
        // Value
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
        *t = Text::new(format!("{:.0}", mind.stats.hunger));
    }
    if let Ok(mut t) = q_happiness.single_mut() {
        *t = Text::new(format!("{:.0}", mind.stats.happiness));
    }
    if let Ok(mut t) = q_energy.single_mut() {
        *t = Text::new(format!("{:.0}", mind.stats.energy));
    }
}
