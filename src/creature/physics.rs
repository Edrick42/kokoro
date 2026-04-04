//! Mini physics engine for creature movement.
//!
//! Provides gravity, ground collision, bounce, and buoyancy.
//! Land creatures fall and rest on the ground plane. Aquatic creatures
//! (Nyxal) float with spring-based buoyancy.
//!
//! Mood transitions trigger physical impulses: playful creatures jump,
//! sleeping ones slump, sick ones stumble.

use bevy::prelude::*;
use rand::Rng;

use crate::creature::species::CreatureRoot;
use crate::mind::{Mind, MoodState};

/// Ground plane Y position — above the button row with margin.
/// Window is 700px tall (center=0, bottom=-350). Buttons occupy ~80px
/// from bottom (-350 to -270). This leaves the creature resting above them.
pub const GROUND_Y: f32 = -230.0;

/// Physics state attached to each creature's root entity.
#[derive(Component)]
pub struct PhysicsBody {
    pub velocity: Vec2,
    pub gravity: f32,
    pub grounded: bool,
    pub ground_y: f32,
    pub bounce_factor: f32,
    pub friction: f32,
    pub buoyant: bool,
    pub buoyancy_target_y: f32,
    pub buoyancy_strength: f32,
}

impl PhysicsBody {
    /// Creates a physics body for land-dwelling creatures.
    pub fn land_creature(ground_y: f32) -> Self {
        Self {
            velocity: Vec2::ZERO,
            gravity: 400.0,
            grounded: false,
            ground_y,
            bounce_factor: 0.3,
            friction: 0.85,
            buoyant: false,
            buoyancy_target_y: 0.0,
            buoyancy_strength: 0.0,
        }
    }

    /// Creates a physics body for aquatic creatures (buoyant, no gravity).
    pub fn aquatic_creature(float_y: f32) -> Self {
        Self {
            velocity: Vec2::ZERO,
            gravity: 0.0,
            grounded: false,
            ground_y: float_y - 100.0, // soft floor below float target
            bounce_factor: 0.1,
            friction: 0.92,
            buoyant: true,
            buoyancy_target_y: float_y,
            buoyancy_strength: 120.0,
        }
    }
}

/// Tracks the previous mood so we can detect transitions.
#[derive(Resource, Default)]
struct PreviousMood(Option<MoodState>);

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PreviousMood::default())
           .add_systems(Update, (physics_actions, physics_step).chain());
    }
}

/// Applies impulses on mood transitions.
fn physics_actions(
    mind: Res<Mind>,
    mut prev: ResMut<PreviousMood>,
    mut query: Query<&mut PhysicsBody, With<CreatureRoot>>,
) {
    let current = mind.mood.clone();

    // Only act on transitions
    if prev.0.as_ref() == Some(&current) {
        return;
    }
    prev.0 = Some(current.clone());

    let mut rng = rand::rng();

    for mut body in query.iter_mut() {
        match current {
            MoodState::Playful => {
                // Jump!
                let jump_force = rng.random_range(80.0..150.0);
                body.velocity.y += jump_force;
                body.grounded = false;
            }
            MoodState::Sleeping => {
                // Slump down
                body.velocity.y -= 30.0;
            }
            MoodState::Sick => {
                // Random stumble
                let stumble = rng.random_range(-50.0..50.0);
                body.velocity.x += stumble;
            }
            _ => {}
        }
    }
}

/// Core physics step — runs every frame.
fn physics_step(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PhysicsBody), With<CreatureRoot>>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 || dt > 0.2 {
        return; // Skip on pause or huge frame spikes
    }

    for (mut transform, mut body) in query.iter_mut() {
        // --- Buoyancy (aquatic creatures) ---
        if body.buoyant {
            let displacement = body.buoyancy_target_y - transform.translation.y;
            body.velocity.y += displacement * body.buoyancy_strength * dt;
            body.velocity.y *= 0.98_f32.powf(dt * 60.0); // damping
        }

        // --- Gravity (land creatures) ---
        if !body.buoyant && !body.grounded {
            body.velocity.y -= body.gravity * dt;
        }

        // --- Integrate position ---
        transform.translation.x += body.velocity.x * dt;
        transform.translation.y += body.velocity.y * dt;

        // --- Ground collision ---
        if transform.translation.y < body.ground_y {
            transform.translation.y = body.ground_y;

            if body.velocity.y < -10.0 {
                // Bounce
                body.velocity.y = -body.velocity.y * body.bounce_factor;
            } else {
                body.velocity.y = 0.0;
                body.grounded = true;
            }
        }

        // --- Leave ground if pushed up ---
        if body.grounded && body.velocity.y > 1.0 {
            body.grounded = false;
        }

        // --- Horizontal friction (framerate-independent) ---
        body.velocity.x *= body.friction.powf(dt * 60.0);
        if body.velocity.x.abs() < 0.5 {
            body.velocity.x = 0.0;
        }

        // --- Keep creature on screen (soft walls) ---
        let max_x = 160.0;
        if transform.translation.x.abs() > max_x {
            transform.translation.x = transform.translation.x.clamp(-max_x, max_x);
            body.velocity.x = -body.velocity.x * 0.3;
        }
    }
}
