//! Creature impulses — one-shot forces applied to soft body points.
//!
//! Each species reacts differently to the same event:
//! - Moluun (mammal): head dips, ears flop, paws reach
//! - Pylum (bird): wings flap, tail bobs, casque dips
//! - Skael (reptile): tail whips, body hunkers, horns display
//! - Nyxal (cephalopod): tentacles spread/curl, mantle pulses, fins flutter

use bevy::prelude::*;

use super::soft_body::SoftBody;
use crate::creature::behavior::reactions::CreatureReaction;
use crate::game::state::AppState;
use crate::genome::{Genome, Species};

pub struct ImpulsePlugin;

impl Plugin for ImpulsePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_reaction_impulses.run_if(in_state(AppState::Gameplay)));
    }
}

fn apply_reaction_impulses(
    mut events: EventReader<CreatureReaction>,
    mut soft_body: Option<ResMut<SoftBody>>,
    genome: Res<Genome>,
) {
    let Some(ref mut body) = soft_body else { return };

    for event in events.read() {
        match &genome.species {
            Species::Moluun => moluun_impulse(body, event),
            Species::Pylum  => pylum_impulse(body, event),
            Species::Skael  => skael_impulse(body, event),
            Species::Nyxal  => nyxal_impulse(body, event),
        }
    }
}

// ===================================================================
// MOLUUN — mammal: head dips, ears flop, paws reach
// ===================================================================

fn moluun_impulse(body: &mut SoftBody, event: &CreatureReaction) {
    match event {
        CreatureReaction::Eating { preferred, .. } => {
            let f = if *preferred { 20.0 } else { 15.0 };
            body.impulse("head", Vec2::new(0.0, f));
            body.impulse("paw_l", Vec2::new(1.0, 3.0));
            body.impulse("paw_r", Vec2::new(-1.0, 3.0));
        }
        CreatureReaction::RefusingFood => {
            body.impulse("head", Vec2::new(0.0, -18.0));
            body.impulse("paw_l", Vec2::new(-3.0, 0.0));
            body.impulse("paw_r", Vec2::new(3.0, 0.0));
        }
        CreatureReaction::PlayStart => {
            body.impulse("foot_l", Vec2::new(0.0, -15.0));
            body.impulse("foot_r", Vec2::new(0.0, -15.0));
            body.impulse("head", Vec2::new(0.0, -12.0));
            body.impulse("paw_l", Vec2::new(-8.0, -8.0));
            body.impulse("paw_r", Vec2::new(8.0, -8.0));
        }
        CreatureReaction::Petted { pleasure } => {
            let f = pleasure * 10.0;
            body.impulse("head", Vec2::new(0.0, f));
            body.impulse("paw_l", Vec2::new(0.0, 2.0));
            body.impulse("paw_r", Vec2::new(0.0, 2.0));
        }
        CreatureReaction::Flinched { .. } => {
            body.impulse("head", Vec2::new(0.0, -22.0));
            body.impulse("ear_anchor", Vec2::new(0.0, -12.0));
            body.impulse("paw_l", Vec2::new(-5.0, -5.0));
            body.impulse("paw_r", Vec2::new(5.0, -5.0));
        }
        CreatureReaction::FallingAsleep => {
            body.impulse("head", Vec2::new(0.0, 8.0));
            body.impulse("paw_l", Vec2::new(0.0, 5.0));
            body.impulse("paw_r", Vec2::new(0.0, 5.0));
            body.impulse("ear_anchor", Vec2::new(0.0, 4.0));
        }
        CreatureReaction::WakingUp => {
            body.impulse("head", Vec2::new(0.0, -10.0));
            body.impulse("ear_anchor", Vec2::new(0.0, -8.0));
        }
        CreatureReaction::Cleaning => {
            body.impulse("head", Vec2::new(8.0, 0.0));
            body.impulse("paw_l", Vec2::new(0.0, -10.0));
        }
        CreatureReaction::GotSick => {
            body.impulse("head", Vec2::new(0.0, 5.0));
            body.impulse("belly", Vec2::new(0.0, 3.0));
        }
        CreatureReaction::Recovered => {
            body.impulse("head", Vec2::new(0.0, -12.0));
            body.impulse("ear_anchor", Vec2::new(0.0, -8.0));
            body.impulse("paw_l", Vec2::new(-3.0, -5.0));
            body.impulse("paw_r", Vec2::new(3.0, -5.0));
        }
    }
}

// ===================================================================
// PYLUM — bird: wings flap, beak pecks, tail bobs, casque dips
// ===================================================================

fn pylum_impulse(body: &mut SoftBody, event: &CreatureReaction) {
    match event {
        CreatureReaction::Eating { preferred, .. } => {
            // Sharp peck: head darts down fast, tail counterbalances up
            let f = if *preferred { 25.0 } else { 18.0 };
            body.impulse("head", Vec2::new(0.0, f));
            body.impulse("casque", Vec2::new(0.0, f * 0.8));
            body.impulse("tail", Vec2::new(0.0, -8.0));
        }
        CreatureReaction::RefusingFood => {
            // Head shake: lateral head motion, wings tighten
            body.impulse("head", Vec2::new(12.0, -10.0));
            body.impulse("wing_l", Vec2::new(3.0, 0.0));
            body.impulse("wing_r", Vec2::new(-3.0, 0.0));
        }
        CreatureReaction::PlayStart => {
            // Wing spread + jump: wings out wide, feet push off
            body.impulse("wingtip_l", Vec2::new(-15.0, -12.0));
            body.impulse("wingtip_r", Vec2::new(15.0, -12.0));
            body.impulse("wing_l", Vec2::new(-8.0, -8.0));
            body.impulse("wing_r", Vec2::new(8.0, -8.0));
            body.impulse("foot_l", Vec2::new(0.0, -18.0));
            body.impulse("foot_r", Vec2::new(0.0, -18.0));
            body.impulse("tail", Vec2::new(0.0, -10.0));
        }
        CreatureReaction::Petted { pleasure } => {
            // Wing droop (comfort): wings lower, head tilts
            let f = pleasure * 8.0;
            body.impulse("wingtip_l", Vec2::new(0.0, f));
            body.impulse("wingtip_r", Vec2::new(0.0, f));
            body.impulse("head", Vec2::new(0.0, f * 0.5));
        }
        CreatureReaction::Flinched { .. } => {
            // Startle display: wings snap open wide, head pulls back
            body.impulse("wingtip_l", Vec2::new(-20.0, -10.0));
            body.impulse("wingtip_r", Vec2::new(20.0, -10.0));
            body.impulse("head", Vec2::new(0.0, -15.0));
            body.impulse("casque", Vec2::new(0.0, -12.0));
            body.impulse("tail", Vec2::new(0.0, 8.0));
        }
        CreatureReaction::FallingAsleep => {
            // Tuck: head dips under wing, wings fold in
            body.impulse("head", Vec2::new(5.0, 8.0));
            body.impulse("wingtip_l", Vec2::new(4.0, 5.0));
            body.impulse("wingtip_r", Vec2::new(-4.0, 5.0));
            body.impulse("tail", Vec2::new(0.0, 3.0));
        }
        CreatureReaction::WakingUp => {
            // Stretch: wings spread, head up
            body.impulse("head", Vec2::new(0.0, -12.0));
            body.impulse("wingtip_l", Vec2::new(-8.0, -5.0));
            body.impulse("wingtip_r", Vec2::new(8.0, -5.0));
        }
        CreatureReaction::Cleaning => {
            // Preening: beak to wing, alternating
            body.impulse("head", Vec2::new(-10.0, 5.0));
            body.impulse("wing_l", Vec2::new(5.0, 0.0));
        }
        CreatureReaction::GotSick => {
            // Puffed up: wings droop, head sags
            body.impulse("head", Vec2::new(0.0, 6.0));
            body.impulse("wingtip_l", Vec2::new(0.0, 8.0));
            body.impulse("wingtip_r", Vec2::new(0.0, 8.0));
        }
        CreatureReaction::Recovered => {
            // Shake off: full body ruffle, wings flap
            body.impulse("head", Vec2::new(0.0, -10.0));
            body.impulse("wingtip_l", Vec2::new(-10.0, -12.0));
            body.impulse("wingtip_r", Vec2::new(10.0, -12.0));
            body.impulse("tail", Vec2::new(0.0, -8.0));
        }
    }
}

// ===================================================================
// SKAEL — reptile: tail whips, body hunkers, armored stillness
// ===================================================================

fn skael_impulse(body: &mut SoftBody, event: &CreatureReaction) {
    match event {
        CreatureReaction::Eating { preferred, .. } => {
            // Slow lunge: head extends forward, tail braces
            let f = if *preferred { 15.0 } else { 10.0 };
            body.impulse("head", Vec2::new(0.0, f));
            body.impulse("tail_3", Vec2::new(0.0, -5.0));
        }
        CreatureReaction::RefusingFood => {
            // Head turn: slow, deliberate rejection
            body.impulse("head", Vec2::new(8.0, -5.0));
            body.impulse("tail_1", Vec2::new(-3.0, 0.0));
        }
        CreatureReaction::PlayStart => {
            // Tail slam + lunge: wind up tail, slam it down
            body.impulse("tail_3", Vec2::new(0.0, -20.0));
            body.impulse("tail_2", Vec2::new(0.0, -12.0));
            body.impulse("head", Vec2::new(0.0, -8.0));
            body.impulse("foot_l", Vec2::new(0.0, -10.0));
            body.impulse("foot_r", Vec2::new(0.0, -10.0));
        }
        CreatureReaction::Petted { pleasure } => {
            // Trust = stillness: reptiles show comfort by NOT moving.
            // Only the tail tip relaxes down slowly.
            let f = pleasure * 5.0;
            body.impulse("tail_3", Vec2::new(0.0, f));
            body.impulse("tail_2", Vec2::new(0.0, f * 0.5));
        }
        CreatureReaction::Flinched { .. } => {
            // Defensive: tail whip + head tuck behind armor
            body.impulse("tail_3", Vec2::new(15.0, -15.0));
            body.impulse("tail_2", Vec2::new(8.0, -8.0));
            body.impulse("head", Vec2::new(0.0, 5.0)); // tuck head DOWN (protect)
            body.impulse("horn_l", Vec2::new(-3.0, 5.0));
            body.impulse("horn_r", Vec2::new(3.0, 5.0));
        }
        CreatureReaction::FallingAsleep => {
            // Lower to ground: everything sags, tail wraps
            body.impulse("head", Vec2::new(0.0, 6.0));
            body.impulse("tail_1", Vec2::new(3.0, 4.0));
            body.impulse("tail_2", Vec2::new(5.0, 3.0));
            body.impulse("tail_3", Vec2::new(6.0, 2.0));
            body.impulse("leg_l", Vec2::new(0.0, 3.0));
            body.impulse("leg_r", Vec2::new(0.0, 3.0));
        }
        CreatureReaction::WakingUp => {
            // Slow rise: head lifts first, then body follows
            body.impulse("head", Vec2::new(0.0, -8.0));
            body.impulse("tail_3", Vec2::new(0.0, -4.0));
        }
        CreatureReaction::Cleaning => {
            // Shedding shake: rapid lateral oscillation
            body.impulse("head", Vec2::new(-8.0, 0.0));
            body.impulse("tail_3", Vec2::new(10.0, 0.0));
            body.impulse("tail_2", Vec2::new(-6.0, 0.0));
        }
        CreatureReaction::GotSick => {
            // Hunker: body lowers, tail curls protectively
            body.impulse("head", Vec2::new(0.0, 4.0));
            body.impulse("tail_3", Vec2::new(5.0, 3.0));
            body.impulse("belly", Vec2::new(0.0, 3.0));
        }
        CreatureReaction::Recovered => {
            // Tail display: aggressive tail snap (I'm back!)
            body.impulse("tail_3", Vec2::new(0.0, -18.0));
            body.impulse("tail_2", Vec2::new(0.0, -10.0));
            body.impulse("head", Vec2::new(0.0, -6.0));
        }
    }
}

// ===================================================================
// NYXAL — cephalopod: tentacles flow, mantle pulses, everything undulates
// ===================================================================

fn nyxal_impulse(body: &mut SoftBody, event: &CreatureReaction) {
    match event {
        CreatureReaction::Eating { preferred, .. } => {
            // Tentacles gather food: inner pair reaches, outer pair steadies
            let f = if *preferred { 15.0 } else { 10.0 };
            body.impulse("tent_fl", Vec2::new(2.0, f));
            body.impulse("tent_fr", Vec2::new(-2.0, f));
            body.impulse("mantle_top", Vec2::new(0.0, -3.0)); // mantle contracts
        }
        CreatureReaction::RefusingFood => {
            // Tentacles retract: all pull inward and up
            body.impulse("tent_fl", Vec2::new(3.0, -12.0));
            body.impulse("tent_fr", Vec2::new(-3.0, -12.0));
            body.impulse("tent_bl", Vec2::new(5.0, -8.0));
            body.impulse("tent_br", Vec2::new(-5.0, -8.0));
            body.impulse("mantle_top", Vec2::new(0.0, -5.0));
        }
        CreatureReaction::PlayStart => {
            // Tentacle spin: all tips spread out in different directions
            body.impulse("tip_fl", Vec2::new(-10.0, 15.0));
            body.impulse("tip_fr", Vec2::new(10.0, 15.0));
            body.impulse("tip_bl", Vec2::new(-15.0, -8.0));
            body.impulse("tip_br", Vec2::new(15.0, -8.0));
            body.impulse("mantle_top", Vec2::new(0.0, -10.0));
            body.impulse("fin_l", Vec2::new(-5.0, 0.0));
            body.impulse("fin_r", Vec2::new(5.0, 0.0));
        }
        CreatureReaction::Petted { pleasure } => {
            // Tentacles reach toward touch: slow, gentle extension
            let f = pleasure * 8.0;
            body.impulse("tent_fl", Vec2::new(0.0, f));
            body.impulse("tent_fr", Vec2::new(0.0, f));
            body.impulse("fin_l", Vec2::new(-2.0, 0.0));
            body.impulse("fin_r", Vec2::new(2.0, 0.0));
        }
        CreatureReaction::Flinched { .. } => {
            // Ink defense: tentacles retract FAST, mantle contracts
            body.impulse("tent_fl", Vec2::new(4.0, -18.0));
            body.impulse("tent_fr", Vec2::new(-4.0, -18.0));
            body.impulse("tent_bl", Vec2::new(8.0, -15.0));
            body.impulse("tent_br", Vec2::new(-8.0, -15.0));
            body.impulse("mantle_top", Vec2::new(0.0, -12.0));
            body.impulse("fin_l", Vec2::new(5.0, -5.0));
            body.impulse("fin_r", Vec2::new(-5.0, -5.0));
        }
        CreatureReaction::FallingAsleep => {
            // Sink: everything drifts downward gently, tentacles gather
            body.impulse("tent_fl", Vec2::new(2.0, 5.0));
            body.impulse("tent_fr", Vec2::new(-2.0, 5.0));
            body.impulse("tent_bl", Vec2::new(3.0, 4.0));
            body.impulse("tent_br", Vec2::new(-3.0, 4.0));
            body.impulse("mantle_top", Vec2::new(0.0, 3.0));
        }
        CreatureReaction::WakingUp => {
            // Pulse awake: mantle expands, tentacles unfurl
            body.impulse("mantle_top", Vec2::new(0.0, -10.0));
            body.impulse("tent_fl", Vec2::new(-3.0, 5.0));
            body.impulse("tent_fr", Vec2::new(3.0, 5.0));
            body.impulse("fin_l", Vec2::new(-4.0, 0.0));
            body.impulse("fin_r", Vec2::new(4.0, 0.0));
        }
        CreatureReaction::Cleaning => {
            // Ink expulsion: mantle contracts then expands
            body.impulse("mantle_top", Vec2::new(0.0, 8.0));
            body.impulse("tent_fl", Vec2::new(-6.0, 10.0));
            body.impulse("tent_fr", Vec2::new(6.0, 10.0));
        }
        CreatureReaction::GotSick => {
            // Pale: tentacles droop, mantle deflates
            body.impulse("tip_fl", Vec2::new(0.0, 8.0));
            body.impulse("tip_fr", Vec2::new(0.0, 8.0));
            body.impulse("tip_bl", Vec2::new(0.0, 6.0));
            body.impulse("tip_br", Vec2::new(0.0, 6.0));
            body.impulse("mantle_top", Vec2::new(0.0, 3.0));
        }
        CreatureReaction::Recovered => {
            // Pulse of life: full body expansion, all tips glow
            body.impulse("mantle_top", Vec2::new(0.0, -12.0));
            body.impulse("tip_fl", Vec2::new(-5.0, -10.0));
            body.impulse("tip_fr", Vec2::new(5.0, -10.0));
            body.impulse("tip_bl", Vec2::new(-8.0, -8.0));
            body.impulse("tip_br", Vec2::new(8.0, -8.0));
            body.impulse("fin_l", Vec2::new(-6.0, 0.0));
            body.impulse("fin_r", Vec2::new(6.0, 0.0));
        }
    }
}
