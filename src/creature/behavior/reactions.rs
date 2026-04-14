//! Creature Reaction System — visible responses to player actions.
//!
//! # Design Pattern: Event-Driven Decoupling
//!
//! The #1 pitfall in Rust gamedev is borrow checker conflicts when one system
//! needs to read state AND trigger side effects. The classic trap:
//!
//! ```ignore
//! // ❌ This fails — can't borrow `mind` mutably AND read `genome`
//! fn feed(mind: &mut Mind, genome: &Genome, physics: &mut PhysicsBody) { ... }
//! ```
//!
//! The solution: **events as decoupling layer**. The action system writes an event.
//! The reaction system reads it in a separate system. No overlapping borrows.
//!
//! ```ignore
//! // System A: writes event (only needs &mut EventWriter)
//! fn handle_feed(mut events: EventWriter<CreatureReaction>) {
//!     events.write(CreatureReaction::Eating { ... });
//! }
//!
//! // System B: reads event + applies effects (separate borrow scope)
//! fn apply_reactions(mut events: EventReader<CreatureReaction>, mut pose: ResMut<PoseState>) {
//!     for event in events.read() { ... }
//! }
//! ```
//!
//! This pattern eliminates 90% of borrow checker friction in game code.

use bevy::prelude::*;

use crate::config::nutrition::FoodType;
use crate::creature::behavior::pose::{ActiveAnimation, PoseAnimation, pose_from};
use crate::creature::interaction::physics::PhysicsBody;
use crate::creature::identity::species::CreatureRoot;
use crate::game::state::AppState;
use crate::genome::{Genome, Species};
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

/// Reads reaction events and triggers pose animations + physics + expressions.
fn process_reactions(
    mut events: EventReader<CreatureReaction>,
    mut active_anim: ResMut<ActiveAnimation>,
    mut expression: ResMut<ExpressionOverride>,
    mut physics_q: Query<&mut PhysicsBody, With<CreatureRoot>>,
    genome: Res<Genome>,
) {
    for event in events.read() {
        // Only process if no animation is already playing (queue would be better, but simple first)
        // Lesson: start simple, add complexity only when needed
        if active_anim.animation.is_some() { continue; }

        let species = &genome.species;

        match event {
            CreatureReaction::Eating { preferred, .. } => {
                // Pose: species-specific eating motion
                active_anim.animation = Some(eating_animation(species));

                // Physics: bounce (bigger for preferred food)
                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.y = if *preferred { 8.0 } else { 4.0 };
                }

                // Expression: mouth open, happy eyes
                expression.set(
                    if *preferred { 2 } else { 0 }, // eyes: half-closed if preferred (savoring)
                    1,                                // mouth: open (eating)
                    if *preferred { 0.8 } else { 0.3 }, // blush
                    8,
                );
            }

            CreatureReaction::RefusingFood => {
                active_anim.animation = Some(refusal_animation(species));

                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.x = -5.0; // back away
                }

                expression.set(-1, -1, 0.0, 10); // squint eyes, tight mouth
            }

            CreatureReaction::PlayStart => {
                active_anim.animation = Some(play_animation(species));

                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.y = 15.0; // big jump
                }

                expression.set(1, 2, 0.5, 12); // wide eyes, big smile
            }

            CreatureReaction::Petted { pleasure } => {
                active_anim.animation = Some(petted_animation(species));

                expression.set(
                    2,                      // half-closed (pleasure)
                    if *pleasure > 0.5 { 2 } else { 0 }, // smile if sweet spot
                    *pleasure,              // blush proportional to pleasure
                    6,
                );
            }

            CreatureReaction::Flinched { .. } => {
                active_anim.animation = Some(flinch_animation(species));

                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.x = 8.0; // jerk away
                    body.velocity.y = 3.0;
                }

                expression.set(1, -1, 0.0, 5); // wide eyes, tight mouth (shock)
            }

            CreatureReaction::FallingAsleep => {
                active_anim.animation = Some(sleep_animation(species));
                expression.set(2, 0, 0.0, 6); // half-closed eyes (peaceful drowsy), neutral mouth
            }

            CreatureReaction::WakingUp => {
                expression.set(1, 0, 0.2, 8); // blink wide, neutral mouth
            }

            CreatureReaction::Cleaning => {
                active_anim.animation = Some(clean_animation(species));
                expression.set(0, 0, 0.3, 8);
            }

            CreatureReaction::GotSick => {
                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.y = -3.0; // sag
                }
                expression.set(-1, -1, 0.0, 8); // droopy eyes, grimace (discomfort)
            }

            CreatureReaction::Recovered => {
                if let Ok(mut body) = physics_q.single_mut() {
                    body.velocity.y = 5.0; // perk up
                }
                expression.set(0, 2, 0.4, 10); // normal eyes, smile (relief)
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

// ===================================================================
// SPECIES-SPECIFIC POSE ANIMATIONS
// ===================================================================
// Lesson from real gamedev: keep animation data CLOSE to where it's used.
// Don't abstract prematurely. A HashMap of joint angles is simple and debuggable.

fn eating_animation(species: &Species) -> PoseAnimation {
    match species {
        Species::Moluun => PoseAnimation::new(vec![
            // Head dips DOWN to food (neck positive = down)
            (pose_from(&[("neck", 25.0), ("shoulder_left", 5.0), ("shoulder_right", 5.0)]), 3),
            // Head comes back UP (chomp/chew)
            (pose_from(&[("neck", -8.0)]), 2),
            // Second dip (another bite)
            (pose_from(&[("neck", 20.0)]), 3),
            // Head comes up, chewing
            (pose_from(&[("neck", -5.0)]), 2),
            // Return to neutral
            (pose_from(&[]), 4),
        ]),
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("neck", 35.0), ("tail", 5.0)]), 2),    // sharp peck
            (pose_from(&[("neck", -10.0)]), 2),                    // head up (swallow)
            (pose_from(&[("neck", 35.0), ("tail", 5.0)]), 2),    // peck again
            (pose_from(&[("neck", 30.0)]), 2),                     // third peck
            (pose_from(&[]), 3),
        ]),
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("neck", 15.0), ("tail", -10.0)]), 5),  // slow lunge
            (pose_from(&[("neck", -5.0), ("tail", 5.0)]), 3),    // snap shut
            (pose_from(&[]), 5),                                   // slow return
        ]),
        Species::Nyxal => PoseAnimation::new(vec![
            (pose_from(&[("tentacle_front_left", 30.0), ("tentacle_front_right", 30.0), ("mantle_base", -5.0)]), 4),
            (pose_from(&[("tentacle_front_left", 10.0), ("tentacle_front_right", 10.0), ("mantle_base", -10.0)]), 3), // contract (swallow)
            (pose_from(&[]), 4),
        ]),
    }
}

fn refusal_animation(species: &Species) -> PoseAnimation {
    match species {
        Species::Moluun => PoseAnimation::new(vec![
            // Head pulls BACK/UP (refusing — negative neck = up)
            (pose_from(&[("neck", -20.0), ("shoulder_left", -5.0), ("shoulder_right", -5.0)]), 3),
            // Slight forward (hesitation)
            (pose_from(&[("neck", -10.0)]), 2),
            // Back again (firm refusal)
            (pose_from(&[("neck", -18.0)]), 3),
            // Return
            (pose_from(&[]), 4),
        ]),
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("neck", -20.0), ("wing_left", -10.0), ("wing_right", -10.0)]), 4), // head turn, wings fold
            (pose_from(&[]), 4),
        ]),
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("neck", -8.0)]), 4),    // slow head shake
            (pose_from(&[("neck", 8.0)]), 4),
            (pose_from(&[]), 4),
        ]),
        Species::Nyxal => PoseAnimation::new(vec![
            (pose_from(&[("tentacle_front_left", -20.0), ("tentacle_front_right", -20.0)]), 4), // retract
            (pose_from(&[("mantle_base", 5.0)]), 3), // puff up (displeasure)
            (pose_from(&[]), 4),
        ]),
    }
}

fn play_animation(species: &Species) -> PoseAnimation {
    match species {
        Species::Moluun => PoseAnimation::new(vec![
            // Crouch (hips flex, head dips)
            (pose_from(&[("hip_left", 15.0), ("hip_right", 15.0), ("neck", 10.0), ("shoulder_left", 15.0), ("shoulder_right", 15.0)]), 2),
            // Jump! (hips extend, arms up, head up)
            (pose_from(&[("hip_left", -15.0), ("hip_right", -15.0), ("neck", -15.0), ("shoulder_left", -10.0), ("shoulder_right", -10.0)]), 2),
            // Land (hips absorb)
            (pose_from(&[("hip_left", 10.0), ("hip_right", 10.0), ("neck", 5.0)]), 2),
            // Bounce again
            (pose_from(&[("hip_left", -10.0), ("hip_right", -10.0), ("neck", -10.0)]), 2),
            // Settle
            (pose_from(&[]), 4),
        ]),
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("wing_left", 40.0), ("wing_right", 40.0), ("keel_joint", -15.0)]), 3), // wings spread, crouch
            (pose_from(&[("wing_left", -10.0), ("wing_right", -10.0), ("keel_joint", 5.0)]), 2), // jump
            (pose_from(&[("wing_left", 30.0), ("wing_right", 30.0)]), 3), // flap
            (pose_from(&[]), 4),
        ]),
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("tail", 25.0)]), 3),     // tail wind-up
            (pose_from(&[("tail", -30.0), ("neck", 10.0)]), 3), // tail slam + lunge
            (pose_from(&[]), 5),
        ]),
        Species::Nyxal => PoseAnimation::new(vec![
            (pose_from(&[("tentacle_front_left", 40.0), ("tentacle_front_right", -40.0), ("mantle_base", 10.0)]), 3),
            (pose_from(&[("tentacle_front_left", -40.0), ("tentacle_front_right", 40.0)]), 3), // spin
            (pose_from(&[("tentacle_front_left", 40.0), ("tentacle_front_right", -40.0)]), 3),
            (pose_from(&[]), 4),
        ]),
    }
}

fn petted_animation(species: &Species) -> PoseAnimation {
    match species {
        // Moluun: lean in (head dips toward touch), arms relax down
        Species::Moluun => PoseAnimation::new(vec![
            (pose_from(&[("neck", 12.0), ("shoulder_left", 8.0), ("shoulder_right", 8.0)]), 5),  // lean in
            (pose_from(&[("neck", 10.0), ("shoulder_left", 5.0), ("shoulder_right", 5.0)]), 8),  // hold (enjoying)
            (pose_from(&[("neck", 8.0)]), 5),  // slowly relax
            (pose_from(&[]), 4),
        ]),
        // Pylum: slight wing spread (comfort)
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("wing_left", 15.0), ("wing_right", 15.0)]), 6),
            (pose_from(&[("wing_left", 10.0), ("wing_right", 10.0)]), 8),
            (pose_from(&[]), 4),
        ]),
        // Skael: stay very still (trust = stillness for reptiles)
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("tail", -5.0)]), 10), // tail relaxes flat
            (pose_from(&[]), 4),
        ]),
        // Nyxal: tentacles reach toward touch
        Species::Nyxal => PoseAnimation::new(vec![
            (pose_from(&[("tentacle_front_left", 15.0), ("tentacle_front_right", 15.0), ("mantle_base", 3.0)]), 8),
            (pose_from(&[]), 4),
        ]),
    }
}

fn flinch_animation(species: &Species) -> PoseAnimation {
    match species {
        // Moluun flinch: head snaps UP/BACK, arms retract, feet brace
        Species::Moluun => PoseAnimation::new(vec![
            (pose_from(&[("neck", -25.0), ("shoulder_left", -15.0), ("shoulder_right", -15.0), ("hip_left", 8.0), ("hip_right", 8.0)]), 2),
            (pose_from(&[("neck", -10.0)]), 3),  // recovering
            (pose_from(&[]), 4),
        ]),
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("wing_left", 50.0), ("wing_right", 50.0), ("neck", -25.0)]), 3), // startle display
            (pose_from(&[]), 5),
        ]),
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("tail", 30.0), ("neck", -10.0)]), 3), // tail whip, head tuck
            (pose_from(&[]), 5),
        ]),
        Species::Nyxal => PoseAnimation::new(vec![
            (pose_from(&[("tentacle_front_left", -30.0), ("tentacle_front_right", -30.0), ("mantle_base", -15.0)]), 3),
            (pose_from(&[]), 5),
        ]),
    }
}

fn sleep_animation(species: &Species) -> PoseAnimation {
    match species {
        // Moluun: head dips DOWN, arms drop, feet settle — curling into ball
        Species::Moluun => PoseAnimation::new(vec![
            (pose_from(&[("neck", 15.0), ("shoulder_left", 10.0), ("shoulder_right", 10.0), ("hip_left", 10.0), ("hip_right", 10.0)]), 6),
            (pose_from(&[("neck", 20.0), ("shoulder_left", 15.0), ("shoulder_right", 15.0), ("hip_left", 15.0), ("hip_right", 15.0)]), 15), // hold (deeply asleep — everything droops)
        ]),
        // Pylum: tuck head under wing
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("neck", 25.0), ("wing_left", 20.0)]), 8),
            (pose_from(&[("neck", 30.0), ("wing_left", 25.0)]), 15),
        ]),
        // Skael: lower to ground, tail wraps
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("hip_left", 10.0), ("hip_right", 10.0), ("tail", 15.0)]), 8),
            (pose_from(&[("hip_left", 15.0), ("hip_right", 15.0), ("tail", 20.0)]), 15),
        ]),
        // Nyxal: tentacles gather, sink
        Species::Nyxal => PoseAnimation::new(vec![
            (pose_from(&[("tentacle_front_left", -15.0), ("tentacle_front_right", -15.0), ("mantle_base", -8.0)]), 8),
            (pose_from(&[("tentacle_front_left", -20.0), ("tentacle_front_right", -20.0), ("mantle_base", -12.0)]), 15),
        ]),
    }
}

fn clean_animation(species: &Species) -> PoseAnimation {
    match species {
        // Moluun: grooming motion (lick paw, rub face)
        Species::Moluun => PoseAnimation::new(vec![
            (pose_from(&[("shoulder_left", 20.0), ("neck", 15.0)]), 4),
            (pose_from(&[("shoulder_left", 10.0), ("neck", 5.0)]), 3),
            (pose_from(&[("shoulder_right", 20.0), ("neck", -15.0)]), 4),
            (pose_from(&[]), 3),
        ]),
        // Pylum: preening (beak to feathers)
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("neck", -20.0), ("wing_left", 25.0)]), 4),
            (pose_from(&[("neck", 20.0), ("wing_right", 25.0)]), 4),
            (pose_from(&[]), 3),
        ]),
        // Skael: shedding shake
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("neck", -10.0), ("tail", -15.0)]), 2),
            (pose_from(&[("neck", 10.0), ("tail", 15.0)]), 2),
            (pose_from(&[("neck", -10.0), ("tail", -15.0)]), 2),
            (pose_from(&[("neck", 10.0), ("tail", 15.0)]), 2),
            (pose_from(&[]), 3),
        ]),
        // Nyxal: ink expulsion (tentacles spread, mantle contracts)
        Species::Nyxal => PoseAnimation::new(vec![
            (pose_from(&[("mantle_base", -20.0)]), 3), // contract
            (pose_from(&[("tentacle_front_left", 35.0), ("tentacle_front_right", 35.0), ("mantle_base", 10.0)]), 3), // expel
            (pose_from(&[]), 4),
        ]),
    }
}
