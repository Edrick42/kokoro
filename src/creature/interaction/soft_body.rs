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
        (Species::Pylum, GrowthStage::Cub) => pylum_cub(),
        (Species::Pylum, _) => pylum_adult(),
        (Species::Skael, GrowthStage::Cub) => skael_cub(),
        (Species::Skael, _) => skael_adult(),
        (Species::Nyxal, GrowthStage::Cub) => nyxal_cub(),
        (Species::Nyxal, _) => nyxal_adult(),
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

// ===================================================================
// PYLUM (bird) — tall, wings dominate, long legs, tail feathers
// ===================================================================

/// Pylum adult: tall body, large wings, casque crown, powerful taloned legs.
pub fn pylum_adult() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",      cx, 10.0, 5.0),       // small head with casque
        SoftPoint::new("casque",    cx, 3.0, 1.0),        // crown on top
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 24.0, 18.0) },
        SoftPoint::new("belly",     cx, 27.0, 4.0),
        SoftPoint::new("wing_l",    cx - 14.0, 20.0, 3.0), // wing roots
        SoftPoint::new("wing_r",    cx + 14.0, 20.0, 3.0),
        SoftPoint::new("wingtip_l", cx - 20.0, 26.0, 1.5), // floppy wing tips
        SoftPoint::new("wingtip_r", cx + 20.0, 26.0, 1.5),
        SoftPoint::new("tail",      cx, 34.0, 2.0),       // tail feathers
        SoftPoint::new("foot_l",    cx - 5.0, 54.0, 3.5), // powerful legs, low
        SoftPoint::new("foot_r",    cx + 5.0, 54.0, 3.5),
    ];

    let springs = vec![
        // Neck — bird necks are flexible
        Spring { a: 0, b: 2, rest_length: 14.0, stiffness: 250.0, damping: 18.0 },
        // Casque on head — rigid crown
        Spring { a: 1, b: 0, rest_length: 7.0, stiffness: 400.0, damping: 25.0 },
        // Belly
        Spring { a: 2, b: 3, rest_length: 3.0, stiffness: 500.0, damping: 25.0 },
        // Wing roots (shoulders) — medium stiffness
        Spring { a: 4, b: 2, rest_length: 14.0, stiffness: 200.0, damping: 14.0 },
        Spring { a: 5, b: 2, rest_length: 14.0, stiffness: 200.0, damping: 14.0 },
        // Wing tips — FLOPPY (low stiffness, low damping = flappy!)
        Spring { a: 6, b: 4, rest_length: 8.0, stiffness: 80.0, damping: 6.0 },
        Spring { a: 7, b: 5, rest_length: 8.0, stiffness: 80.0, damping: 6.0 },
        // Tail feathers — moderate flop
        Spring { a: 8, b: 2, rest_length: 10.0, stiffness: 120.0, damping: 10.0 },
        // Legs — long and stiff (bird legs are rigid struts)
        Spring { a: 9, b: 2, rest_length: 30.0, stiffness: 350.0, damping: 20.0 },
        Spring { a: 10, b: 2, rest_length: 30.0, stiffness: 350.0, damping: 20.0 },
        // Structural: wing spread
        Spring { a: 4, b: 5, rest_length: 28.0, stiffness: 200.0, damping: 12.0 },
        // Structural: foot spread
        Spring { a: 9, b: 10, rest_length: 10.0, stiffness: 200.0, damping: 12.0 },
    ];

    SoftBody::new(points, springs)
}

/// Pylum cub: fluffy ball, short legs, no real wings.
pub fn pylum_cub() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",   cx, 22.0, 12.0),       // head IS the creature (fluffy ball)
        SoftPoint::new("casque", cx, 10.0, 0.5),         // tiny tuft
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 32.0, 8.0) },
        SoftPoint::new("belly",  cx, 33.0, 2.0),
        SoftPoint::new("foot_l", cx - 5.0, 42.0, 2.0),  // short stubby legs
        SoftPoint::new("foot_r", cx + 5.0, 42.0, 2.0),
    ];

    let springs = vec![
        Spring { a: 0, b: 2, rest_length: 10.0, stiffness: 300.0, damping: 20.0 },
        Spring { a: 1, b: 0, rest_length: 12.0, stiffness: 200.0, damping: 15.0 },
        Spring { a: 2, b: 3, rest_length: 1.0, stiffness: 200.0, damping: 15.0 },
        Spring { a: 4, b: 2, rest_length: 10.0, stiffness: 100.0, damping: 8.0 },
        Spring { a: 5, b: 2, rest_length: 10.0, stiffness: 100.0, damping: 8.0 },
        Spring { a: 4, b: 5, rest_length: 10.0, stiffness: 50.0, damping: 4.0 },
    ];

    SoftBody::new(points, springs)
}

// ===================================================================
// SKAEL (reptile) — horizontal, armored, long segmented tail
// ===================================================================

/// Skael adult: horizontal tank, thick legs, long tail with 3 segments.
pub fn skael_adult() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",    cx, 14.0, 8.0),        // armored head
        SoftPoint::new("horn_l",  cx - 7.0, 4.0, 0.8),   // left horn
        SoftPoint::new("horn_r",  cx + 7.0, 4.0, 0.8),   // right horn
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 28.0, 25.0) }, // HEAVY body
        SoftPoint::new("belly",   cx, 31.0, 5.0),
        SoftPoint::new("leg_l",   cx - 15.0, 38.0, 4.0), // thick front legs
        SoftPoint::new("leg_r",   cx + 15.0, 38.0, 4.0),
        SoftPoint::new("foot_l",  cx - 8.0, 46.0, 3.5),  // rear feet
        SoftPoint::new("foot_r",  cx + 8.0, 46.0, 3.5),
        SoftPoint::new("tail_1",  cx, 40.0, 4.0),        // tail base (thick)
        SoftPoint::new("tail_2",  cx, 47.0, 2.5),        // tail mid
        SoftPoint::new("tail_3",  cx, 53.0, 1.5),        // tail tip (light, whippy)
    ];

    let springs = vec![
        // Neck — thick and rigid (armored)
        Spring { a: 0, b: 3, rest_length: 14.0, stiffness: 350.0, damping: 22.0 },
        // Horns — rigid on head
        Spring { a: 1, b: 0, rest_length: 12.0, stiffness: 400.0, damping: 25.0 },
        Spring { a: 2, b: 0, rest_length: 12.0, stiffness: 400.0, damping: 25.0 },
        // Belly
        Spring { a: 3, b: 4, rest_length: 3.0, stiffness: 500.0, damping: 25.0 },
        // Front legs — thick, stable
        Spring { a: 5, b: 3, rest_length: 15.0, stiffness: 300.0, damping: 18.0 },
        Spring { a: 6, b: 3, rest_length: 15.0, stiffness: 300.0, damping: 18.0 },
        // Rear feet
        Spring { a: 7, b: 3, rest_length: 20.0, stiffness: 300.0, damping: 18.0 },
        Spring { a: 8, b: 3, rest_length: 20.0, stiffness: 300.0, damping: 18.0 },
        // TAIL CHAIN — each segment gets progressively floppier
        Spring { a: 9, b: 3, rest_length: 12.0, stiffness: 250.0, damping: 16.0 },   // base: stiff
        Spring { a: 10, b: 9, rest_length: 7.0, stiffness: 150.0, damping: 10.0 },   // mid: medium
        Spring { a: 11, b: 10, rest_length: 6.0, stiffness: 80.0, damping: 6.0 },    // tip: whippy!
        // Structural: leg spread
        Spring { a: 5, b: 6, rest_length: 30.0, stiffness: 200.0, damping: 12.0 },
        Spring { a: 7, b: 8, rest_length: 16.0, stiffness: 200.0, damping: 12.0 },
    ];

    SoftBody::new(points, springs)
}

/// Skael cub: upright, smooth, thin tail, no armor.
pub fn skael_cub() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",   cx, 16.0, 9.0),        // big head (baby)
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 32.0, 10.0) },
        SoftPoint::new("belly",  cx, 34.0, 2.0),
        SoftPoint::new("tail_1", cx, 42.0, 2.0),        // thin whip tail
        SoftPoint::new("tail_2", cx, 48.0, 1.0),
        SoftPoint::new("foot_l", cx - 4.0, 44.0, 2.0),
        SoftPoint::new("foot_r", cx + 4.0, 44.0, 2.0),
    ];

    let springs = vec![
        Spring { a: 0, b: 1, rest_length: 16.0, stiffness: 280.0, damping: 20.0 },
        Spring { a: 1, b: 2, rest_length: 2.0, stiffness: 200.0, damping: 15.0 },
        Spring { a: 3, b: 1, rest_length: 10.0, stiffness: 150.0, damping: 10.0 },
        Spring { a: 4, b: 3, rest_length: 6.0, stiffness: 80.0, damping: 6.0 },
        Spring { a: 5, b: 1, rest_length: 14.0, stiffness: 100.0, damping: 8.0 },
        Spring { a: 6, b: 1, rest_length: 14.0, stiffness: 100.0, damping: 8.0 },
        Spring { a: 5, b: 6, rest_length: 8.0, stiffness: 50.0, damping: 4.0 },
    ];

    SoftBody::new(points, springs)
}

// ===================================================================
// NYXAL (cephalopod) — mantle dome + flowing tentacles, no skeleton
// ===================================================================

/// Nyxal adult: small dome, 4 long tentacles, side fins, all very soft.
pub fn nyxal_adult() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("mantle_top", cx, 4.0, 2.0),      // top of dome
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 14.0, 12.0) }, // mantle center
        SoftPoint::new("belly",      cx, 17.0, 3.0),
        SoftPoint::new("fin_l",      cx - 12.0, 12.0, 1.0), // side fins
        SoftPoint::new("fin_r",      cx + 12.0, 12.0, 1.0),
        SoftPoint::new("tent_fl",    cx - 5.0, 22.0, 2.0),  // front-left tentacle root
        SoftPoint::new("tent_fr",    cx + 5.0, 22.0, 2.0),  // front-right
        SoftPoint::new("tent_bl",    cx - 14.0, 22.0, 2.0), // back-left (outer)
        SoftPoint::new("tent_br",    cx + 14.0, 22.0, 2.0), // back-right (outer)
        SoftPoint::new("tip_fl",     cx - 5.0, 48.0, 0.8),  // tentacle tips — very light
        SoftPoint::new("tip_fr",     cx + 5.0, 48.0, 0.8),
        SoftPoint::new("tip_bl",     cx - 14.0, 44.0, 0.8),
        SoftPoint::new("tip_br",     cx + 14.0, 44.0, 0.8),
    ];

    let springs = vec![
        // Mantle dome — relatively rigid (the "skull")
        Spring { a: 0, b: 1, rest_length: 10.0, stiffness: 300.0, damping: 20.0 },
        // Belly
        Spring { a: 1, b: 2, rest_length: 3.0, stiffness: 400.0, damping: 20.0 },
        // Side fins — floppy, decorative
        Spring { a: 3, b: 1, rest_length: 12.0, stiffness: 60.0, damping: 5.0 },
        Spring { a: 4, b: 1, rest_length: 12.0, stiffness: 60.0, damping: 5.0 },
        // Tentacle roots from body — medium flexibility
        Spring { a: 5, b: 1, rest_length: 10.0, stiffness: 180.0, damping: 12.0 },
        Spring { a: 6, b: 1, rest_length: 10.0, stiffness: 180.0, damping: 12.0 },
        Spring { a: 7, b: 1, rest_length: 16.0, stiffness: 150.0, damping: 10.0 },
        Spring { a: 8, b: 1, rest_length: 16.0, stiffness: 150.0, damping: 10.0 },
        // Tentacle tips — VERY soft (the flowing motion comes from here)
        Spring { a: 9, b: 5, rest_length: 26.0, stiffness: 40.0, damping: 3.0 },
        Spring { a: 10, b: 6, rest_length: 26.0, stiffness: 40.0, damping: 3.0 },
        Spring { a: 11, b: 7, rest_length: 22.0, stiffness: 40.0, damping: 3.0 },
        Spring { a: 12, b: 8, rest_length: 22.0, stiffness: 40.0, damping: 3.0 },
        // Structural: tentacle spread (prevents crossing)
        Spring { a: 5, b: 6, rest_length: 10.0, stiffness: 100.0, damping: 8.0 },
        Spring { a: 7, b: 8, rest_length: 28.0, stiffness: 80.0, damping: 6.0 },
    ];

    SoftBody::new(points, springs)
}

/// Nyxal cub: huge dome, tiny tentacle stubs, almost all mantle.
pub fn nyxal_cub() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("mantle_top", cx, 6.0, 3.0),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 18.0, 12.0) },
        SoftPoint::new("belly",      cx, 20.0, 2.0),
        SoftPoint::new("tent_fl",    cx - 4.0, 32.0, 1.5),  // tiny stubs
        SoftPoint::new("tent_fr",    cx + 4.0, 32.0, 1.5),
        SoftPoint::new("tent_bl",    cx - 10.0, 30.0, 1.5),
        SoftPoint::new("tent_br",    cx + 10.0, 30.0, 1.5),
    ];

    let springs = vec![
        Spring { a: 0, b: 1, rest_length: 12.0, stiffness: 300.0, damping: 20.0 },
        Spring { a: 1, b: 2, rest_length: 2.0, stiffness: 300.0, damping: 15.0 },
        Spring { a: 3, b: 1, rest_length: 16.0, stiffness: 100.0, damping: 8.0 },
        Spring { a: 4, b: 1, rest_length: 16.0, stiffness: 100.0, damping: 8.0 },
        Spring { a: 5, b: 1, rest_length: 16.0, stiffness: 80.0, damping: 6.0 },
        Spring { a: 6, b: 1, rest_length: 16.0, stiffness: 80.0, damping: 6.0 },
        Spring { a: 3, b: 4, rest_length: 8.0, stiffness: 50.0, damping: 4.0 },
    ];

    SoftBody::new(points, springs)
}

// ===================================================================
// MOLUUN (mammal) — continued
// ===================================================================

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
