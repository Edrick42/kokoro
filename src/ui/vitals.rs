//! Vital signs panel — displays heartbeat and breathing as a medical-style monitor.
//!
//! A small panel in the top-right corner showing:
//! - Heart icon + BPM value (pulses red on each beat)
//! - Lungs icon + breaths/min value
//!
//! The panel is always visible and updates in real time.

use bevy::prelude::*;

use crate::creature::species::CreatureRoot;
use crate::visuals::breathing::{BreathingState, HeartbeatState};

pub struct VitalsPlugin;

impl Plugin for VitalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_vitals_panel)
           .add_systems(Update, update_vitals_panel);
    }
}

/// Marker for the BPM text.
#[derive(Component)]
struct VitalsBpm;

/// Marker for the BPM pulse dot.
#[derive(Component)]
struct VitalsBpmDot;

/// Marker for the breathing rate text.
#[derive(Component)]
struct VitalsBreathing;

/// Marker for the breathing pulse dot.
#[derive(Component)]
struct VitalsBreathingDot;

fn setup_vitals_panel(mut commands: Commands) {
    // Panel — top right
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            right: Val::Px(8.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.75)),
        BorderRadius::all(Val::Px(8.0)),
    )).with_children(|panel| {
        // Heart rate row
        panel.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        }).with_children(|row| {
            // Pulse dot
            row.spawn((
                Node {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.9, 0.2, 0.25)),
                BorderRadius::all(Val::Px(4.0)),
                VitalsBpmDot,
            ));
            // BPM text
            row.spawn((
                Text::new("-- BPM"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.9, 0.3, 0.3)),
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
            // Pulse dot
            row.spawn((
                Node {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.7, 0.9)),
                BorderRadius::all(Val::Px(4.0)),
                VitalsBreathingDot,
            ));
            // Breathing text
            row.spawn((
                Text::new("-- br/min"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.4, 0.75, 0.95)),
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

    // Update BPM text
    if let Ok(mut text) = bpm_text.single_mut() {
        *text = Text::new(format!("{:.0} BPM", heartbeat.bpm));
    }

    // Update breathing text (convert Hz to breaths per minute)
    if let Ok(mut text) = breath_text.single_mut() {
        let breaths_per_min = breathing.rate * 60.0;
        *text = Text::new(format!("{:.0} br/min", breaths_per_min));
    }

    // Pulse the heart dot on each beat
    if let Ok(mut bg) = bpm_dot.single_mut() {
        if heartbeat.pulse_active > 0.0 {
            let t = (heartbeat.pulse_active / 0.12).max(0.0);
            bg.0 = Color::srgb(1.0, 0.1 + t * 0.3, 0.15 + t * 0.2);
        } else {
            bg.0 = Color::srgb(0.5, 0.15, 0.15);
        }
    }

    // Pulse the breathing dot on inhale peak
    if let Ok(mut bg) = breath_dot.single_mut() {
        let inhale = breathing.phase.sin().max(0.0);
        let r = 0.2 + inhale * 0.15;
        let g = 0.4 + inhale * 0.35;
        let b = 0.6 + inhale * 0.35;
        bg.0 = Color::srgb(r, g, b);
    }
}
