//! Idle Behavior System — involuntary micro-animations driven by the autonomic nervous system.
//!
//! Real animals constantly perform small unconscious movements: blinking,
//! ear twitches, yawning, scratching, scanning for threats. These aren't
//! random — they correlate with the animal's internal state:
//!
//! - **Parasympathetic** (calm): stretch, yawn, slow blink, settle, purr
//! - **Neutral**: ear twitch, look around, tail flick, sniff
//! - **Sympathetic** (alert): freeze, ears erect, rapid scan, pupils dilate
//! - **Conflict** (both high): displacement scratch, sudden groom, shake off
//!
//! Each behavior is a short PoseAnimation (3-8 ticks) triggered on a timer.
//! The timer interval depends on arousal: alert creatures fidget more,
//! calm creatures move less.

use bevy::prelude::*;
use rand::Rng;

use crate::config::autonomic as cfg;
use crate::creature::pose::{ActiveAnimation, PoseAnimation, pose_from};
use crate::creature::reactions::ExpressionOverride;
use crate::game::state::AppState;
use crate::genome::{Genome, Species};
use crate::mind::autonomic::AutonomicState;
use crate::mind::MoodState;
use crate::mind::Mind;

/// Tracks the idle behavior timer.
#[derive(Resource)]
pub struct IdleTimer {
    pub ticks_until_next: u32,
    pub last_behavior: &'static str,
}

impl Default for IdleTimer {
    fn default() -> Self {
        Self { ticks_until_next: 60, last_behavior: "none" }
    }
}

pub struct IdleBehaviorPlugin;

impl Plugin for IdleBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(IdleTimer::default())
            .add_systems(Update, idle_behavior_system.run_if(in_state(AppState::Gameplay)));
    }
}

/// Picks and triggers idle micro-animations based on autonomic state.
fn idle_behavior_system(
    ans: Res<AutonomicState>,
    mind: Res<Mind>,
    genome: Res<Genome>,
    mut timer: ResMut<IdleTimer>,
    mut active_anim: ResMut<ActiveAnimation>,
    mut expression: ResMut<ExpressionOverride>,
) {
    // Don't interrupt player-triggered animations
    if active_anim.animation.is_some() { return; }

    // Tick down
    if timer.ticks_until_next > 0 {
        timer.ticks_until_next -= 1;
        return;
    }

    // Pick a behavior category based on ANS level
    let mut rng = rand::rng();
    let behavior = pick_behavior(&ans, &mind.mood, &mut rng);

    // Apply it
    let species = &genome.species;
    match behavior {
        // === Parasympathetic (calm) ===
        IdleBehavior::Yawn => {
            timer.last_behavior = "yawn";
            expression.set(2, 1, 0.0, 8); // half-closed eyes, mouth open
            active_anim.animation = Some(yawn_animation(species));
        }
        IdleBehavior::Stretch => {
            timer.last_behavior = "stretch";
            active_anim.animation = Some(stretch_animation(species));
        }
        IdleBehavior::SlowBlink => {
            timer.last_behavior = "slow blink";
            expression.set(2, 0, 0.0, 6); // eyes close slowly then open
        }
        IdleBehavior::Settle => {
            timer.last_behavior = "settle";
            active_anim.animation = Some(settle_animation(species));
        }

        // === Neutral ===
        IdleBehavior::EarTwitch => {
            timer.last_behavior = "ear twitch";
            active_anim.animation = Some(ear_twitch_animation(species));
        }
        IdleBehavior::LookAround => {
            timer.last_behavior = "look around";
            active_anim.animation = Some(look_around_animation(species));
        }
        IdleBehavior::TailFlick => {
            timer.last_behavior = "tail flick";
            active_anim.animation = Some(tail_flick_animation(species));
        }
        IdleBehavior::Sniff => {
            timer.last_behavior = "sniff";
            active_anim.animation = Some(sniff_animation(species));
        }

        // === Sympathetic (alert) ===
        IdleBehavior::Freeze => {
            timer.last_behavior = "freeze";
            expression.set(1, 0, 0.0, 10); // wide eyes, still
            // No pose animation — freeze IS the lack of movement
        }
        IdleBehavior::EarsErect => {
            timer.last_behavior = "ears erect";
            expression.set(1, -1, 0.0, 8); // wide eyes, mouth tight
        }
        IdleBehavior::RapidScan => {
            timer.last_behavior = "rapid scan";
            active_anim.animation = Some(rapid_scan_animation(species));
        }

        // === Conflict (displacement) ===
        IdleBehavior::DisplacementScratch => {
            timer.last_behavior = "scratch";
            active_anim.animation = Some(scratch_animation(species));
        }
        IdleBehavior::ShakeOff => {
            timer.last_behavior = "shake off";
            active_anim.animation = Some(shake_off_animation(species));
        }
    }

    // Reset timer — interval depends on arousal
    let base_interval = rng.random_range(cfg::idle::MIN_INTERVAL..=cfg::idle::MAX_INTERVAL);
    let speed_mult = if ans.is_sympathetic() {
        cfg::idle::SYMPATHETIC_SPEED       // 0.5 — fidget more
    } else if ans.is_parasympathetic() {
        cfg::idle::PARASYMPATHETIC_SPEED   // 1.8 — move less
    } else {
        1.0
    };
    timer.ticks_until_next = (base_interval as f32 * speed_mult) as u32;
}

// ===================================================================
// BEHAVIOR SELECTION
// ===================================================================

#[derive(Debug, Clone, Copy)]
enum IdleBehavior {
    // Parasympathetic
    Yawn, Stretch, SlowBlink, Settle,
    // Neutral
    EarTwitch, LookAround, TailFlick, Sniff,
    // Sympathetic
    Freeze, EarsErect, RapidScan,
    // Conflict
    DisplacementScratch, ShakeOff,
}

fn pick_behavior(ans: &AutonomicState, mood: &MoodState, rng: &mut impl Rng) -> IdleBehavior {
    // Conflict overrides: displacement behaviors
    if ans.is_conflicted() && rng.random_bool(0.4) {
        return if rng.random_bool(0.5) {
            IdleBehavior::DisplacementScratch
        } else {
            IdleBehavior::ShakeOff
        };
    }

    // Sleeping: only slow blink (almost nothing)
    if *mood == MoodState::Sleeping {
        return IdleBehavior::SlowBlink;
    }

    let pool: &[IdleBehavior] = if ans.is_parasympathetic() {
        &[IdleBehavior::Yawn, IdleBehavior::Stretch, IdleBehavior::SlowBlink, IdleBehavior::Settle]
    } else if ans.is_sympathetic() {
        &[IdleBehavior::Freeze, IdleBehavior::EarsErect, IdleBehavior::RapidScan,
          IdleBehavior::EarTwitch, IdleBehavior::LookAround]
    } else {
        &[IdleBehavior::EarTwitch, IdleBehavior::LookAround, IdleBehavior::TailFlick,
          IdleBehavior::Sniff, IdleBehavior::Yawn]
    };

    pool[rng.random_range(0..pool.len())]
}

// ===================================================================
// MICRO-ANIMATIONS (species-specific, short duration)
// ===================================================================

fn yawn_animation(species: &Species) -> PoseAnimation {
    match species {
        Species::Moluun => PoseAnimation::new(vec![
            (pose_from(&[("neck", -10.0)]), 3),  // head tilts back
            (pose_from(&[("neck", -15.0)]), 4),  // hold (mouth open via expression)
            (pose_from(&[]), 3),
        ]),
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("neck", -12.0), ("wing_left", 8.0), ("wing_right", 8.0)]), 4),
            (pose_from(&[]), 3),
        ]),
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("neck", -8.0), ("tail", -5.0)]), 5), // slow yawn
            (pose_from(&[]), 4),
        ]),
        Species::Nyxal => PoseAnimation::new(vec![
            (pose_from(&[("mantle_base", 8.0)]), 4), // mantle expands (deep "breath")
            (pose_from(&[]), 3),
        ]),
    }
}

fn stretch_animation(species: &Species) -> PoseAnimation {
    match species {
        Species::Moluun => PoseAnimation::new(vec![
            (pose_from(&[("shoulder_left", -15.0), ("shoulder_right", -15.0), ("hip_left", 10.0), ("hip_right", 10.0)]), 5),
            (pose_from(&[]), 4),
        ]),
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("wing_left", 35.0), ("wing_right", 35.0), ("keel_joint", -8.0)]), 5),
            (pose_from(&[]), 4),
        ]),
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("tail", 20.0), ("neck", 8.0)]), 6), // arch back
            (pose_from(&[]), 4),
        ]),
        Species::Nyxal => PoseAnimation::new(vec![
            (pose_from(&[("tentacle_front_left", 25.0), ("tentacle_front_right", -25.0), ("mantle_base", 5.0)]), 5),
            (pose_from(&[]), 4),
        ]),
    }
}

fn settle_animation(species: &Species) -> PoseAnimation {
    match species {
        Species::Moluun => PoseAnimation::new(vec![
            (pose_from(&[("hip_left", 5.0), ("hip_right", 5.0)]), 4),
            (pose_from(&[("hip_left", 3.0), ("hip_right", 3.0)]), 6), // slight settle
            (pose_from(&[]), 3),
        ]),
        _ => PoseAnimation::new(vec![
            (pose_from(&[("neck", 3.0)]), 5),
            (pose_from(&[]), 3),
        ]),
    }
}

fn ear_twitch_animation(_species: &Species) -> PoseAnimation {
    // Ear twitch: quick neck jolt (ears are on the head)
    PoseAnimation::new(vec![
        (pose_from(&[("neck", 5.0)]), 2),
        (pose_from(&[("neck", -3.0)]), 2),
        (pose_from(&[]), 2),
    ])
}

fn look_around_animation(_species: &Species) -> PoseAnimation {
    PoseAnimation::new(vec![
        (pose_from(&[("neck", -12.0)]), 4),  // look left
        (pose_from(&[("neck", 0.0)]), 2),     // center
        (pose_from(&[("neck", 12.0)]), 4),   // look right
        (pose_from(&[]), 3),
    ])
}

fn tail_flick_animation(species: &Species) -> PoseAnimation {
    match species {
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("tail", 15.0)]), 2),
            (pose_from(&[("tail", -10.0)]), 2),
            (pose_from(&[]), 2),
        ]),
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("tail", 10.0)]), 2),
            (pose_from(&[("tail", -8.0)]), 2),
            (pose_from(&[]), 2),
        ]),
        _ => PoseAnimation::new(vec![
            (pose_from(&[("hip_left", 3.0)]), 2),
            (pose_from(&[("hip_right", 3.0)]), 2),
            (pose_from(&[]), 2),
        ]),
    }
}

fn sniff_animation(_species: &Species) -> PoseAnimation {
    PoseAnimation::new(vec![
        (pose_from(&[("neck", 8.0)]), 2),   // head forward (sniffing)
        (pose_from(&[("neck", 10.0)]), 2),  // hold
        (pose_from(&[("neck", 6.0)]), 2),
        (pose_from(&[]), 2),
    ])
}

fn rapid_scan_animation(_species: &Species) -> PoseAnimation {
    PoseAnimation::new(vec![
        (pose_from(&[("neck", -15.0)]), 2),  // snap left
        (pose_from(&[("neck", 15.0)]), 2),   // snap right
        (pose_from(&[("neck", -8.0)]), 2),   // back left
        (pose_from(&[]), 2),
    ])
}

fn scratch_animation(species: &Species) -> PoseAnimation {
    match species {
        Species::Moluun => PoseAnimation::new(vec![
            (pose_from(&[("shoulder_left", 20.0), ("neck", 10.0)]), 2),
            (pose_from(&[("shoulder_left", 15.0), ("neck", 8.0)]), 2),
            (pose_from(&[("shoulder_left", 20.0), ("neck", 10.0)]), 2),
            (pose_from(&[("shoulder_left", 15.0)]), 2),
            (pose_from(&[]), 2),
        ]),
        Species::Pylum => PoseAnimation::new(vec![
            (pose_from(&[("keel_joint", 10.0), ("neck", -15.0)]), 2), // peck at self
            (pose_from(&[("keel_joint", 8.0), ("neck", -10.0)]), 2),
            (pose_from(&[("keel_joint", 10.0), ("neck", -15.0)]), 2),
            (pose_from(&[]), 2),
        ]),
        Species::Skael => PoseAnimation::new(vec![
            (pose_from(&[("tail", -15.0), ("neck", -8.0)]), 3), // rub against something
            (pose_from(&[("tail", -10.0)]), 3),
            (pose_from(&[]), 2),
        ]),
        Species::Nyxal => PoseAnimation::new(vec![
            (pose_from(&[("tentacle_front_left", 20.0), ("tentacle_front_right", 20.0)]), 3),
            (pose_from(&[("tentacle_front_left", 10.0), ("tentacle_front_right", 10.0)]), 2),
            (pose_from(&[]), 2),
        ]),
    }
}

fn shake_off_animation(_species: &Species) -> PoseAnimation {
    PoseAnimation::new(vec![
        (pose_from(&[("neck", -8.0), ("shoulder_left", -5.0), ("shoulder_right", 5.0)]), 2),
        (pose_from(&[("neck", 8.0), ("shoulder_left", 5.0), ("shoulder_right", -5.0)]), 2),
        (pose_from(&[("neck", -6.0)]), 2),
        (pose_from(&[("neck", 6.0)]), 2),
        (pose_from(&[]), 2),
    ])
}
