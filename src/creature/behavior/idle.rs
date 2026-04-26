//! Idle Behavior System — involuntary micro-behaviors driven by the autonomic nervous system.
//!
//! Real animals constantly perform small unconscious tells: yawning, stretching,
//! ear twitches, freezing, scratching. These correlate with internal state:
//!
//! - **Parasympathetic** (calm): yawn, stretch, slow blink, settle
//! - **Neutral**: ear twitch, look around, tail flick, sniff
//! - **Sympathetic** (alert): freeze, ears erect, rapid scan
//! - **Conflict** (both high): displacement scratch, shake off
//!
//! Each behavior may set a facial expression and/or apply soft-body impulses.
//! The timer interval depends on arousal: alert creatures fidget more,
//! calm creatures move less.

use bevy::prelude::*;
use rand::Rng;

use crate::config::autonomic as cfg;
use crate::creature::behavior::reactions::ExpressionOverride;
use crate::creature::interaction::soft_body::SoftBody;
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

fn idle_behavior_system(
    ans: Res<AutonomicState>,
    mind: Res<Mind>,
    genome: Res<Genome>,
    time: Res<Time>,
    mut timer: ResMut<IdleTimer>,
    mut elapsed: Local<f32>,
    mut expression: ResMut<ExpressionOverride>,
    soft_body: Option<ResMut<SoftBody>>,
) {
    // Count at ~1 tick per second (not per frame)
    *elapsed += time.delta_secs();
    if *elapsed < 1.0 { return; }
    *elapsed -= 1.0;

    // Tick down
    if timer.ticks_until_next > 0 {
        timer.ticks_until_next -= 1;
        return;
    }

    let mut rng = rand::rng();
    let behavior = pick_behavior(&ans, &mind.mood, &mut rng);

    timer.last_behavior = behavior.label();
    apply_idle_expression(behavior, &mut expression);
    if let Some(mut body) = soft_body {
        apply_idle_impulse(behavior, &genome.species, &mut body);
    }

    // Reset timer — interval depends on arousal
    let base_interval = rng.random_range(cfg::idle::MIN_INTERVAL..=cfg::idle::MAX_INTERVAL);
    let speed_mult = if ans.is_sympathetic() {
        cfg::idle::SYMPATHETIC_SPEED
    } else if ans.is_parasympathetic() {
        cfg::idle::PARASYMPATHETIC_SPEED
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
    Scratch, ShakeOff,
}

impl IdleBehavior {
    fn label(self) -> &'static str {
        match self {
            Self::Yawn => "yawn",
            Self::Stretch => "stretch",
            Self::SlowBlink => "slow blink",
            Self::Settle => "settle",
            Self::EarTwitch => "ear twitch",
            Self::LookAround => "look around",
            Self::TailFlick => "tail flick",
            Self::Sniff => "sniff",
            Self::Freeze => "freeze",
            Self::EarsErect => "ears erect",
            Self::RapidScan => "rapid scan",
            Self::Scratch => "scratch",
            Self::ShakeOff => "shake off",
        }
    }
}

fn pick_behavior(ans: &AutonomicState, mood: &MoodState, rng: &mut impl Rng) -> IdleBehavior {
    // Conflict overrides: displacement behaviors
    if ans.is_conflicted() && rng.random_bool(0.4) {
        return if rng.random_bool(0.5) { IdleBehavior::Scratch } else { IdleBehavior::ShakeOff };
    }

    // Sleeping: only slow blink
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
// EXPRESSION APPLICATION
// ===================================================================

fn apply_idle_expression(behavior: IdleBehavior, expression: &mut ExpressionOverride) {
    match behavior {
        IdleBehavior::Yawn => expression.set(2, 1, 0.0, 8),          // half-closed + mouth open
        IdleBehavior::SlowBlink => expression.set(2, 0, 0.0, 6),     // eyes close briefly
        IdleBehavior::Freeze => expression.set(1, 0, 0.0, 10),       // wide eyes, still
        IdleBehavior::EarsErect => expression.set(1, -1, 0.0, 8),    // wide eyes, tight mouth
        _ => {}
    }
}

// ===================================================================
// SOFT-BODY IMPULSES (species-specific micro-motion)
// ===================================================================

fn apply_idle_impulse(behavior: IdleBehavior, species: &Species, body: &mut SoftBody) {
    match species {
        Species::Moluun => moluun_idle(body, behavior),
        Species::Pylum  => pylum_idle(body, behavior),
        Species::Skael  => skael_idle(body, behavior),
        Species::Nyxal  => nyxal_idle(body, behavior),
    }
}

// ----- Moluun (mammal) -----

fn moluun_idle(body: &mut SoftBody, behavior: IdleBehavior) {
    match behavior {
        IdleBehavior::Stretch => {
            body.impulse("paw_l", Vec2::new(-4.0, -6.0));
            body.impulse("paw_r", Vec2::new(4.0, -6.0));
            body.impulse("head", Vec2::new(0.0, -4.0));
        }
        IdleBehavior::Settle => {
            body.impulse("belly", Vec2::new(0.0, 2.0));
            body.impulse("head", Vec2::new(0.0, 1.5));
        }
        IdleBehavior::EarTwitch => {
            body.impulse("ear_anchor", Vec2::new(2.0, -3.0));
        }
        IdleBehavior::LookAround => {
            body.impulse("head", Vec2::new(5.0, 0.0));
            body.impulse("ear_anchor", Vec2::new(3.0, -1.0));
        }
        IdleBehavior::TailFlick => {
            // Moluun has no tail point; tiny hip wobble instead
            body.impulse("foot_l", Vec2::new(1.5, 0.0));
            body.impulse("foot_r", Vec2::new(-1.5, 0.0));
        }
        IdleBehavior::Sniff => {
            body.impulse("head", Vec2::new(0.0, 3.0));
        }
        IdleBehavior::Yawn => {
            body.impulse("head", Vec2::new(0.0, -5.0));
        }
        IdleBehavior::RapidScan => {
            body.impulse("head", Vec2::new(6.0, -1.0));
            body.impulse("ear_anchor", Vec2::new(4.0, -2.0));
        }
        IdleBehavior::Scratch => {
            body.impulse("paw_l", Vec2::new(0.0, -8.0));
            body.impulse("head", Vec2::new(-2.0, 2.0));
        }
        IdleBehavior::ShakeOff => {
            body.impulse("head", Vec2::new(-6.0, 0.0));
            body.impulse("ear_anchor", Vec2::new(-4.0, 0.0));
            body.impulse("paw_l", Vec2::new(-2.0, 0.0));
            body.impulse("paw_r", Vec2::new(2.0, 0.0));
        }
        _ => {}
    }
}

// ----- Pylum (bird) -----

fn pylum_idle(body: &mut SoftBody, behavior: IdleBehavior) {
    match behavior {
        IdleBehavior::Stretch => {
            body.impulse("wingtip_l", Vec2::new(-8.0, -4.0));
            body.impulse("wingtip_r", Vec2::new(8.0, -4.0));
            body.impulse("tail", Vec2::new(0.0, -3.0));
        }
        IdleBehavior::Settle => {
            body.impulse("wingtip_l", Vec2::new(0.0, 2.0));
            body.impulse("wingtip_r", Vec2::new(0.0, 2.0));
        }
        IdleBehavior::EarTwitch => {
            body.impulse("casque", Vec2::new(2.0, -2.0));
        }
        IdleBehavior::LookAround => {
            body.impulse("head", Vec2::new(5.0, 0.0));
            body.impulse("casque", Vec2::new(3.0, 0.0));
        }
        IdleBehavior::TailFlick => {
            body.impulse("tail", Vec2::new(4.0, -1.0));
        }
        IdleBehavior::Sniff => {
            body.impulse("head", Vec2::new(0.0, 2.0));
        }
        IdleBehavior::Yawn => {
            body.impulse("head", Vec2::new(0.0, -3.0));
            body.impulse("wingtip_l", Vec2::new(-2.0, 0.0));
            body.impulse("wingtip_r", Vec2::new(2.0, 0.0));
        }
        IdleBehavior::RapidScan => {
            body.impulse("head", Vec2::new(7.0, -1.0));
            body.impulse("casque", Vec2::new(5.0, -1.0));
        }
        IdleBehavior::Scratch => {
            body.impulse("head", Vec2::new(-6.0, 2.0)); // preen
            body.impulse("wingtip_l", Vec2::new(3.0, 0.0));
        }
        IdleBehavior::ShakeOff => {
            body.impulse("wingtip_l", Vec2::new(-5.0, -3.0));
            body.impulse("wingtip_r", Vec2::new(5.0, -3.0));
            body.impulse("tail", Vec2::new(0.0, -2.0));
        }
        _ => {}
    }
}

// ----- Skael (reptile) -----

fn skael_idle(body: &mut SoftBody, behavior: IdleBehavior) {
    match behavior {
        IdleBehavior::Stretch => {
            body.impulse("head", Vec2::new(0.0, -3.0));
            body.impulse("tail_3", Vec2::new(0.0, -4.0));
        }
        IdleBehavior::Settle => {
            body.impulse("belly", Vec2::new(0.0, 2.0));
            body.impulse("tail_1", Vec2::new(0.0, 1.0));
        }
        IdleBehavior::EarTwitch => {
            body.impulse("horn_l", Vec2::new(1.5, 0.0));
            body.impulse("horn_r", Vec2::new(-1.5, 0.0));
        }
        IdleBehavior::LookAround => {
            body.impulse("head", Vec2::new(4.0, 0.0));
            body.impulse("horn_l", Vec2::new(3.0, 0.0));
            body.impulse("horn_r", Vec2::new(3.0, 0.0));
        }
        IdleBehavior::TailFlick => {
            body.impulse("tail_2", Vec2::new(3.0, 0.0));
            body.impulse("tail_3", Vec2::new(6.0, -2.0));
        }
        IdleBehavior::Sniff => {
            body.impulse("head", Vec2::new(0.0, 2.0));
        }
        IdleBehavior::Yawn => {
            body.impulse("head", Vec2::new(0.0, -2.0));
        }
        IdleBehavior::RapidScan => {
            body.impulse("head", Vec2::new(6.0, -1.0));
        }
        IdleBehavior::Scratch => {
            body.impulse("tail_3", Vec2::new(-5.0, -1.0));
            body.impulse("head", Vec2::new(-2.0, 1.0));
        }
        IdleBehavior::ShakeOff => {
            body.impulse("head", Vec2::new(-5.0, 0.0));
            body.impulse("tail_3", Vec2::new(6.0, 0.0));
            body.impulse("tail_2", Vec2::new(-4.0, 0.0));
        }
        _ => {}
    }
}

// ----- Nyxal (cephalopod) -----

fn nyxal_idle(body: &mut SoftBody, behavior: IdleBehavior) {
    match behavior {
        IdleBehavior::Stretch => {
            body.impulse("tip_fl", Vec2::new(-3.0, 4.0));
            body.impulse("tip_fr", Vec2::new(3.0, 4.0));
            body.impulse("tip_bl", Vec2::new(-4.0, 2.0));
            body.impulse("tip_br", Vec2::new(4.0, 2.0));
        }
        IdleBehavior::Settle => {
            body.impulse("mantle_top", Vec2::new(0.0, 2.0));
        }
        IdleBehavior::EarTwitch => {
            body.impulse("fin_l", Vec2::new(-2.0, 0.0));
            body.impulse("fin_r", Vec2::new(2.0, 0.0));
        }
        IdleBehavior::LookAround => {
            body.impulse("mantle_top", Vec2::new(3.0, 0.0));
            body.impulse("tip_fl", Vec2::new(2.0, 0.0));
            body.impulse("tip_fr", Vec2::new(2.0, 0.0));
        }
        IdleBehavior::TailFlick => {
            body.impulse("tip_bl", Vec2::new(-4.0, 0.0));
            body.impulse("tip_br", Vec2::new(4.0, 0.0));
        }
        IdleBehavior::Sniff => {
            body.impulse("tip_fl", Vec2::new(-1.5, 3.0));
            body.impulse("tip_fr", Vec2::new(1.5, 3.0));
        }
        IdleBehavior::Yawn => {
            body.impulse("mantle_top", Vec2::new(0.0, -3.0));
        }
        IdleBehavior::RapidScan => {
            body.impulse("mantle_top", Vec2::new(5.0, -1.0));
            body.impulse("tip_fl", Vec2::new(3.0, 0.0));
            body.impulse("tip_fr", Vec2::new(3.0, 0.0));
        }
        IdleBehavior::Scratch => {
            body.impulse("tip_fl", Vec2::new(3.0, 2.0));
            body.impulse("tip_fr", Vec2::new(-3.0, 2.0));
        }
        IdleBehavior::ShakeOff => {
            body.impulse("tip_fl", Vec2::new(-4.0, 0.0));
            body.impulse("tip_fr", Vec2::new(4.0, 0.0));
            body.impulse("tip_bl", Vec2::new(-5.0, 0.0));
            body.impulse("tip_br", Vec2::new(5.0, 0.0));
            body.impulse("mantle_top", Vec2::new(0.0, -2.0));
        }
        _ => {}
    }
}
