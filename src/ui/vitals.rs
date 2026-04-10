//! Vital signs panel — heartbeat and breathing in retro style.
//!
//! Top-right panel showing BPM (red) and breathing rate (teal).
//! Flat panel, pixel font, palette colors only.

use bevy::prelude::*;

use crate::config::ui::{palette, fonts, PixelFont};
use crate::creature::species::CreatureRoot;
use crate::visuals::breathing::{BreathingState, HeartbeatState};

pub struct VitalsPlugin;

impl Plugin for VitalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_vitals_panel)
           .add_systems(Update, update_vitals_panel);
    }
}

#[derive(Component)] struct VitalsBpm;
#[derive(Component)] struct VitalsBpmDot;
#[derive(Component)] struct VitalsBreathing;
#[derive(Component)] struct VitalsBreathingDot;

fn setup_vitals_panel(mut commands: Commands, pixel_font: Res<PixelFont>) {
    let font = TextFont {
        font: pixel_font.0.clone(),
        font_size: fonts::MD,
        ..default()
    };

    // Flat dark panel — top right
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(8.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(palette::PANEL_BG),
        BorderColor(palette::NEAR_BLACK),
        BorderRadius::all(Val::Px(0.0)),
    )).with_children(|panel| {
        // Heart rate row
        panel.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        }).with_children(|row| {
            // Pulse dot (square, retro)
            row.spawn((
                Node {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(palette::RED),
                BorderRadius::all(Val::Px(0.0)),
                VitalsBpmDot,
            ));
            row.spawn((
                Text::new("-- BPM"),
                font.clone(),
                TextColor(palette::RED),
                VitalsBpm,
            ));
        });

        // Breathing rate row
        panel.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        }).with_children(|row| {
            row.spawn((
                Node {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(palette::TEAL),
                BorderRadius::all(Val::Px(0.0)),
                VitalsBreathingDot,
            ));
            row.spawn((
                Text::new("-- br/min"),
                font.clone(),
                TextColor(palette::TEAL),
                VitalsBreathing,
            ));
        });
    });
}

fn update_vitals_panel(
    root_q: Query<(&HeartbeatState, &BreathingState), With<CreatureRoot>>,
    mut bpm_text: Query<&mut Text, (With<VitalsBpm>, Without<VitalsBreathing>)>,
    mut breath_text: Query<&mut Text, (With<VitalsBreathing>, Without<VitalsBpm>)>,
    mut bpm_dot: Query<&mut BackgroundColor, (With<VitalsBpmDot>, Without<VitalsBreathingDot>)>,
    mut breath_dot: Query<&mut BackgroundColor, (With<VitalsBreathingDot>, Without<VitalsBpmDot>)>,
) {
    let Ok((heartbeat, breathing)) = root_q.single() else { return };

    if let Ok(mut text) = bpm_text.single_mut() {
        *text = Text::new(format!("{:.0} BPM", heartbeat.bpm));
    }

    if let Ok(mut text) = breath_text.single_mut() {
        let breaths_per_min = breathing.rate * 60.0;
        *text = Text::new(format!("{:.0} br/min", breaths_per_min));
    }

    // Pulse the heart dot — brighter RED on beat, darker between beats
    if let Ok(mut bg) = bpm_dot.single_mut() {
        if heartbeat.pulse_active > 0.0 {
            bg.0 = palette::CREAM; // flash cream on pulse
        } else {
            bg.0 = palette::RED;
        }
    }

    // Pulse the breathing dot — brighter on inhale
    if let Ok(mut bg) = breath_dot.single_mut() {
        let inhale = breathing.phase.sin().max(0.0);
        if inhale > 0.5 {
            bg.0 = palette::CREAM; // flash cream on inhale peak
        } else {
            bg.0 = palette::TEAL;
        }
    }
}
