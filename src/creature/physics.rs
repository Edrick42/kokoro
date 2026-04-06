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

/// Re-export ground Y from config for external use.
pub use crate::config::physics::GROUND_Y;

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
        use crate::config::physics::land;
        Self {
            velocity: Vec2::ZERO,
            gravity: land::GRAVITY,
            grounded: false,
            ground_y,
            bounce_factor: land::BOUNCE_FACTOR,
            friction: land::FRICTION,
            buoyant: false,
            buoyancy_target_y: 0.0,
            buoyancy_strength: 0.0,
        }
    }

    /// Creates a physics body for aquatic creatures (buoyant, no gravity).
    pub fn aquatic_creature(float_y: f32) -> Self {
        use crate::config::physics::aquatic;
        Self {
            velocity: Vec2::ZERO,
            gravity: 0.0,
            grounded: false,
            ground_y: float_y - aquatic::FLOOR_OFFSET,
            bounce_factor: aquatic::BOUNCE_FACTOR,
            friction: aquatic::FRICTION,
            buoyant: true,
            buoyancy_target_y: float_y,
            buoyancy_strength: aquatic::BUOYANCY_STRENGTH,
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

    use crate::config::physics::impulse;
    for mut body in query.iter_mut() {
        match current {
            MoodState::Playful => {
                let jump = rng.random_range(impulse::PLAYFUL_JUMP_MIN..impulse::PLAYFUL_JUMP_MAX);
                body.velocity.y += jump;
                body.grounded = false;
            }
            MoodState::Sleeping => {
                body.velocity.y += impulse::SLEEPING_SLUMP;
            }
            MoodState::Sick => {
                let stumble = rng.random_range(impulse::SICK_STUMBLE_MIN..impulse::SICK_STUMBLE_MAX);
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
    use crate::config::physics::{self as phys, threshold};

    let dt = time.delta_secs();
    if dt <= 0.0 || dt > phys::MAX_DT {
        return;
    }

    for (mut transform, mut body) in query.iter_mut() {
        // --- Buoyancy (aquatic creatures) ---
        if body.buoyant {
            let displacement = body.buoyancy_target_y - transform.translation.y;
            body.velocity.y += displacement * body.buoyancy_strength * dt;
            body.velocity.y *= phys::aquatic::DAMPING.powf(dt * 60.0);
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

            if body.velocity.y < threshold::BOUNCE_MIN {
                body.velocity.y = -body.velocity.y * body.bounce_factor;
            } else {
                body.velocity.y = 0.0;
                body.grounded = true;
            }
        }

        // --- Leave ground if pushed up ---
        if body.grounded && body.velocity.y > threshold::GROUNDED_MIN {
            body.grounded = false;
        }

        // --- Horizontal friction (framerate-independent) ---
        body.velocity.x *= body.friction.powf(dt * 60.0);
        if body.velocity.x.abs() < threshold::FRICTION_STOP {
            body.velocity.x = 0.0;
        }

        // --- Keep creature on screen (soft walls) ---
        if transform.translation.x.abs() > phys::MAX_X {
            transform.translation.x = transform.translation.x.clamp(-phys::MAX_X, phys::MAX_X);
            body.velocity.x = -body.velocity.x * phys::WALL_BOUNCE;
        }
    }
}
