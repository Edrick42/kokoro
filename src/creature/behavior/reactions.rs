//! Creature Reaction System — visible responses to player actions.
//!
//! # Design Pattern: Event-Driven Decoupling
//!
//! Reactions are events. Any system writes `CreatureReaction` events; multiple
//! consumers read them:
//! - This plugin applies whole-creature physics impulses and facial expression
//! - `ImpulsePlugin` applies soft-body point impulses (skeletal deformation)
//!
//! Events decouple the source of the reaction from its visual effects.

use bevy::prelude::*;

use crate::config::nutrition::FoodType;
use crate::creature::interaction::physics::PhysicsBody;
use crate::creature::identity::species::CreatureRoot;
use crate::game::state::AppState;
use crate::mind::{Mind, MoodState};

// ===================================================================
// REACTION EVENTS — the decoupling layer
// ===================================================================

/// A creature reaction triggered by player action or game event.
/// Any system can write these. The ReactionPlugin reads and animates them.
#[derive(Event, Debug, Clone)]
#[allow(dead_code)]
pub enum CreatureReaction {
    // Feeding
    Eating { food: FoodType, preferred: bool },
    RefusingFood,

    // Interaction
    PlayStart,
    Petted { pleasure: f32 },
    Flinched { pain: f32 },

    // State changes
    FallingAsleep,
    WakingUp,
    Cleaning,
    GotSick,
    Recovered,
}

// ===================================================================
// EXPRESSION OVERRIDE — temporary face changes
// ===================================================================

/// Temporary facial expression override (eyes/mouth/blush).
/// Takes priority over the mood-based default expression.
#[derive(Resource, Default)]
pub struct ExpressionOverride {
    /// Override eye state: 0 = normal, 1 = wide, -1 = squint, 2 = half-closed
    pub eyes: i8,
    /// Override mouth state: 0 = normal, 1 = open, -1 = closed tight, 2 = smile wide
    pub mouth: i8,
    /// Extra blush intensity (0.0–1.0)
    pub blush: f32,
    /// Ticks remaining before reverting to mood-based expression.
    pub ticks: u32,
    /// Chewing animation phase (radians). Oscillates when mouth=1 (eating).
    /// sin(chew_phase) > 0 = mouth open, <= 0 = mouth closed (chewing cycle).
    pub chew_phase: f32,
}

impl ExpressionOverride {
    pub fn is_active(&self) -> bool { self.ticks > 0 }

    /// Returns true when mouth is in the "open" part of the chewing cycle.
    pub fn is_mouth_open(&self) -> bool {
        self.chew_phase.sin() > 0.0
    }

    pub fn set(&mut self, eyes: i8, mouth: i8, blush: f32, ticks: u32) {
        self.eyes = eyes;
        self.mouth = mouth;
        self.blush = blush;
        self.ticks = ticks;
        self.chew_phase = 0.0;
    }

    fn clear(&mut self) {
        self.eyes = 0;
        self.mouth = 0;
        self.blush = 0.0;
        self.ticks = 0;
        self.chew_phase = 0.0;
    }
}

// ===================================================================
// PLUGIN
// ===================================================================

pub struct ReactionPlugin;

impl Plugin for ReactionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreatureReaction>()
            .insert_resource(ExpressionOverride::default())
            .add_systems(Update, (
                mood_transition_watcher,
                process_reactions,
                tick_expression,
            ).chain().run_if(in_state(AppState::Gameplay)));
    }
}

/// Watches for mood transitions and fires appropriate reactions.
fn mood_transition_watcher(
    mind: Res<Mind>,
    mut prev_mood: Local<Option<MoodState>>,
    mut events: EventWriter<CreatureReaction>,
) {
    let current = mind.mood.clone();
    if let Some(ref prev) = *prev_mood {
        if *prev != current {
            // Waking up: was sleeping, now not
            if *prev == MoodState::Sleeping && current != MoodState::Sleeping {
                events.write(CreatureReaction::WakingUp);
            }
        }
    }
    *prev_mood = Some(current);
}

// ===================================================================
// SYSTEMS
// ===================================================================

/// Reads reaction events and triggers whole-creature physics + facial expression.
/// Soft-body point impulses are handled separately by `ImpulsePlugin`.
fn process_reactions(
    mut events: EventReader<CreatureReaction>,
    mut expression: ResMut<ExpressionOverride>,
    mut physics_q: Query<&mut PhysicsBody, With<CreatureRoot>>,
) {
    for event in events.read() {
        match event {
            CreatureReaction::Eating { preferred, .. } => {
                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.y = if *preferred { 8.0 } else { 4.0 };
                }
                expression.set(
                    if *preferred { 2 } else { 0 },
                    1,
                    if *preferred { 0.8 } else { 0.3 },
                    8,
                );
            }

            CreatureReaction::RefusingFood => {
                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.x = -5.0;
                }
                expression.set(-1, -1, 0.0, 10);
            }

            CreatureReaction::PlayStart => {
                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.y = 15.0;
                }
                expression.set(1, 2, 0.5, 12);
            }

            CreatureReaction::Petted { pleasure } => {
                expression.set(
                    2,
                    if *pleasure > 0.5 { 2 } else { 0 },
                    *pleasure,
                    6,
                );
            }

            CreatureReaction::Flinched { .. } => {
                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.x = 8.0;
                    body.velocity.y = 3.0;
                }
                expression.set(1, -1, 0.0, 5);
            }

            CreatureReaction::FallingAsleep => {
                expression.set(2, 0, 0.0, 6);
            }

            CreatureReaction::WakingUp => {
                expression.set(1, 0, 0.2, 8);
            }

            CreatureReaction::Cleaning => {
                expression.set(0, 0, 0.3, 8);
            }

            CreatureReaction::GotSick => {
                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.y = -3.0;
                }
                expression.set(-1, -1, 0.0, 8);
            }

            CreatureReaction::Recovered => {
                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.y = 5.0;
                }
                expression.set(0, 2, 0.4, 10);
            }
        }
    }
}

/// Ticks down expression override timer and advances chewing animation.
fn tick_expression(
    mut expression: ResMut<ExpressionOverride>,
    time: Res<Time>,
    mut elapsed: Local<f32>,
) {
    if expression.ticks == 0 { return; }

    let dt = time.delta_secs();

    // Chewing animation: advance phase every frame when eating (mouth=1).
    // ~4 chews per second (frequency = 8π/s, sin completes cycle every 2π).
    if expression.mouth == 1 {
        expression.chew_phase += dt * 8.0 * std::f32::consts::PI;
    }

    // Count ticks at ~1 tick per second (not per frame)
    *elapsed += dt;
    if *elapsed >= 1.0 {
        *elapsed -= 1.0;
        expression.ticks -= 1;
        if expression.ticks == 0 {
            expression.clear();
        }
    }
}
