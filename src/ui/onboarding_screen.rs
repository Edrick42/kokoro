//! Onboarding screen — narrative introduction to Ethara.
//!
//! 6 steps that introduce the player to the world, the Kobaras,
//! the Mirror Bond, and the lifecycle before entering gameplay.

use bevy::prelude::*;

use crate::config::ui::{palette, fonts, buttons, PixelFont};
use crate::game::state::AppState;
use crate::ui::style::{AnimatedButton, ButtonRestColor};

// ===================================================================
// Components
// ===================================================================

#[derive(Component)]
struct OnboardingScreenUI;

#[derive(Component)]
struct OnboardingTitle;

#[derive(Component)]
struct OnboardingBody;

#[derive(Component)]
struct OnboardingBtnText;

#[derive(Component)]
struct OnboardingDot(usize);

#[derive(Component)]
struct ContinueBtn;

// ===================================================================
// State
// ===================================================================

#[derive(Resource)]
struct OnboardingStep(usize);

const TOTAL_STEPS: usize = 6;

// ===================================================================
// Narrative content
// ===================================================================

struct StepContent {
    title: &'static str,
    body: &'static str,
    button_label: &'static str,
}

const STEPS: [StepContent; TOTAL_STEPS] = [
    StepContent {
        title: "THE ARRIVAL",
        body: "You don't remember how you got here.\n\n\
               One moment you were in the waking world. \
               The next, the air changed — thicker, warmer, \
               humming with something you can almost hear.\n\n\
               The light is different. The colors are deeper. \
               And somewhere ahead of you, something small \
               is waiting.\n\n\
               It already has your heartbeat.",
        button_label: "CONTINUE",
    },
    StepContent {
        title: "ETHARA",
        body: "This world exists on the other side of perception. \
               Not a place you travel to — a place you arrive at \
               when the boundary between your inner world and \
               the outer one grows thin enough.\n\n\
               Two moons trace slow arcs across the sky. \
               Everything pulses. The trees, the water, \
               the minerals — even the silence has a rhythm.\n\n\
               What you feel here has weight.",
        button_label: "CONTINUE",
    },
    StepContent {
        title: "THE KOBARAS",
        body: "The creatures of Ethara are not animals in the way \
               you understand animals. They are resonance beings — \
               shaped entirely by emotional energy.\n\n\
               Every Kobara carries a kokoro-sac, a resonance \
               organ that vibrates in response to emotional states. \
               It doesn't just reflect emotions — it becomes them.\n\n\
               Your Kobara is you.",
        button_label: "CONTINUE",
    },
    StepContent {
        title: "THE MIRROR BOND",
        body: "The relationship between you and your Kobara is \
               not caretaker and pet. It is self and reflection.\n\n\
               When you feed it, you tend to something you've \
               been neglecting. When you play with it, you give \
               yourself permission to be light. When you ignore it, \
               it grows distressed — not because it needs you, \
               but because you need you.\n\n\
               This is what kokoro means. The place where feeling \
               and awareness are the same thing.",
        button_label: "CONTINUE",
    },
    StepContent {
        title: "THE CYCLE OF LIFE",
        body: "Every Kobara follows the same cycle:\n\n\
               Egg — Cub — Young — Adult — Elder — Death\n\n\
               Growth is shaped by your care. A well-loved Kobara \
               lives longer. Its voice deepens with age. Its moods \
               become harder to dismiss because they start \
               reflecting truths you recognize.\n\n\
               Death is not failure. It is the completion of a cycle.",
        button_label: "CONTINUE",
    },
    StepContent {
        title: "BEGIN",
        body: "You are not a scientist. You are not a colonist. \
               You are here because something in you resonated \
               at the right frequency, and Ethara answered.\n\n\
               Your role is simple:\n\n\
               Care for what you find.",
        button_label: "ENTER ETHARA",
    },
];

// ===================================================================
// Plugin
// ===================================================================

pub struct OnboardingScreenPlugin;

impl Plugin for OnboardingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OnboardingStep(0))
           .add_systems(OnEnter(AppState::Onboarding), setup_onboarding)
           .add_systems(OnExit(AppState::Onboarding), cleanup_onboarding)
           .add_systems(Update, (
               handle_continue_press,
               update_onboarding_content,
           ).run_if(in_state(AppState::Onboarding)));
    }
}

// ===================================================================
// Setup
// ===================================================================

fn setup_onboarding(mut commands: Commands, pixel_font: Res<PixelFont>, mut step: ResMut<OnboardingStep>) {
    step.0 = 0;

    let font_lg = TextFont { font: pixel_font.0.clone(), font_size: fonts::LG, ..default() };
    let font_sm = TextFont { font: pixel_font.0.clone(), font_size: fonts::SM, ..default() };
    let font_md = TextFont { font: pixel_font.0.clone(), font_size: fonts::MD, ..default() };

    let content = &STEPS[0];

    commands.spawn((
        OnboardingScreenUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(30.0)),
            row_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(palette::CREAM),
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new(content.title),
            font_lg.clone(),
            TextColor(palette::NEAR_BLACK),
            OnboardingTitle,
        ));

        parent.spawn(Node { height: Val::Px(8.0), ..default() });

        // Body text — constrained width for readability
        parent.spawn((
            Text::new(content.body),
            font_sm.clone(),
            TextColor(palette::NEAR_BLACK),
            TextLayout::new_with_linebreak(LineBreak::WordBoundary),
            Node {
                max_width: Val::Px(340.0),
                ..default()
            },
            OnboardingBody,
        ));

        parent.spawn(Node { height: Val::Px(16.0), ..default() });

        // Step indicator dots
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            ..default()
        }).with_children(|dots_row| {
            for i in 0..TOTAL_STEPS {
                let color = if i == 0 { palette::NEAR_BLACK } else { palette::GRAY };
                dots_row.spawn((
                    Node {
                        width: Val::Px(8.0),
                        height: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(color),
                    BorderRadius::all(Val::Px(0.0)),
                    OnboardingDot(i),
                ));
            }
        });

        parent.spawn(Node { height: Val::Px(12.0), ..default() });

        // Continue / Enter Ethara button
        parent.spawn((
            Button, AnimatedButton, ButtonRestColor(palette::TEAL), ContinueBtn,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(buttons::HEIGHT + 4.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(buttons::BORDER_WIDTH)),
                ..default()
            },
            BorderColor(palette::CREAM),
            BorderRadius::all(Val::Px(0.0)),
            BackgroundColor(palette::TEAL),
        )).with_child((
            Text::new(content.button_label),
            font_md.clone(),
            TextColor(palette::CREAM),
            OnboardingBtnText,
        ));
    });
}

fn cleanup_onboarding(
    mut commands: Commands,
    query: Query<Entity, With<OnboardingScreenUI>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// ===================================================================
// Systems
// ===================================================================

fn handle_continue_press(
    query: Query<&Interaction, (Changed<Interaction>, With<ContinueBtn>)>,
    mut step: ResMut<OnboardingStep>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in query.iter() {
        if *interaction != Interaction::Pressed { continue; }

        if step.0 + 1 >= TOTAL_STEPS {
            // Last step — enter gameplay
            info!("Onboarding complete — entering Ethara");
            next_state.set(AppState::Gameplay);
        } else {
            step.0 += 1;
        }
    }
}

fn update_onboarding_content(
    step: Res<OnboardingStep>,
    mut q_title: Query<&mut Text, (With<OnboardingTitle>, Without<OnboardingBody>, Without<OnboardingBtnText>)>,
    mut q_body: Query<&mut Text, (With<OnboardingBody>, Without<OnboardingTitle>, Without<OnboardingBtnText>)>,
    mut q_btn: Query<&mut Text, (With<OnboardingBtnText>, Without<OnboardingTitle>, Without<OnboardingBody>)>,
    mut q_dots: Query<(&OnboardingDot, &mut BackgroundColor)>,
) {
    if !step.is_changed() { return; }

    let content = &STEPS[step.0];

    if let Ok(mut t) = q_title.single_mut() {
        *t = Text::new(content.title);
    }
    if let Ok(mut t) = q_body.single_mut() {
        *t = Text::new(content.body);
    }
    if let Ok(mut t) = q_btn.single_mut() {
        *t = Text::new(content.button_label);
    }

    // Update dot indicators
    for (dot, mut bg) in q_dots.iter_mut() {
        bg.0 = if dot.0 == step.0 { palette::NEAR_BLACK } else { palette::GRAY };
    }
}
