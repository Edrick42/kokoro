//! Creature impulses — one-shot forces applied to soft body points.
//!
//! Actions apply physics impulses. The spring system + distance constraints
//! handle the rest — head dips, springs pull back, body settles.

use bevy::prelude::*;

use super::soft_body::SoftBody;
use crate::creature::behavior::reactions::CreatureReaction;
use crate::game::state::AppState;

pub struct ImpulsePlugin;

impl Plugin for ImpulsePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_reaction_impulses.run_if(in_state(AppState::Gameplay)));
    }
}

fn apply_reaction_impulses(
    mut events: EventReader<CreatureReaction>,
    mut soft_body: Option<ResMut<SoftBody>>,
) {
    let Some(ref mut body) = soft_body else { return };

    for event in events.read() {
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
}
