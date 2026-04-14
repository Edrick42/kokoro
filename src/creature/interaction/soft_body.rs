//! Soft Body Physics — Rain World style point-spring system.
//!
//! Each creature is a set of points (head, body, feet, paws) connected by
//! springs (neck, arms, legs). Physics runs every frame: gravity pulls,
//! springs restore, friction grounds. The pixel art draws on top.
//!
//! This replaces the old pose-angle system with emergent movement:
//! an impulse on the head point creates a natural dip, the neck spring
//! stretches, the body leans, and everything returns when the impulse stops.

use std::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::state::AppState;

// ===================================================================
// DATA STRUCTURES
// ===================================================================

/// A physical point in the soft body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SoftPoint {
    pub name: String,
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    /// Rest position (where the point wants to be when no forces act)
    pub rest: Vec2,
    /// Is this point touching the ground?
    pub grounded: bool,
    /// Should this point be pinned (immovable)? Used for body anchor.
    pub pinned: bool,
}

impl SoftPoint {
    pub fn new(name: &str, x: f32, y: f32, mass: f32) -> Self {
        Self {
            name: name.to_string(),
            position: Vec2::new(x, y),
            velocity: Vec2::ZERO,
            mass,
            rest: Vec2::new(x, y),
            grounded: false,
            pinned: false,
        }
    }

    /// Pixel coordinates for drawing (i32).
    pub fn px(&self) -> (i32, i32) {
        (self.position.x as i32, self.position.y as i32)
    }
}

/// A spring connecting two points.
#[derive(Debug, Clone)]
pub struct Spring {
    /// Index of point A in the points array.
    pub a: usize,
    /// Index of point B in the points array.
    pub b: usize,
    /// Natural length when no force is applied.
    pub rest_length: f32,
    /// How strongly the spring pulls back (higher = stiffer).
    pub stiffness: f32,
    /// Damping ratio (0.0 = no damping, 1.0 = critical).
    pub damping: f32,
}

/// Complete soft body for one creature.
#[derive(Resource, Debug, Clone)]
#[allow(dead_code)]
pub struct SoftBody {
    pub points: Vec<SoftPoint>,
    pub springs: Vec<Spring>,
    /// Name → index lookup.
    index: HashMap<String, usize>,
}

impl SoftBody {
    pub fn new(points: Vec<SoftPoint>, springs: Vec<Spring>) -> Self {
        let index = points.iter().enumerate()
            .map(|(i, p)| (p.name.clone(), i))
            .collect();
        Self { points, springs, index }
    }

    /// Get a point by name. Returns the body anchor as fallback if name not found.
    /// This prevents panics when switching growth stages (e.g., Cub has no shoulders).
    pub fn point(&self, name: &str) -> &SoftPoint {
        let idx = self.index.get(name)
            .or_else(|| self.index.get("body"))
            .copied()
            .unwrap_or(0);
        &self.points[idx]
    }

    /// Get a mutable point by name. Returns body anchor as fallback.
    pub fn point_mut(&mut self, name: &str) -> &mut SoftPoint {
        let idx = self.index.get(name)
            .or_else(|| self.index.get("body"))
            .copied()
            .unwrap_or(0);
        &mut self.points[idx]
    }

    /// Apply an impulse (velocity change) to a named point.
    pub fn impulse(&mut self, name: &str, force: Vec2) {
        if let Some(&idx) = self.index.get(name) {
            self.points[idx].velocity += force;
        }
    }
}

// ===================================================================
// PHYSICS CONSTANTS
// ===================================================================

/// Gravity — VERY gentle for a 64px canvas. Just enough to settle points downward.
const GRAVITY: f32 = 20.0;

/// Global damping — high to prevent oscillation (creature should be stable, not jelly).
const DAMPING: f32 = 8.0;

/// Ground friction (0.0 = ice, 1.0 = instant stop).
const GROUND_FRICTION: f32 = 0.9;

/// Ground Y position in the 64×64 canvas.
const GROUND_Y: f32 = 52.0;

/// Maximum velocity — low to prevent parts from flying off.
const MAX_VELOCITY: f32 = 30.0;

/// Maximum stretch ratio — springs CANNOT stretch beyond this × rest_length.
/// This prevents disconnection. Like a bone — it has a maximum length.
const MAX_STRETCH_RATIO: f32 = 1.2;

// ===================================================================
// PLUGIN
// ===================================================================

pub struct SoftBodyPlugin;

impl Plugin for SoftBodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            reinit_on_stage_change,
            soft_body_step,
        ).chain().run_if(in_state(AppState::Gameplay)));
    }
}

/// Reinitializes the soft body when the growth stage changes.
/// This prevents panics when switching stages (e.g., Cub→Young) because
/// different stages have different point layouts.
fn reinit_on_stage_change(
    mut commands: Commands,
    growth: Res<crate::visuals::evolution::GrowthState>,
    genome: Res<crate::genome::Genome>,
    mut prev_stage: Local<Option<crate::visuals::evolution::GrowthStage>>,
) {
    let current = growth.stage;
    let changed = prev_stage.map_or(true, |prev| prev != current);
    if !changed { return; }
    *prev_stage = Some(current);

    use crate::genome::Species;
    use crate::visuals::evolution::GrowthStage;

    let new_body = match (&genome.species, current) {
        (Species::Moluun, GrowthStage::Cub) => moluun_cub(),
        (Species::Moluun, _) => moluun_adult(),
        _ => moluun_adult(), // TODO: other species
    };
    commands.insert_resource(new_body);
}

// ===================================================================
// PHYSICS STEP — runs every frame
// ===================================================================

fn soft_body_step(
    time: Res<Time>,
    mut body: Option<ResMut<SoftBody>>,
) {
    let Some(ref mut body) = body else { return };
    let dt = time.delta_secs().min(0.033); // cap at ~30fps minimum

    let num_points = body.points.len();

    // 1. Gravity (only non-pinned, non-grounded points)
    for point in &mut body.points {
        if !point.pinned && !point.grounded {
            point.velocity.y += GRAVITY * dt;
        }
    }

    // 2. Spring forces (Hooke's law + damping)
    // Collect forces first to avoid borrow issues
    let mut forces = vec![Vec2::ZERO; num_points];

    for spring in &body.springs {
        let a_pos = body.points[spring.a].position;
        let b_pos = body.points[spring.b].position;
        let a_vel = body.points[spring.a].velocity;
        let b_vel = body.points[spring.b].velocity;
        let a_mass = body.points[spring.a].mass;
        let b_mass = body.points[spring.b].mass;

        let delta = b_pos - a_pos;
        let distance = delta.length();
        if distance < 0.001 { continue; } // prevent division by zero

        let direction = delta / distance;
        let displacement = distance - spring.rest_length;

        // Hooke's force: F = -k * displacement
        let spring_force = spring.stiffness * displacement;

        // Damping force: F = -c * relative_velocity_along_spring
        let relative_vel = b_vel - a_vel;
        let damping_force = spring.damping * relative_vel.dot(direction);

        let total_force = (spring_force + damping_force) * direction;

        if !body.points[spring.a].pinned {
            forces[spring.a] += total_force / a_mass;
        }
        if !body.points[spring.b].pinned {
            forces[spring.b] -= total_force / b_mass;
        }
    }

    // Apply forces
    for (i, point) in body.points.iter_mut().enumerate() {
        if !point.pinned {
            point.velocity += forces[i] * dt;
        }
    }

    // 3. Global damping
    for point in &mut body.points {
        point.velocity *= 1.0 - (DAMPING * dt);
    }

    // 4. Velocity cap (prevent explosion)
    for point in &mut body.points {
        let speed = point.velocity.length();
        if speed > MAX_VELOCITY {
            point.velocity = point.velocity.normalize() * MAX_VELOCITY;
        }
    }

    // 5. Integration (position += velocity * dt)
    for point in &mut body.points {
        if !point.pinned {
            point.position += point.velocity * dt;
        }
    }

    // 6. DISTANCE CONSTRAINTS — prevent parts from disconnecting
    // This is like bones: they have a maximum length and can't stretch beyond it.
    // Run multiple iterations for stability (like a real constraint solver).
    for _iteration in 0..5 {
        for spring_idx in 0..body.springs.len() {
            let a_idx = body.springs[spring_idx].a;
            let b_idx = body.springs[spring_idx].b;
            let rest = body.springs[spring_idx].rest_length;
            let max_dist = rest * MAX_STRETCH_RATIO;

            let a_pos = body.points[a_idx].position;
            let b_pos = body.points[b_idx].position;
            let a_pinned = body.points[a_idx].pinned;
            let b_pinned = body.points[b_idx].pinned;

            let delta = b_pos - a_pos;
            let dist = delta.length();

            if dist > max_dist && dist > 0.001 {
                let correction = delta.normalize() * (dist - max_dist);

                if a_pinned && !b_pinned {
                    body.points[b_idx].position -= correction;
                    body.points[b_idx].velocity *= 0.5; // dampen on constraint hit
                } else if b_pinned && !a_pinned {
                    body.points[a_idx].position += correction;
                    body.points[a_idx].velocity *= 0.5;
                } else if !a_pinned && !b_pinned {
                    let half = correction * 0.5;
                    body.points[a_idx].position += half;
                    body.points[b_idx].position -= half;
                    body.points[a_idx].velocity *= 0.7;
                    body.points[b_idx].velocity *= 0.7;
                }
            }
        }
    }

    // 7. Ground collision
    for point in &mut body.points {
        if point.position.y > GROUND_Y {
            point.position.y = GROUND_Y;
            if point.velocity.y > 0.0 {
                point.velocity.y = -point.velocity.y * 0.2; // small bounce
            }
            point.grounded = true;
            // Friction
            point.velocity.x *= GROUND_FRICTION;
        } else {
            point.grounded = false;
        }
    }

    // 7. Canvas bounds (keep inside 64×64)
    for point in &mut body.points {
        point.position.x = point.position.x.clamp(2.0, 62.0);
        point.position.y = point.position.y.clamp(2.0, 62.0);
    }

    // 9. Rest position pull — gently pulls points back toward their natural position.
    // This prevents drift and keeps the creature's shape recognizable.
    // Stronger = more rigid (robotic). Weaker = more organic but risks deformation.
    for point in &mut body.points {
        if !point.pinned {
            let to_rest = point.rest - point.position;
            point.velocity += to_rest * 2.0 * dt; // moderate centering force
        }
    }
}

// ===================================================================
// SPECIES INITIALIZATION
// ===================================================================

/// Creates a Moluun adult soft body.
pub fn moluun_adult() -> SoftBody {
    let cx = 32.0; // canvas center X

    let points = vec![
        SoftPoint::new("head",       cx, 14.0, 7.5),
        SoftPoint::new("ear_anchor", cx, 6.0,  1.5),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 30.0, 20.0) }, // body is the anchor
        SoftPoint::new("belly",      cx, 33.0, 5.0),
        SoftPoint::new("shoulder_l", cx - 17.0, 26.0, 2.5),
        SoftPoint::new("paw_l",      cx - 17.0, 36.0, 2.5),
        SoftPoint::new("shoulder_r", cx + 17.0, 26.0, 2.5),
        SoftPoint::new("paw_r",      cx + 17.0, 36.0, 2.5),
        SoftPoint::new("foot_l",     cx - 8.0,  46.0, 3.0),
        SoftPoint::new("foot_r",     cx + 8.0,  46.0, 3.0),
    ];

    let springs = vec![
        // Neck (head to body) — strong enough to hold head up, flexible enough for dips
        Spring { a: 0, b: 2, rest_length: 16.0, stiffness: 300.0, damping: 20.0 },
        // Ear link (floppy but connected)
        Spring { a: 1, b: 0, rest_length: 8.0, stiffness: 150.0, damping: 10.0 },
        // Spine to belly (rigid core — breathing target)
        Spring { a: 2, b: 3, rest_length: 3.0, stiffness: 500.0, damping: 25.0 },
        // Left arm
        Spring { a: 4, b: 2, rest_length: 13.0, stiffness: 250.0, damping: 15.0 },
        Spring { a: 5, b: 4, rest_length: 10.0, stiffness: 200.0, damping: 12.0 },
        // Right arm
        Spring { a: 6, b: 2, rest_length: 13.0, stiffness: 250.0, damping: 15.0 },
        Spring { a: 7, b: 6, rest_length: 10.0, stiffness: 200.0, damping: 12.0 },
        // Left leg
        Spring { a: 8, b: 2, rest_length: 18.0, stiffness: 300.0, damping: 18.0 },
        // Right leg
        Spring { a: 9, b: 2, rest_length: 18.0, stiffness: 300.0, damping: 18.0 },
        // Structural: foot spread
        Spring { a: 8, b: 9, rest_length: 16.0, stiffness: 200.0, damping: 12.0 },
        // Structural: shoulder spread
        Spring { a: 4, b: 6, rest_length: 34.0, stiffness: 250.0, damping: 15.0 },
    ];

    SoftBody::new(points, springs)
}

/// Creates a Moluun cub soft body (simpler — bigger head, no arms).
#[allow(dead_code)]
pub fn moluun_cub() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",  cx, 20.0, 10.0),    // head IS the creature
        SoftPoint::new("ear_anchor", cx, 8.0, 1.0),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 40.0, 8.0) },
        SoftPoint::new("belly", cx, 41.0, 3.0),
        SoftPoint::new("foot_l", cx - 5.0, 46.0, 2.0),
        SoftPoint::new("foot_r", cx + 5.0, 46.0, 2.0),
    ];

    let springs = vec![
        Spring { a: 0, b: 2, rest_length: 20.0, stiffness: 300.0, damping: 20.0 },
        Spring { a: 1, b: 0, rest_length: 12.0, stiffness: 150.0, damping: 10.0 },
        Spring { a: 2, b: 3, rest_length: 1.0, stiffness: 200.0, damping: 15.0 },
        Spring { a: 4, b: 2, rest_length: 10.0, stiffness: 80.0, damping: 6.0 },
        Spring { a: 5, b: 2, rest_length: 10.0, stiffness: 80.0, damping: 6.0 },
        Spring { a: 4, b: 5, rest_length: 10.0, stiffness: 40.0, damping: 4.0 },
    ];

    SoftBody::new(points, springs)
}
