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

/// A hierarchical bound: a child point is hard-clamped to a circle of radius
/// `max_drift` centered at `parent.position + rest_offset`. Applied AFTER
/// integration. Guarantees that no impulse can ever push the child outside
/// the geometric envelope, regardless of magnitude.
///
/// Optionally linked to an anatomy joint name — when a joint loses flexibility
/// or integrity, the effective drift is modulated (stiffer or more dangly).
#[derive(Debug, Clone)]
pub struct Bound {
    pub child: usize,
    pub parent: usize,
    pub rest_offset: Vec2,
    pub max_drift: f32,
    /// Anatomy joint that gates this bound's drift (e.g. "neck", "shoulder_left").
    /// None = always use base `max_drift`.
    pub joint_name: Option<String>,
}

/// A polygon cluster — a group of points whose mutual shape (distances AND
/// angles) is preserved via shape matching. Each frame, the optimal rigid
/// transform (translation + rotation) that maps the rest polygon onto the
/// current points is computed; points are pulled toward their target
/// positions under that transform. `stiffness` ∈ [0, 1] controls how strongly
/// (1 = perfectly rigid, 0 = no shape preservation).
///
/// Use for face triangles (eyes + mouth) and torso quads where the silhouette
/// must never deform.
#[derive(Debug, Clone)]
pub struct Cluster {
    pub indices: Vec<usize>,
    /// Rest positions in cluster-local coords (centered at the rest centroid).
    pub rest_local: Vec<Vec2>,
    pub stiffness: f32,
}

/// Complete soft body for one creature.
#[derive(Resource, Debug, Clone)]
#[allow(dead_code)]
pub struct SoftBody {
    pub points: Vec<SoftPoint>,
    pub springs: Vec<Spring>,
    pub bounds: Vec<Bound>,
    pub clusters: Vec<Cluster>,
    /// Name → index lookup.
    index: HashMap<String, usize>,
}

impl SoftBody {
    pub fn new(points: Vec<SoftPoint>, springs: Vec<Spring>) -> Self {
        let index = points.iter().enumerate()
            .map(|(i, p)| (p.name.clone(), i))
            .collect();
        Self { points, springs, bounds: Vec::new(), clusters: Vec::new(), index }
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

    /// Apply an impulse (velocity change) to a named point.
    pub fn impulse(&mut self, name: &str, force: Vec2) {
        if let Some(&idx) = self.index.get(name) {
            self.points[idx].velocity += force;
        }
    }

    /// Add a hierarchical bound: clamp `child` to a circle of `max_drift`
    /// around `parent.position + rest_offset`.
    pub fn add_bound(&mut self, child: &str, parent: &str, rest_offset: Vec2, max_drift: f32) {
        if let (Some(&c), Some(&p)) = (self.index.get(child), self.index.get(parent)) {
            self.bounds.push(Bound { child: c, parent: p, rest_offset, max_drift, joint_name: None });
        }
    }

    /// Same as `add_bound` but links the bound to an anatomy joint — when the
    /// joint stiffens (low flexibility) or breaks (low integrity), the drift
    /// envelope is modulated accordingly.
    pub fn add_bound_jointed(&mut self, child: &str, parent: &str, rest_offset: Vec2, max_drift: f32, joint: &str) {
        if let (Some(&c), Some(&p)) = (self.index.get(child), self.index.get(parent)) {
            self.bounds.push(Bound {
                child: c, parent: p, rest_offset, max_drift,
                joint_name: Some(joint.to_string()),
            });
        }
    }

    /// Add a polygon cluster — preserves the shape (lengths AND angles) of
    /// the named points. Rest shape is captured from current positions.
    pub fn add_cluster(&mut self, names: &[&str], stiffness: f32) {
        let indices: Vec<usize> = names.iter()
            .filter_map(|n| self.index.get(*n).copied())
            .collect();
        if indices.len() < 2 { return; }

        // Compute rest centroid and rest_local positions
        let mut centroid = Vec2::ZERO;
        for &i in &indices { centroid += self.points[i].position; }
        centroid /= indices.len() as f32;
        let rest_local: Vec<Vec2> = indices.iter()
            .map(|&i| self.points[i].position - centroid)
            .collect();

        self.clusters.push(Cluster { indices, rest_local, stiffness });
    }
}

// ===================================================================
// PHYSICS CONSTANTS
// ===================================================================

/// Gravity — VERY gentle for a 64px canvas. Just enough to settle points downward.
const GRAVITY: f32 = 12.0;

/// Global damping — high to prevent oscillation (creature should be stable, not jelly).
const DAMPING: f32 = 10.0;

/// Ground friction (0.0 = ice, 1.0 = instant stop).
const GROUND_FRICTION: f32 = 0.9;

/// Ground Y position in the 64×64 canvas.
const GROUND_Y: f32 = 52.0;

/// Maximum velocity — low to prevent parts from flying off.
const MAX_VELOCITY: f32 = 22.0;

/// Maximum stretch ratio — springs CANNOT stretch beyond this × rest_length.
/// This prevents disconnection. Like a bone — it has a maximum length.
const MAX_STRETCH_RATIO: f32 = 1.12;

/// Strength of the per-point rest-position centering pull.
/// Higher = creature snaps back to its silhouette quickly (less wobble drift).
/// Lower = more organic float, but parts can drift away.
const REST_PULL: f32 = 7.0;

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
        (Species::Moluun, GrowthStage::Cub)   => moluun_cub(),
        (Species::Moluun, GrowthStage::Young) => moluun_young(),
        (Species::Moluun, GrowthStage::Elder) => moluun_elder(),
        (Species::Moluun, _)                  => moluun_adult(),
        (Species::Pylum,  GrowthStage::Cub)   => pylum_cub(),
        (Species::Pylum,  GrowthStage::Young) => pylum_young(),
        (Species::Pylum,  GrowthStage::Elder) => pylum_elder(),
        (Species::Pylum,  _)                  => pylum_adult(),
        (Species::Skael,  GrowthStage::Cub)   => skael_cub(),
        (Species::Skael,  GrowthStage::Young) => skael_young(),
        (Species::Skael,  GrowthStage::Elder) => skael_elder(),
        (Species::Skael,  _)                  => skael_adult(),
        (Species::Nyxal,  GrowthStage::Cub)   => nyxal_cub(),
        (Species::Nyxal,  GrowthStage::Young) => nyxal_young(),
        (Species::Nyxal,  GrowthStage::Elder) => nyxal_elder(),
        (Species::Nyxal,  _)                  => nyxal_adult(),
    };
    commands.insert_resource(new_body);
}

// ===================================================================
// PHYSICS STEP — runs every frame
// ===================================================================

fn soft_body_step(
    time: Res<Time>,
    mut body: Option<ResMut<SoftBody>>,
    anatomy: Option<Res<crate::creature::anatomy::AnatomyState>>,
) {
    let Some(ref mut body) = body else { return };
    let dt = time.delta_secs().min(0.033); // cap at ~30fps minimum
    let anat = anatomy.as_deref();

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

    // 7a. Polygon clusters — preserve mutual shape (lengths AND angles).
    // For each cluster, fit the optimal rigid transform from rest_local to
    // current positions, then pull each point toward its target under that
    // transform. This guarantees the polygon never deforms (only translates+rotates).
    apply_clusters(body);

    // 7b. Hierarchical bounds — hard clamp each child to a circle around its
    // ideal position relative to its parent. NO impulse can ever push a child
    // outside this circle (geometric envelope guarantee).
    apply_bounds(body, anat);

    // 8. Ground collision
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

    // 9. Canvas bounds (keep inside 64×64)
    for point in &mut body.points {
        point.position.x = point.position.x.clamp(2.0, 62.0);
        point.position.y = point.position.y.clamp(2.0, 62.0);
    }

    // 10. Rest position pull — pulls points back toward their natural position.
    // This prevents drift and keeps the creature's silhouette recognizable.
    // Stronger = more rigid (robotic). Weaker = more organic but risks deformation.
    for point in &mut body.points {
        if !point.pinned {
            let to_rest = point.rest - point.position;
            point.velocity += to_rest * REST_PULL * dt;
        }
    }
}

// ===================================================================
// SHAPE-MATCHING (polygon clusters)
// ===================================================================

/// For each cluster, find the optimal rigid 2D transform (translation + rotation)
/// that maps the rest polygon onto the current point positions, then pull each
/// point toward its transformed rest position by the cluster's stiffness.
fn apply_clusters(body: &mut SoftBody) {
    for cluster in &body.clusters {
        if cluster.indices.len() < 2 { continue; }

        // 1. Current centroid
        let mut current_com = Vec2::ZERO;
        for &i in &cluster.indices {
            current_com += body.points[i].position;
        }
        current_com /= cluster.indices.len() as f32;

        // 2. Build 2×2 cross-covariance matrix A = sum (current_local) * (rest_local)^T
        //    Procrustes problem in 2D: optimal rotation θ = atan2(A.y.x - A.x.y, A.x.x + A.y.y)
        let mut a00 = 0.0_f32;
        let mut a01 = 0.0_f32;
        let mut a10 = 0.0_f32;
        let mut a11 = 0.0_f32;
        for (k, &i) in cluster.indices.iter().enumerate() {
            let p = body.points[i].position - current_com;
            let q = cluster.rest_local[k];
            a00 += p.x * q.x;
            a01 += p.x * q.y;
            a10 += p.y * q.x;
            a11 += p.y * q.y;
        }
        let theta = (a10 - a01).atan2(a00 + a11);
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        // 3. For each point, compute its target = current_com + R * rest_local;
        //    pull current toward target by stiffness (lerp on position).
        let alpha = cluster.stiffness.clamp(0.0, 1.0);
        for (k, &i) in cluster.indices.iter().enumerate() {
            if body.points[i].pinned { continue; }
            let q = cluster.rest_local[k];
            let target = current_com + Vec2::new(
                cos_t * q.x - sin_t * q.y,
                sin_t * q.x + cos_t * q.y,
            );
            let pos = body.points[i].position;
            body.points[i].position = pos.lerp(target, alpha);
        }
    }
}

// ===================================================================
// HIERARCHICAL BOUNDS (hard clamp)
// ===================================================================

/// Clamp each child point to a circle of `max_drift` around its ideal
/// position (parent.position + rest_offset). Hard guarantee — no impulse
/// can ever push a child outside this envelope.
///
/// If a bound is linked to an anatomy joint, the drift is modulated:
/// - low flexibility (stiff/elder) → smaller drift (less mobile)
/// - low integrity (broken bone) → larger drift (dangling part)
fn apply_bounds(body: &mut SoftBody, anatomy: Option<&crate::creature::anatomy::AnatomyState>) {
    // Iterate twice for stability when bounds chain (child of a child)
    for _ in 0..2 {
        for bound in body.bounds.clone().iter() {
            if body.points[bound.child].pinned { continue; }

            // Compute effective drift from anatomy joint, if linked
            let drift = if let (Some(anat), Some(joint_name)) = (anatomy, &bound.joint_name) {
                let joint = anat.joints.joints.iter().find(|j| j.name == *joint_name);
                if let Some(j) = joint {
                    // flex_factor 0.4..1.0 (low flex → tight drift)
                    let flex_factor = 0.4 + 0.6 * j.flexibility;
                    // integrity_factor 1.0 normal, up to 2.0 when integrity < 0.5 (dangly)
                    let integrity_factor = if j.integrity > 0.5 {
                        1.0
                    } else {
                        1.0 + (0.5 - j.integrity).max(0.0) * 2.0
                    };
                    bound.max_drift * flex_factor * integrity_factor
                } else {
                    bound.max_drift
                }
            } else {
                bound.max_drift
            };

            let parent_pos = body.points[bound.parent].position;
            let ideal = parent_pos + bound.rest_offset;
            let pos = body.points[bound.child].position;
            let delta = pos - ideal;
            let dist = delta.length();
            if dist > drift && dist > 0.001 {
                let clamped = ideal + delta.normalize() * drift;
                body.points[bound.child].position = clamped;
                // Kill outward velocity component (so the point doesn't keep pushing)
                let vel = body.points[bound.child].velocity;
                let outward = delta.normalize();
                let outward_vel = vel.dot(outward);
                if outward_vel > 0.0 {
                    body.points[bound.child].velocity -= outward * outward_vel * 0.8;
                }
            }
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
        SoftPoint::new("head",       cx, 12.0, 7.5),                        // moved up 2px for visible neck
        SoftPoint::new("ear_anchor", cx, 4.0,  1.5),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 30.0, 20.0) }, // anchor
        SoftPoint::new("belly",      cx, 33.0, 5.0),
        SoftPoint::new("shoulder_l", cx - 17.0, 26.0, 2.5),
        SoftPoint::new("paw_l",      cx - 17.0, 36.0, 2.5),
        SoftPoint::new("shoulder_r", cx + 17.0, 26.0, 2.5),
        SoftPoint::new("paw_r",      cx + 17.0, 36.0, 2.5),
        SoftPoint::new("foot_l",     cx - 8.0,  46.0, 3.0),
        SoftPoint::new("foot_r",     cx + 8.0,  46.0, 3.0),
        // Face features — pinned to head via bounds
        SoftPoint::new("eye_l",      cx - 4.0,  13.0, 0.3),
        SoftPoint::new("eye_r",      cx + 4.0,  13.0, 0.3),
        SoftPoint::new("mouth",      cx,        25.0, 0.3),
    ];

    let springs = vec![
        // Neck (head to body)
        Spring { a: 0, b: 2, rest_length: 16.0, stiffness: 300.0, damping: 20.0 },
        Spring { a: 1, b: 0, rest_length: 8.0,  stiffness: 150.0, damping: 10.0 },
        Spring { a: 2, b: 3, rest_length: 3.0,  stiffness: 500.0, damping: 25.0 },
        Spring { a: 4, b: 2, rest_length: 13.0, stiffness: 250.0, damping: 15.0 },
        Spring { a: 5, b: 4, rest_length: 10.0, stiffness: 200.0, damping: 12.0 },
        Spring { a: 6, b: 2, rest_length: 13.0, stiffness: 250.0, damping: 15.0 },
        Spring { a: 7, b: 6, rest_length: 10.0, stiffness: 200.0, damping: 12.0 },
        Spring { a: 8, b: 2, rest_length: 18.0, stiffness: 300.0, damping: 18.0 },
        Spring { a: 9, b: 2, rest_length: 18.0, stiffness: 300.0, damping: 18.0 },
        Spring { a: 8, b: 9, rest_length: 16.0, stiffness: 200.0, damping: 12.0 },
        Spring { a: 4, b: 6, rest_length: 34.0, stiffness: 250.0, damping: 15.0 },
    ];

    let mut body = SoftBody::new(points, springs);

    // Hierarchical bounds — geometric envelope guarantee.
    // Joint-linked bounds modulate drift based on anatomy state (stiffness/damage).
    body.add_bound_jointed("head",       "body", Vec2::new(0.0, -18.0), 2.0, "neck");
    body.add_bound("ear_anchor", "head", Vec2::new(0.0, -8.0), 2.5);
    body.add_bound("belly",      "body", Vec2::new(0.0,  3.0), 1.5);
    body.add_bound_jointed("shoulder_l", "body", Vec2::new(-13.0, -4.0), 2.0, "shoulder_left");
    body.add_bound_jointed("shoulder_r", "body", Vec2::new( 13.0, -4.0), 2.0, "shoulder_right");
    body.add_bound("paw_l", "shoulder_l", Vec2::new(0.0, 10.0), 3.0);
    body.add_bound("paw_r", "shoulder_r", Vec2::new(0.0, 10.0), 3.0);
    body.add_bound_jointed("foot_l", "body", Vec2::new(-8.0, 16.0), 2.0, "hip_left");
    body.add_bound_jointed("foot_r", "body", Vec2::new( 8.0, 16.0), 2.0, "hip_right");
    // Face features — tightly locked to head (1.2px tolerance for micro-expressions)
    body.add_bound("eye_l", "head", Vec2::new(-4.0, 1.0), 1.2);
    body.add_bound("eye_r", "head", Vec2::new( 4.0, 1.0), 1.2);
    body.add_bound("mouth", "head", Vec2::new( 0.0, 13.0), 1.2);

    // === ANATOMICAL TRIANGULATED MESH ===
    // The body silhouette is decomposed into a mesh of triangles. Each triangle
    // is a rigid cluster that preserves its mutual shape (lengths AND angles)
    // via shape matching. Shared vertices stitch triangles together so the
    // whole creature is a single connected topology — like a face wireframe.

    // FACE (head region) — triangulated forehead, eye bridge, cheeks
    body.add_cluster(&["ear_anchor", "eye_l", "eye_r"], 0.6);   // forehead
    body.add_cluster(&["eye_l", "head", "eye_r"], 0.6);         // eye bridge / nose
    body.add_cluster(&["eye_l", "head", "mouth"], 0.55);        // left cheek
    body.add_cluster(&["eye_r", "head", "mouth"], 0.55);        // right cheek

    // NECK BRIDGE — connects head region to torso (mouth + shoulders)
    body.add_cluster(&["mouth", "shoulder_l", "shoulder_r"], 0.5);

    // TORSO — shoulder girdle + spine + belly
    body.add_cluster(&["shoulder_l", "body", "shoulder_r"], 0.6); // chest cap
    body.add_cluster(&["shoulder_l", "body", "belly"], 0.5);       // left side
    body.add_cluster(&["shoulder_r", "body", "belly"], 0.5);       // right side

    // ARMS — each arm is a triangle (shoulder→paw with body as anchor)
    body.add_cluster(&["shoulder_l", "paw_l", "body"], 0.45);
    body.add_cluster(&["shoulder_r", "paw_r", "body"], 0.45);

    // LEGS — feet triangulated with belly + body
    body.add_cluster(&["body", "foot_l", "foot_r"], 0.55);
    body.add_cluster(&["belly", "foot_l", "body"], 0.45);
    body.add_cluster(&["belly", "foot_r", "body"], 0.45);

    body
}

// ===================================================================
// PYLUM (bird) — tall, wings dominate, long legs, tail feathers
// ===================================================================

/// Pylum adult: tall body, large wings, casque crown, powerful taloned legs.
pub fn pylum_adult() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",      cx, 10.0, 5.0),
        SoftPoint::new("casque",    cx, 3.0,  1.0),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 24.0, 18.0) },
        SoftPoint::new("belly",     cx, 27.0, 4.0),
        SoftPoint::new("wing_l",    cx - 14.0, 20.0, 3.0),
        SoftPoint::new("wing_r",    cx + 14.0, 20.0, 3.0),
        SoftPoint::new("wingtip_l", cx - 20.0, 26.0, 1.5),
        SoftPoint::new("wingtip_r", cx + 20.0, 26.0, 1.5),
        SoftPoint::new("tail",      cx, 34.0, 2.0),
        SoftPoint::new("foot_l",    cx - 5.0, 54.0, 3.5),
        SoftPoint::new("foot_r",    cx + 5.0, 54.0, 3.5),
        // Face — beak area
        SoftPoint::new("eye_l",     cx - 3.0, 11.0, 0.3),
        SoftPoint::new("eye_r",     cx + 3.0, 11.0, 0.3),
        SoftPoint::new("mouth",     cx,       23.0, 0.3), // beak
    ];

    let springs = vec![
        Spring { a: 0, b: 2, rest_length: 14.0, stiffness: 250.0, damping: 18.0 },
        Spring { a: 1, b: 0, rest_length: 7.0,  stiffness: 400.0, damping: 25.0 },
        Spring { a: 2, b: 3, rest_length: 3.0,  stiffness: 500.0, damping: 25.0 },
        Spring { a: 4, b: 2, rest_length: 14.0, stiffness: 200.0, damping: 14.0 },
        Spring { a: 5, b: 2, rest_length: 14.0, stiffness: 200.0, damping: 14.0 },
        Spring { a: 6, b: 4, rest_length: 8.0,  stiffness: 80.0,  damping: 6.0 },
        Spring { a: 7, b: 5, rest_length: 8.0,  stiffness: 80.0,  damping: 6.0 },
        Spring { a: 8, b: 2, rest_length: 10.0, stiffness: 120.0, damping: 10.0 },
        Spring { a: 9, b: 2, rest_length: 30.0, stiffness: 350.0, damping: 20.0 },
        Spring { a: 10,b: 2, rest_length: 30.0, stiffness: 350.0, damping: 20.0 },
        Spring { a: 4, b: 5, rest_length: 28.0, stiffness: 200.0, damping: 12.0 },
        Spring { a: 9, b: 10,rest_length: 10.0, stiffness: 200.0, damping: 12.0 },
    ];

    let mut body = SoftBody::new(points, springs);
    body.add_bound_jointed("head", "body", Vec2::new(0.0, -14.0), 2.5, "neck");
    body.add_bound("casque", "head", Vec2::new(0.0, -7.0),  2.0);
    body.add_bound("belly",  "body", Vec2::new(0.0,  3.0),  1.5);
    body.add_bound_jointed("wing_l", "body", Vec2::new(-14.0, -4.0), 3.0, "wing_left");
    body.add_bound_jointed("wing_r", "body", Vec2::new( 14.0, -4.0), 3.0, "wing_right");
    body.add_bound("wingtip_l", "wing_l", Vec2::new(-6.0, 6.0), 4.0);
    body.add_bound("wingtip_r", "wing_r", Vec2::new( 6.0, 6.0), 4.0);
    body.add_bound_jointed("tail", "body", Vec2::new(0.0, 10.0), 3.0, "tail");
    body.add_bound("foot_l", "body", Vec2::new(-5.0, 30.0), 2.0);
    body.add_bound("foot_r", "body", Vec2::new( 5.0, 30.0), 2.0);
    body.add_bound("eye_l",  "head", Vec2::new(-3.0, 1.0), 1.0);
    body.add_bound("eye_r",  "head", Vec2::new( 3.0, 1.0), 1.0);
    body.add_bound("mouth",  "head", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Pylum adult) ===
    // Face
    body.add_cluster(&["casque", "eye_l", "eye_r"], 0.6);
    body.add_cluster(&["eye_l", "head", "eye_r"], 0.6);
    body.add_cluster(&["eye_l", "head", "mouth"], 0.55);
    body.add_cluster(&["eye_r", "head", "mouth"], 0.55);
    // Neck bridge
    body.add_cluster(&["mouth", "wing_l", "wing_r"], 0.5);
    // Torso
    body.add_cluster(&["wing_l", "body", "wing_r"], 0.55);
    body.add_cluster(&["wing_l", "body", "belly"], 0.5);
    body.add_cluster(&["wing_r", "body", "belly"], 0.5);
    // Wings (triangles, not lines — wingtip floats relative to wing root + body)
    body.add_cluster(&["wing_l", "wingtip_l", "body"], 0.4);
    body.add_cluster(&["wing_r", "wingtip_r", "body"], 0.4);
    // Lower body + tail + legs
    body.add_cluster(&["body", "tail", "belly"], 0.5);
    body.add_cluster(&["body", "foot_l", "foot_r"], 0.55);
    body.add_cluster(&["body", "foot_l", "tail"], 0.45);
    body.add_cluster(&["body", "foot_r", "tail"], 0.45);
    body
}

/// Pylum cub: fluffy sphere, head dominates, tiny legs tucked under.
pub fn pylum_cub() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",   cx, 24.0, 16.0),
        SoftPoint::new("casque", cx, 11.0, 0.4),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 36.0, 5.0) },
        SoftPoint::new("belly",  cx, 37.0, 1.5),
        SoftPoint::new("foot_l", cx - 3.0, 44.0, 1.2),
        SoftPoint::new("foot_r", cx + 3.0, 44.0, 1.2),
        SoftPoint::new("eye_l",  cx - 4.0, 22.0, 0.3),
        SoftPoint::new("eye_r",  cx + 4.0, 22.0, 0.3),
        SoftPoint::new("mouth",  cx,       37.0, 0.3),
    ];

    let springs = vec![
        Spring { a: 0, b: 2, rest_length: 10.0, stiffness: 240.0, damping: 16.0 },
        Spring { a: 1, b: 0, rest_length: 13.0, stiffness: 150.0, damping: 10.0 },
        Spring { a: 2, b: 3, rest_length: 1.0,  stiffness: 200.0, damping: 15.0 },
        Spring { a: 4, b: 2, rest_length: 8.0,  stiffness: 90.0,  damping: 7.0  },
        Spring { a: 5, b: 2, rest_length: 8.0,  stiffness: 90.0,  damping: 7.0  },
        Spring { a: 4, b: 5, rest_length: 6.0,  stiffness: 50.0,  damping: 4.0  },
    ];

    let mut body = SoftBody::new(points, springs);
    body.add_bound("head",   "body", Vec2::new(0.0, -12.0), 2.0);
    body.add_bound("casque", "head", Vec2::new(0.0, -13.0), 2.5);
    body.add_bound("belly",  "body", Vec2::new(0.0,  1.0),  1.0);
    body.add_bound("foot_l", "body", Vec2::new(-3.0, 8.0),  1.5);
    body.add_bound("foot_r", "body", Vec2::new( 3.0, 8.0),  1.5);
    body.add_bound("eye_l",  "head", Vec2::new(-4.0, -2.0), 1.0);
    body.add_bound("eye_r",  "head", Vec2::new( 4.0, -2.0), 1.0);
    body.add_bound("mouth",  "head", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Pylum cub) ===
    // Face — head dominates, casque + eyes + mouth + head as anchors
    body.add_cluster(&["casque", "eye_l", "eye_r"], 0.7);
    body.add_cluster(&["eye_l", "head", "eye_r"], 0.7);
    body.add_cluster(&["eye_l", "head", "mouth"], 0.65);
    body.add_cluster(&["eye_r", "head", "mouth"], 0.65);
    // Neck bridge
    body.add_cluster(&["mouth", "body", "belly"], 0.5);
    // Body + feet
    body.add_cluster(&["body", "foot_l", "foot_r"], 0.5);
    body.add_cluster(&["belly", "foot_l", "foot_r"], 0.5);
    body
}

// ===================================================================
// SKAEL (reptile) — horizontal, armored, long segmented tail
// ===================================================================

/// Skael adult: horizontal tank, thick legs, long tail with 3 segments.
pub fn skael_adult() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",    cx, 14.0, 8.0),
        SoftPoint::new("horn_l",  cx - 7.0, 4.0, 0.8),
        SoftPoint::new("horn_r",  cx + 7.0, 4.0, 0.8),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 28.0, 25.0) },
        SoftPoint::new("belly",   cx, 31.0, 5.0),
        SoftPoint::new("leg_l",   cx - 15.0, 38.0, 4.0),
        SoftPoint::new("leg_r",   cx + 15.0, 38.0, 4.0),
        SoftPoint::new("foot_l",  cx - 8.0, 46.0, 3.5),
        SoftPoint::new("foot_r",  cx + 8.0, 46.0, 3.5),
        SoftPoint::new("tail_1",  cx, 40.0, 4.0),
        SoftPoint::new("tail_2",  cx, 47.0, 2.5),
        SoftPoint::new("tail_3",  cx, 53.0, 1.5),
        // Face — snout area
        SoftPoint::new("eye_l",   cx - 3.0, 13.0, 0.3),
        SoftPoint::new("eye_r",   cx + 3.0, 13.0, 0.3),
        SoftPoint::new("mouth",   cx,       27.0, 0.3),
    ];

    let springs = vec![
        Spring { a: 0, b: 3, rest_length: 14.0, stiffness: 350.0, damping: 22.0 },
        Spring { a: 1, b: 0, rest_length: 12.0, stiffness: 400.0, damping: 25.0 },
        Spring { a: 2, b: 0, rest_length: 12.0, stiffness: 400.0, damping: 25.0 },
        Spring { a: 3, b: 4, rest_length: 3.0,  stiffness: 500.0, damping: 25.0 },
        Spring { a: 5, b: 3, rest_length: 15.0, stiffness: 300.0, damping: 18.0 },
        Spring { a: 6, b: 3, rest_length: 15.0, stiffness: 300.0, damping: 18.0 },
        Spring { a: 7, b: 3, rest_length: 20.0, stiffness: 300.0, damping: 18.0 },
        Spring { a: 8, b: 3, rest_length: 20.0, stiffness: 300.0, damping: 18.0 },
        Spring { a: 9, b: 3, rest_length: 12.0, stiffness: 250.0, damping: 16.0 },
        Spring { a: 10,b: 9, rest_length: 7.0,  stiffness: 150.0, damping: 10.0 },
        Spring { a: 11,b: 10,rest_length: 6.0,  stiffness: 80.0,  damping: 6.0  },
        Spring { a: 5, b: 6, rest_length: 30.0, stiffness: 200.0, damping: 12.0 },
        Spring { a: 7, b: 8, rest_length: 16.0, stiffness: 200.0, damping: 12.0 },
    ];

    let mut body = SoftBody::new(points, springs);
    body.add_bound_jointed("head", "body", Vec2::new(0.0, -14.0), 2.5, "neck");
    body.add_bound("horn_l", "head", Vec2::new(-7.0, -10.0), 2.0);
    body.add_bound("horn_r", "head", Vec2::new( 7.0, -10.0), 2.0);
    body.add_bound("belly",  "body", Vec2::new(0.0, 3.0), 1.5);
    body.add_bound_jointed("leg_l",  "body", Vec2::new(-15.0, 10.0), 2.5, "shoulder_left");
    body.add_bound_jointed("leg_r",  "body", Vec2::new( 15.0, 10.0), 2.5, "shoulder_right");
    body.add_bound_jointed("foot_l", "body", Vec2::new(-8.0, 18.0), 2.0, "hip_left");
    body.add_bound_jointed("foot_r", "body", Vec2::new( 8.0, 18.0), 2.0, "hip_right");
    body.add_bound_jointed("tail_1", "body", Vec2::new(0.0, 12.0), 2.5, "tail");
    body.add_bound("tail_2", "tail_1", Vec2::new(0.0, 7.0), 3.0);
    body.add_bound("tail_3", "tail_2", Vec2::new(0.0, 6.0), 4.0);
    body.add_bound("eye_l",  "head", Vec2::new(-3.0, -1.0), 1.0);
    body.add_bound("eye_r",  "head", Vec2::new( 3.0, -1.0), 1.0);
    body.add_bound("mouth",  "head", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Skael adult) ===
    // Face — armored skull with horns
    body.add_cluster(&["horn_l", "horn_r", "head"], 0.6);
    body.add_cluster(&["eye_l", "head", "eye_r"], 0.6);
    body.add_cluster(&["eye_l", "head", "mouth"], 0.55);
    body.add_cluster(&["eye_r", "head", "mouth"], 0.55);
    body.add_cluster(&["horn_l", "eye_l", "head"], 0.55);
    body.add_cluster(&["horn_r", "eye_r", "head"], 0.55);
    // Neck bridge
    body.add_cluster(&["mouth", "leg_l", "leg_r"], 0.5);
    // Torso (front legs frame the chest)
    body.add_cluster(&["leg_l", "body", "leg_r"], 0.6);
    body.add_cluster(&["leg_l", "body", "belly"], 0.5);
    body.add_cluster(&["leg_r", "body", "belly"], 0.5);
    // Hindquarters + feet
    body.add_cluster(&["body", "foot_l", "foot_r"], 0.55);
    body.add_cluster(&["body", "foot_l", "tail_1"], 0.5);
    body.add_cluster(&["body", "foot_r", "tail_1"], 0.5);
    // Tail chain — three segments preserved as triangles
    body.add_cluster(&["tail_1", "tail_2", "body"], 0.5);
    body.add_cluster(&["tail_1", "tail_2", "tail_3"], 0.45);
    body
}

/// Skael cub: big round head dominates, fat pudgy belly, stubby tail, tiny feet.
pub fn skael_cub() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",   cx, 18.0, 13.0),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 33.0, 8.0) },
        SoftPoint::new("belly",  cx, 36.0, 3.0),
        SoftPoint::new("tail_1", cx, 41.0, 1.5),
        SoftPoint::new("tail_2", cx, 46.0, 0.8),
        SoftPoint::new("foot_l", cx - 3.0, 44.0, 1.5),
        SoftPoint::new("foot_r", cx + 3.0, 44.0, 1.5),
        SoftPoint::new("eye_l",  cx - 4.0, 17.0, 0.3),
        SoftPoint::new("eye_r",  cx + 4.0, 17.0, 0.3),
        SoftPoint::new("mouth",  cx,       31.0, 0.3),
    ];

    let springs = vec![
        Spring { a: 0, b: 1, rest_length: 15.0, stiffness: 240.0, damping: 16.0 },
        Spring { a: 1, b: 2, rest_length: 3.0,  stiffness: 200.0, damping: 15.0 },
        Spring { a: 3, b: 1, rest_length: 8.0,  stiffness: 120.0, damping: 9.0  },
        Spring { a: 4, b: 3, rest_length: 5.0,  stiffness: 60.0,  damping: 5.0  },
        Spring { a: 5, b: 1, rest_length: 12.0, stiffness: 90.0,  damping: 7.0  },
        Spring { a: 6, b: 1, rest_length: 12.0, stiffness: 90.0,  damping: 7.0  },
        Spring { a: 5, b: 6, rest_length: 6.0,  stiffness: 50.0,  damping: 4.0  },
    ];

    let mut body = SoftBody::new(points, springs);
    body.add_bound("head",   "body", Vec2::new(0.0, -15.0), 2.0);
    body.add_bound("belly",  "body", Vec2::new(0.0,  3.0),  1.0);
    body.add_bound("tail_1", "body", Vec2::new(0.0,  8.0),  2.0);
    body.add_bound("tail_2", "tail_1", Vec2::new(0.0, 5.0), 3.0);
    body.add_bound("foot_l", "body", Vec2::new(-3.0, 11.0), 1.5);
    body.add_bound("foot_r", "body", Vec2::new( 3.0, 11.0), 1.5);
    body.add_bound("eye_l",  "head", Vec2::new(-4.0, -1.0), 1.0);
    body.add_bound("eye_r",  "head", Vec2::new( 4.0, -1.0), 1.0);
    body.add_bound("mouth",  "head", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Skael cub) ===
    // Face
    body.add_cluster(&["eye_l", "head", "eye_r"], 0.7);
    body.add_cluster(&["eye_l", "head", "mouth"], 0.65);
    body.add_cluster(&["eye_r", "head", "mouth"], 0.65);
    // Neck bridge
    body.add_cluster(&["mouth", "body", "belly"], 0.55);
    // Torso + feet
    body.add_cluster(&["body", "foot_l", "foot_r"], 0.55);
    body.add_cluster(&["belly", "foot_l", "body"], 0.5);
    body.add_cluster(&["belly", "foot_r", "body"], 0.5);
    // Tail
    body.add_cluster(&["body", "tail_1", "tail_2"], 0.45);
    body
}

// ===================================================================
// NYXAL (cephalopod) — mantle dome + flowing tentacles, no skeleton
// ===================================================================

/// Nyxal adult: small dome, 4 long tentacles, side fins, all very soft.
pub fn nyxal_adult() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("mantle_top", cx, 4.0, 2.0),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 14.0, 12.0) },
        SoftPoint::new("belly",      cx, 17.0, 3.0),
        SoftPoint::new("fin_l",      cx - 12.0, 12.0, 1.0),
        SoftPoint::new("fin_r",      cx + 12.0, 12.0, 1.0),
        SoftPoint::new("tent_fl",    cx - 5.0, 22.0, 2.0),
        SoftPoint::new("tent_fr",    cx + 5.0, 22.0, 2.0),
        SoftPoint::new("tent_bl",    cx - 14.0, 22.0, 2.0),
        SoftPoint::new("tent_br",    cx + 14.0, 22.0, 2.0),
        SoftPoint::new("tip_fl",     cx - 5.0, 48.0, 0.8),
        SoftPoint::new("tip_fr",     cx + 5.0, 48.0, 0.8),
        SoftPoint::new("tip_bl",     cx - 14.0, 44.0, 0.8),
        SoftPoint::new("tip_br",     cx + 14.0, 44.0, 0.8),
        // Face — on mantle (no separate "head" for cephalopod; eyes on body)
        SoftPoint::new("eye_l",      cx - 4.0, 14.0, 0.3),
        SoftPoint::new("eye_r",      cx + 4.0, 14.0, 0.3),
        SoftPoint::new("mouth",      cx,       27.0, 0.3),
    ];

    let springs = vec![
        Spring { a: 0, b: 1, rest_length: 10.0, stiffness: 300.0, damping: 20.0 },
        Spring { a: 1, b: 2, rest_length: 3.0,  stiffness: 400.0, damping: 20.0 },
        Spring { a: 3, b: 1, rest_length: 12.0, stiffness: 60.0,  damping: 5.0  },
        Spring { a: 4, b: 1, rest_length: 12.0, stiffness: 60.0,  damping: 5.0  },
        Spring { a: 5, b: 1, rest_length: 10.0, stiffness: 180.0, damping: 12.0 },
        Spring { a: 6, b: 1, rest_length: 10.0, stiffness: 180.0, damping: 12.0 },
        Spring { a: 7, b: 1, rest_length: 16.0, stiffness: 150.0, damping: 10.0 },
        Spring { a: 8, b: 1, rest_length: 16.0, stiffness: 150.0, damping: 10.0 },
        Spring { a: 9, b: 5, rest_length: 26.0, stiffness: 40.0,  damping: 3.0  },
        Spring { a: 10,b: 6, rest_length: 26.0, stiffness: 40.0,  damping: 3.0  },
        Spring { a: 11,b: 7, rest_length: 22.0, stiffness: 40.0,  damping: 3.0  },
        Spring { a: 12,b: 8, rest_length: 22.0, stiffness: 40.0,  damping: 3.0  },
        Spring { a: 5, b: 6, rest_length: 10.0, stiffness: 100.0, damping: 8.0  },
        Spring { a: 7, b: 8, rest_length: 28.0, stiffness: 80.0,  damping: 6.0  },
    ];

    let mut body = SoftBody::new(points, springs);
    body.add_bound_jointed("mantle_top", "body", Vec2::new(0.0, -10.0), 2.5, "mantle_base");
    body.add_bound("belly", "body", Vec2::new(0.0, 3.0), 1.5);
    body.add_bound("fin_l", "body", Vec2::new(-12.0, -2.0), 4.0);
    body.add_bound("fin_r", "body", Vec2::new( 12.0, -2.0), 4.0);
    body.add_bound_jointed("tent_fl", "body", Vec2::new(-5.0, 8.0), 2.5, "tentacle_front_left");
    body.add_bound_jointed("tent_fr", "body", Vec2::new( 5.0, 8.0), 2.5, "tentacle_front_right");
    body.add_bound_jointed("tent_bl", "body", Vec2::new(-14.0, 8.0), 2.5, "tentacle_back");
    body.add_bound_jointed("tent_br", "body", Vec2::new( 14.0, 8.0), 2.5, "tentacle_back");
    body.add_bound("tip_fl", "tent_fl", Vec2::new(0.0, 26.0), 6.0);
    body.add_bound("tip_fr", "tent_fr", Vec2::new(0.0, 26.0), 6.0);
    body.add_bound("tip_bl", "tent_bl", Vec2::new(0.0, 22.0), 6.0);
    body.add_bound("tip_br", "tent_br", Vec2::new(0.0, 22.0), 6.0);
    body.add_bound("eye_l", "body", Vec2::new(-4.0, 0.0), 1.0);
    body.add_bound("eye_r", "body", Vec2::new( 4.0, 0.0), 1.0);
    body.add_bound("mouth", "body", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Nyxal adult) ===
    // Mantle dome (top of head)
    body.add_cluster(&["mantle_top", "eye_l", "eye_r"], 0.65);
    body.add_cluster(&["mantle_top", "fin_l", "body"], 0.55);
    body.add_cluster(&["mantle_top", "fin_r", "body"], 0.55);
    // Face on body
    body.add_cluster(&["eye_l", "body", "eye_r"], 0.6);
    body.add_cluster(&["eye_l", "body", "mouth"], 0.55);
    body.add_cluster(&["eye_r", "body", "mouth"], 0.55);
    // Tentacle bridge
    body.add_cluster(&["mouth", "tent_fl", "tent_fr"], 0.5);
    // Tentacle roots framework
    body.add_cluster(&["tent_fl", "body", "tent_fr"], 0.5);
    body.add_cluster(&["tent_bl", "body", "tent_br"], 0.5);
    body.add_cluster(&["tent_fl", "body", "tent_bl"], 0.45);
    body.add_cluster(&["tent_fr", "body", "tent_br"], 0.45);
    // Tentacle pairs — each is a triangle (root, tip, body)
    body.add_cluster(&["tent_fl", "tip_fl", "body"], 0.35);
    body.add_cluster(&["tent_fr", "tip_fr", "body"], 0.35);
    body.add_cluster(&["tent_bl", "tip_bl", "body"], 0.35);
    body.add_cluster(&["tent_br", "tip_br", "body"], 0.35);
    body
}

/// Nyxal cub: balloon mantle dominates, tentacles are mere stubs pulled in close.
pub fn nyxal_cub() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("mantle_top", cx, 4.0, 3.5),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 16.0, 16.0) },
        SoftPoint::new("belly",      cx, 19.0, 2.0),
        SoftPoint::new("tent_fl",    cx - 3.0, 27.0, 1.0),
        SoftPoint::new("tent_fr",    cx + 3.0, 27.0, 1.0),
        SoftPoint::new("tent_bl",    cx - 7.0, 26.0, 1.0),
        SoftPoint::new("tent_br",    cx + 7.0, 26.0, 1.0),
        SoftPoint::new("eye_l",      cx - 5.0, 17.0, 0.3),
        SoftPoint::new("eye_r",      cx + 5.0, 17.0, 0.3),
        SoftPoint::new("mouth",      cx,       29.0, 0.3),
    ];

    let springs = vec![
        Spring { a: 0, b: 1, rest_length: 12.0, stiffness: 280.0, damping: 18.0 },
        Spring { a: 1, b: 2, rest_length: 3.0,  stiffness: 300.0, damping: 15.0 },
        Spring { a: 3, b: 1, rest_length: 11.0, stiffness: 90.0,  damping: 7.0  },
        Spring { a: 4, b: 1, rest_length: 11.0, stiffness: 90.0,  damping: 7.0  },
        Spring { a: 5, b: 1, rest_length: 12.0, stiffness: 70.0,  damping: 5.0  },
        Spring { a: 6, b: 1, rest_length: 12.0, stiffness: 70.0,  damping: 5.0  },
        Spring { a: 3, b: 4, rest_length: 6.0,  stiffness: 40.0,  damping: 3.0  },
    ];

    let mut body = SoftBody::new(points, springs);
    body.add_bound("mantle_top", "body", Vec2::new(0.0, -12.0), 2.0);
    body.add_bound("belly",      "body", Vec2::new(0.0,  3.0),  1.0);
    body.add_bound("tent_fl",    "body", Vec2::new(-3.0, 11.0), 2.0);
    body.add_bound("tent_fr",    "body", Vec2::new( 3.0, 11.0), 2.0);
    body.add_bound("tent_bl",    "body", Vec2::new(-7.0, 10.0), 2.0);
    body.add_bound("tent_br",    "body", Vec2::new( 7.0, 10.0), 2.0);
    body.add_bound("eye_l",      "body", Vec2::new(-5.0, 1.0),  1.0);
    body.add_bound("eye_r",      "body", Vec2::new( 5.0, 1.0),  1.0);
    body.add_bound("mouth",      "body", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Nyxal cub) ===
    // Mantle balloon
    body.add_cluster(&["mantle_top", "eye_l", "eye_r"], 0.7);
    body.add_cluster(&["eye_l", "body", "eye_r"], 0.7);
    body.add_cluster(&["eye_l", "body", "mouth"], 0.65);
    body.add_cluster(&["eye_r", "body", "mouth"], 0.65);
    // Tentacle bridge + stubs
    body.add_cluster(&["mouth", "tent_fl", "tent_fr"], 0.55);
    body.add_cluster(&["tent_fl", "body", "tent_fr"], 0.55);
    body.add_cluster(&["tent_bl", "body", "tent_br"], 0.55);
    body
}

// ===================================================================
// MOLUUN (mammal) — continued
// ===================================================================

/// Creates a Moluun cub soft body — exaggerated baby proportions:
/// huge head, short neck, tiny feet pressed together.
#[allow(dead_code)]
pub fn moluun_cub() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",       cx, 22.0, 14.0),
        SoftPoint::new("ear_anchor", cx, 9.0,  0.8),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 38.0, 6.0) },
        SoftPoint::new("belly",      cx, 39.0, 2.5),
        SoftPoint::new("foot_l",     cx - 3.0, 45.0, 1.5),
        SoftPoint::new("foot_r",     cx + 3.0, 45.0, 1.5),
        SoftPoint::new("eye_l",      cx - 5.0, 22.0, 0.3),
        SoftPoint::new("eye_r",      cx + 5.0, 22.0, 0.3),
        SoftPoint::new("mouth",      cx,       35.0, 0.3),
    ];

    let springs = vec![
        Spring { a: 0, b: 2, rest_length: 14.0, stiffness: 240.0, damping: 16.0 },
        Spring { a: 1, b: 0, rest_length: 13.0, stiffness: 110.0, damping: 7.0  },
        Spring { a: 2, b: 3, rest_length: 1.0,  stiffness: 200.0, damping: 15.0 },
        Spring { a: 4, b: 2, rest_length: 8.0,  stiffness: 80.0,  damping: 6.0  },
        Spring { a: 5, b: 2, rest_length: 8.0,  stiffness: 80.0,  damping: 6.0  },
        Spring { a: 4, b: 5, rest_length: 6.0,  stiffness: 40.0,  damping: 4.0  },
    ];

    let mut body = SoftBody::new(points, springs);
    body.add_bound("head",       "body", Vec2::new(0.0, -16.0), 2.0);
    body.add_bound("ear_anchor", "head", Vec2::new(0.0, -13.0), 2.5);
    body.add_bound("belly",      "body", Vec2::new(0.0,  1.0),  1.0);
    body.add_bound("foot_l",     "body", Vec2::new(-3.0, 7.0),  1.5);
    body.add_bound("foot_r",     "body", Vec2::new( 3.0, 7.0),  1.5);
    body.add_bound("eye_l",      "head", Vec2::new(-5.0, 0.0),  1.0);
    body.add_bound("eye_r",      "head", Vec2::new( 5.0, 0.0),  1.0);
    body.add_bound("mouth",      "head", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Moluun cub) ===
    // Face — head is the creature, all features rigid against it
    body.add_cluster(&["ear_anchor", "eye_l", "eye_r"], 0.75);
    body.add_cluster(&["eye_l", "head", "eye_r"], 0.75);
    body.add_cluster(&["eye_l", "head", "mouth"], 0.7);
    body.add_cluster(&["eye_r", "head", "mouth"], 0.7);
    // Neck (head→tiny body)
    body.add_cluster(&["mouth", "body", "belly"], 0.55);
    // Body + feet
    body.add_cluster(&["body", "foot_l", "foot_r"], 0.5);
    body.add_cluster(&["belly", "foot_l", "foot_r"], 0.5);
    body
}

// ===================================================================
// YOUNG STAGES — transitional bodies between cub and adult
// ===================================================================

/// Moluun young: head shrinking, body elongating, short arms appearing.
pub fn moluun_young() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",       cx, 18.0, 10.0),
        SoftPoint::new("ear_anchor", cx, 7.5, 1.2),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 32.0, 14.0) },
        SoftPoint::new("belly",      cx, 35.0, 4.0),
        SoftPoint::new("shoulder_l", cx - 13.0, 29.0, 2.0),
        SoftPoint::new("paw_l",      cx - 13.0, 36.0, 2.0),
        SoftPoint::new("shoulder_r", cx + 13.0, 29.0, 2.0),
        SoftPoint::new("paw_r",      cx + 13.0, 36.0, 2.0),
        SoftPoint::new("foot_l",     cx - 6.0, 43.0, 2.5),
        SoftPoint::new("foot_r",     cx + 6.0, 43.0, 2.5),
        SoftPoint::new("eye_l",      cx - 4.0, 19.0, 0.3),
        SoftPoint::new("eye_r",      cx + 4.0, 19.0, 0.3),
        SoftPoint::new("mouth",      cx,       31.0, 0.3),
    ];
    let springs = vec![
        Spring { a: 0, b: 2, rest_length: 14.0, stiffness: 270.0, damping: 18.0 },
        Spring { a: 1, b: 0, rest_length: 10.5, stiffness: 130.0, damping: 9.0  },
        Spring { a: 2, b: 3, rest_length: 3.0,  stiffness: 450.0, damping: 22.0 },
        Spring { a: 4, b: 2, rest_length: 13.0, stiffness: 230.0, damping: 14.0 },
        Spring { a: 5, b: 4, rest_length: 7.0,  stiffness: 180.0, damping: 11.0 },
        Spring { a: 6, b: 2, rest_length: 13.0, stiffness: 230.0, damping: 14.0 },
        Spring { a: 7, b: 6, rest_length: 7.0,  stiffness: 180.0, damping: 11.0 },
        Spring { a: 8, b: 2, rest_length: 13.0, stiffness: 280.0, damping: 16.0 },
        Spring { a: 9, b: 2, rest_length: 13.0, stiffness: 280.0, damping: 16.0 },
        Spring { a: 8, b: 9, rest_length: 12.0, stiffness: 180.0, damping: 11.0 },
        Spring { a: 4, b: 6, rest_length: 26.0, stiffness: 230.0, damping: 14.0 },
    ];
    let mut body = SoftBody::new(points, springs);
    body.add_bound("head",       "body", Vec2::new(0.0, -14.0), 2.2);
    body.add_bound("ear_anchor", "head", Vec2::new(0.0, -10.5), 2.5);
    body.add_bound("belly",      "body", Vec2::new(0.0,  3.0),  1.5);
    body.add_bound("shoulder_l", "body", Vec2::new(-13.0, -3.0), 2.0);
    body.add_bound("shoulder_r", "body", Vec2::new( 13.0, -3.0), 2.0);
    body.add_bound("paw_l",      "shoulder_l", Vec2::new(0.0, 7.0), 2.5);
    body.add_bound("paw_r",      "shoulder_r", Vec2::new(0.0, 7.0), 2.5);
    body.add_bound("foot_l",     "body", Vec2::new(-6.0, 11.0), 2.0);
    body.add_bound("foot_r",     "body", Vec2::new( 6.0, 11.0), 2.0);
    body.add_bound("eye_l",      "head", Vec2::new(-4.0, 1.0), 1.0);
    body.add_bound("eye_r",      "head", Vec2::new( 4.0, 1.0), 1.0);
    body.add_bound("mouth",      "head", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Moluun young) ===
    // Face
    body.add_cluster(&["ear_anchor", "eye_l", "eye_r"], 0.65);
    body.add_cluster(&["eye_l", "head", "eye_r"], 0.65);
    body.add_cluster(&["eye_l", "head", "mouth"], 0.6);
    body.add_cluster(&["eye_r", "head", "mouth"], 0.6);
    // Neck bridge
    body.add_cluster(&["mouth", "shoulder_l", "shoulder_r"], 0.5);
    // Torso
    body.add_cluster(&["shoulder_l", "body", "shoulder_r"], 0.55);
    body.add_cluster(&["shoulder_l", "body", "belly"], 0.5);
    body.add_cluster(&["shoulder_r", "body", "belly"], 0.5);
    // Arms (sprouting)
    body.add_cluster(&["shoulder_l", "paw_l", "body"], 0.45);
    body.add_cluster(&["shoulder_r", "paw_r", "body"], 0.45);
    // Legs
    body.add_cluster(&["body", "foot_l", "foot_r"], 0.55);
    body.add_cluster(&["belly", "foot_l", "body"], 0.45);
    body.add_cluster(&["belly", "foot_r", "body"], 0.45);
    body
}

/// Moluun elder: same silhouette as adult, slightly hunched (head lower, weaker ears).
pub fn moluun_elder() -> SoftBody {
    let mut body = moluun_adult();
    // Hunched: head ideal slightly lower, ears droopier (larger drift)
    for bound in body.bounds.iter_mut() {
        let child_name = body.points[bound.child].name.clone();
        if child_name == "head" {
            bound.rest_offset.y += 1.0; // head sits lower
            bound.max_drift = 3.0;       // looser hold
        }
        if child_name == "ear_anchor" {
            bound.max_drift = 3.5;       // droopy ears
        }
    }
    body
}

/// Pylum young: wings sprouting, casque growing, neck lengthening.
pub fn pylum_young() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",      cx, 12.0, 5.0),
        SoftPoint::new("casque",    cx, 5.0,  0.8),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 26.0, 14.0) },
        SoftPoint::new("belly",     cx, 29.0, 3.5),
        SoftPoint::new("wing_l",    cx - 11.0, 22.0, 2.5),
        SoftPoint::new("wing_r",    cx + 11.0, 22.0, 2.5),
        SoftPoint::new("wingtip_l", cx - 16.0, 27.0, 1.2),
        SoftPoint::new("wingtip_r", cx + 16.0, 27.0, 1.2),
        SoftPoint::new("tail",      cx, 34.0, 1.5),
        SoftPoint::new("foot_l",    cx - 4.0, 48.0, 3.0),
        SoftPoint::new("foot_r",    cx + 4.0, 48.0, 3.0),
        SoftPoint::new("eye_l",     cx - 3.0, 13.0, 0.3),
        SoftPoint::new("eye_r",     cx + 3.0, 13.0, 0.3),
        SoftPoint::new("mouth",     cx,       25.0, 0.3),
    ];
    let springs = vec![
        Spring { a: 0, b: 2, rest_length: 14.0, stiffness: 240.0, damping: 17.0 },
        Spring { a: 1, b: 0, rest_length: 7.0,  stiffness: 350.0, damping: 22.0 },
        Spring { a: 2, b: 3, rest_length: 3.0,  stiffness: 450.0, damping: 22.0 },
        Spring { a: 4, b: 2, rest_length: 11.0, stiffness: 180.0, damping: 13.0 },
        Spring { a: 5, b: 2, rest_length: 11.0, stiffness: 180.0, damping: 13.0 },
        Spring { a: 6, b: 4, rest_length: 6.0,  stiffness: 70.0,  damping: 5.0  },
        Spring { a: 7, b: 5, rest_length: 6.0,  stiffness: 70.0,  damping: 5.0  },
        Spring { a: 8, b: 2, rest_length: 8.0,  stiffness: 110.0, damping: 9.0  },
        Spring { a: 9, b: 2, rest_length: 22.0, stiffness: 320.0, damping: 18.0 },
        Spring { a: 10,b: 2, rest_length: 22.0, stiffness: 320.0, damping: 18.0 },
        Spring { a: 4, b: 5, rest_length: 22.0, stiffness: 180.0, damping: 11.0 },
        Spring { a: 9, b: 10,rest_length: 8.0,  stiffness: 180.0, damping: 11.0 },
    ];
    let mut body = SoftBody::new(points, springs);
    body.add_bound("head",      "body", Vec2::new(0.0, -14.0), 2.5);
    body.add_bound("casque",    "head", Vec2::new(0.0, -7.0),  2.0);
    body.add_bound("belly",     "body", Vec2::new(0.0,  3.0),  1.5);
    body.add_bound("wing_l",    "body", Vec2::new(-11.0, -4.0), 2.5);
    body.add_bound("wing_r",    "body", Vec2::new( 11.0, -4.0), 2.5);
    body.add_bound("wingtip_l", "wing_l", Vec2::new(-5.0, 5.0), 3.5);
    body.add_bound("wingtip_r", "wing_r", Vec2::new( 5.0, 5.0), 3.5);
    body.add_bound("tail",      "body", Vec2::new(0.0, 8.0), 2.5);
    body.add_bound("foot_l",    "body", Vec2::new(-4.0, 22.0), 2.0);
    body.add_bound("foot_r",    "body", Vec2::new( 4.0, 22.0), 2.0);
    body.add_bound("eye_l",     "head", Vec2::new(-3.0, 1.0), 1.0);
    body.add_bound("eye_r",     "head", Vec2::new( 3.0, 1.0), 1.0);
    body.add_bound("mouth",     "head", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Pylum young) ===
    body.add_cluster(&["casque", "eye_l", "eye_r"], 0.6);
    body.add_cluster(&["eye_l", "head", "eye_r"], 0.6);
    body.add_cluster(&["eye_l", "head", "mouth"], 0.55);
    body.add_cluster(&["eye_r", "head", "mouth"], 0.55);
    body.add_cluster(&["mouth", "wing_l", "wing_r"], 0.5);
    body.add_cluster(&["wing_l", "body", "wing_r"], 0.55);
    body.add_cluster(&["wing_l", "body", "belly"], 0.5);
    body.add_cluster(&["wing_r", "body", "belly"], 0.5);
    body.add_cluster(&["wing_l", "wingtip_l", "body"], 0.4);
    body.add_cluster(&["wing_r", "wingtip_r", "body"], 0.4);
    body.add_cluster(&["body", "tail", "belly"], 0.5);
    body.add_cluster(&["body", "foot_l", "foot_r"], 0.55);
    body.add_cluster(&["body", "foot_l", "tail"], 0.45);
    body.add_cluster(&["body", "foot_r", "tail"], 0.45);
    body
}

/// Pylum elder: hunched, wings looser.
pub fn pylum_elder() -> SoftBody {
    let mut body = pylum_adult();
    for bound in body.bounds.iter_mut() {
        let name = body.points[bound.child].name.clone();
        if name == "head" {
            bound.rest_offset.y += 1.0;
            bound.max_drift = 3.0;
        }
        if name == "wingtip_l" || name == "wingtip_r" {
            bound.max_drift = 5.0; // sagging wings
        }
    }
    body
}

/// Skael young: tail growing, horns sprouting, body lengthening.
pub fn skael_young() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("head",    cx, 16.0, 9.0),
        SoftPoint::new("horn_l",  cx - 5.0, 8.0, 0.6),
        SoftPoint::new("horn_r",  cx + 5.0, 8.0, 0.6),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 30.0, 18.0) },
        SoftPoint::new("belly",   cx, 33.0, 4.0),
        SoftPoint::new("leg_l",   cx - 12.0, 38.0, 3.0),
        SoftPoint::new("leg_r",   cx + 12.0, 38.0, 3.0),
        SoftPoint::new("foot_l",  cx - 6.0, 45.0, 2.5),
        SoftPoint::new("foot_r",  cx + 6.0, 45.0, 2.5),
        SoftPoint::new("tail_1",  cx, 40.0, 2.5),
        SoftPoint::new("tail_2",  cx, 47.0, 1.5),
        SoftPoint::new("tail_3",  cx, 52.0, 0.8),
        SoftPoint::new("eye_l",   cx - 3.0, 15.0, 0.3),
        SoftPoint::new("eye_r",   cx + 3.0, 15.0, 0.3),
        SoftPoint::new("mouth",   cx,       29.0, 0.3),
    ];
    let springs = vec![
        Spring { a: 0, b: 3, rest_length: 14.0, stiffness: 320.0, damping: 20.0 },
        Spring { a: 1, b: 0, rest_length: 9.0,  stiffness: 350.0, damping: 22.0 },
        Spring { a: 2, b: 0, rest_length: 9.0,  stiffness: 350.0, damping: 22.0 },
        Spring { a: 3, b: 4, rest_length: 3.0,  stiffness: 450.0, damping: 22.0 },
        Spring { a: 5, b: 3, rest_length: 13.0, stiffness: 270.0, damping: 16.0 },
        Spring { a: 6, b: 3, rest_length: 13.0, stiffness: 270.0, damping: 16.0 },
        Spring { a: 7, b: 3, rest_length: 17.0, stiffness: 270.0, damping: 16.0 },
        Spring { a: 8, b: 3, rest_length: 17.0, stiffness: 270.0, damping: 16.0 },
        Spring { a: 9, b: 3, rest_length: 10.0, stiffness: 220.0, damping: 14.0 },
        Spring { a: 10,b: 9, rest_length: 7.0,  stiffness: 130.0, damping: 9.0  },
        Spring { a: 11,b: 10,rest_length: 5.0,  stiffness: 70.0,  damping: 5.0  },
        Spring { a: 5, b: 6, rest_length: 24.0, stiffness: 180.0, damping: 11.0 },
        Spring { a: 7, b: 8, rest_length: 12.0, stiffness: 180.0, damping: 11.0 },
    ];
    let mut body = SoftBody::new(points, springs);
    body.add_bound("head",   "body", Vec2::new(0.0, -14.0), 2.5);
    body.add_bound("horn_l", "head", Vec2::new(-5.0, -8.0), 2.0);
    body.add_bound("horn_r", "head", Vec2::new( 5.0, -8.0), 2.0);
    body.add_bound("belly",  "body", Vec2::new(0.0, 3.0),  1.5);
    body.add_bound("leg_l",  "body", Vec2::new(-12.0, 8.0), 2.5);
    body.add_bound("leg_r",  "body", Vec2::new( 12.0, 8.0), 2.5);
    body.add_bound("foot_l", "body", Vec2::new(-6.0, 15.0), 2.0);
    body.add_bound("foot_r", "body", Vec2::new( 6.0, 15.0), 2.0);
    body.add_bound("tail_1", "body", Vec2::new(0.0, 10.0), 2.5);
    body.add_bound("tail_2", "tail_1", Vec2::new(0.0, 7.0), 3.0);
    body.add_bound("tail_3", "tail_2", Vec2::new(0.0, 5.0), 4.0);
    body.add_bound("eye_l",  "head", Vec2::new(-3.0, -1.0), 1.0);
    body.add_bound("eye_r",  "head", Vec2::new( 3.0, -1.0), 1.0);
    body.add_bound("mouth",  "head", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Skael young) ===
    body.add_cluster(&["horn_l", "horn_r", "head"], 0.6);
    body.add_cluster(&["eye_l", "head", "eye_r"], 0.6);
    body.add_cluster(&["eye_l", "head", "mouth"], 0.55);
    body.add_cluster(&["eye_r", "head", "mouth"], 0.55);
    body.add_cluster(&["mouth", "leg_l", "leg_r"], 0.5);
    body.add_cluster(&["leg_l", "body", "leg_r"], 0.6);
    body.add_cluster(&["leg_l", "body", "belly"], 0.5);
    body.add_cluster(&["leg_r", "body", "belly"], 0.5);
    body.add_cluster(&["body", "foot_l", "foot_r"], 0.55);
    body.add_cluster(&["body", "foot_l", "tail_1"], 0.5);
    body.add_cluster(&["body", "foot_r", "tail_1"], 0.5);
    body.add_cluster(&["tail_1", "tail_2", "body"], 0.5);
    body.add_cluster(&["tail_1", "tail_2", "tail_3"], 0.45);
    body
}

/// Skael elder: hunched, tail tip droopier.
pub fn skael_elder() -> SoftBody {
    let mut body = skael_adult();
    for bound in body.bounds.iter_mut() {
        let name = body.points[bound.child].name.clone();
        if name == "head" {
            bound.rest_offset.y += 1.0;
            bound.max_drift = 3.0;
        }
        if name == "tail_3" {
            bound.max_drift = 5.0; // tail drags
        }
    }
    body
}

/// Nyxal young: tentacles exploding in growth, dome shrinks proportionally.
pub fn nyxal_young() -> SoftBody {
    let cx = 32.0;
    let points = vec![
        SoftPoint::new("mantle_top", cx, 4.0, 2.5),
        SoftPoint { pinned: true, ..SoftPoint::new("body", cx, 16.0, 14.0) },
        SoftPoint::new("belly",      cx, 19.0, 2.5),
        SoftPoint::new("fin_l",      cx - 9.0, 14.0, 0.8),
        SoftPoint::new("fin_r",      cx + 9.0, 14.0, 0.8),
        SoftPoint::new("tent_fl",    cx - 4.0, 22.0, 1.5),
        SoftPoint::new("tent_fr",    cx + 4.0, 22.0, 1.5),
        SoftPoint::new("tent_bl",    cx - 11.0, 22.0, 1.5),
        SoftPoint::new("tent_br",    cx + 11.0, 22.0, 1.5),
        SoftPoint::new("tip_fl",     cx - 4.0, 42.0, 0.6),
        SoftPoint::new("tip_fr",     cx + 4.0, 42.0, 0.6),
        SoftPoint::new("tip_bl",     cx - 11.0, 39.0, 0.6),
        SoftPoint::new("tip_br",     cx + 11.0, 39.0, 0.6),
        SoftPoint::new("eye_l",      cx - 4.0, 16.0, 0.3),
        SoftPoint::new("eye_r",      cx + 4.0, 16.0, 0.3),
        SoftPoint::new("mouth",      cx,       29.0, 0.3),
    ];
    let springs = vec![
        Spring { a: 0, b: 1, rest_length: 12.0, stiffness: 280.0, damping: 18.0 },
        Spring { a: 1, b: 2, rest_length: 3.0,  stiffness: 380.0, damping: 18.0 },
        Spring { a: 3, b: 1, rest_length: 9.0,  stiffness: 70.0,  damping: 5.0  },
        Spring { a: 4, b: 1, rest_length: 9.0,  stiffness: 70.0,  damping: 5.0  },
        Spring { a: 5, b: 1, rest_length: 8.0,  stiffness: 160.0, damping: 11.0 },
        Spring { a: 6, b: 1, rest_length: 8.0,  stiffness: 160.0, damping: 11.0 },
        Spring { a: 7, b: 1, rest_length: 13.0, stiffness: 130.0, damping: 9.0  },
        Spring { a: 8, b: 1, rest_length: 13.0, stiffness: 130.0, damping: 9.0  },
        Spring { a: 9, b: 5, rest_length: 20.0, stiffness: 50.0,  damping: 4.0  },
        Spring { a: 10,b: 6, rest_length: 20.0, stiffness: 50.0,  damping: 4.0  },
        Spring { a: 11,b: 7, rest_length: 17.0, stiffness: 50.0,  damping: 4.0  },
        Spring { a: 12,b: 8, rest_length: 17.0, stiffness: 50.0,  damping: 4.0  },
        Spring { a: 5, b: 6, rest_length: 8.0,  stiffness: 90.0,  damping: 7.0  },
        Spring { a: 7, b: 8, rest_length: 22.0, stiffness: 70.0,  damping: 5.0  },
    ];
    let mut body = SoftBody::new(points, springs);
    body.add_bound("mantle_top", "body", Vec2::new(0.0, -12.0), 2.0);
    body.add_bound("belly",      "body", Vec2::new(0.0,  3.0),  1.5);
    body.add_bound("fin_l",      "body", Vec2::new(-9.0, -2.0), 3.5);
    body.add_bound("fin_r",      "body", Vec2::new( 9.0, -2.0), 3.5);
    body.add_bound("tent_fl",    "body", Vec2::new(-4.0, 6.0),  2.5);
    body.add_bound("tent_fr",    "body", Vec2::new( 4.0, 6.0),  2.5);
    body.add_bound("tent_bl",    "body", Vec2::new(-11.0, 6.0), 2.5);
    body.add_bound("tent_br",    "body", Vec2::new( 11.0, 6.0), 2.5);
    body.add_bound("tip_fl",     "tent_fl", Vec2::new(0.0, 20.0), 5.0);
    body.add_bound("tip_fr",     "tent_fr", Vec2::new(0.0, 20.0), 5.0);
    body.add_bound("tip_bl",     "tent_bl", Vec2::new(0.0, 17.0), 5.0);
    body.add_bound("tip_br",     "tent_br", Vec2::new(0.0, 17.0), 5.0);
    body.add_bound("eye_l",      "body", Vec2::new(-4.0, 0.0), 1.0);
    body.add_bound("eye_r",      "body", Vec2::new( 4.0, 0.0), 1.0);
    body.add_bound("mouth",      "body", Vec2::new( 0.0, 13.0), 1.0);

    // === TRIANGULATED MESH (Nyxal young) ===
    body.add_cluster(&["mantle_top", "eye_l", "eye_r"], 0.65);
    body.add_cluster(&["mantle_top", "fin_l", "body"], 0.55);
    body.add_cluster(&["mantle_top", "fin_r", "body"], 0.55);
    body.add_cluster(&["eye_l", "body", "eye_r"], 0.6);
    body.add_cluster(&["eye_l", "body", "mouth"], 0.55);
    body.add_cluster(&["eye_r", "body", "mouth"], 0.55);
    body.add_cluster(&["mouth", "tent_fl", "tent_fr"], 0.5);
    body.add_cluster(&["tent_fl", "body", "tent_fr"], 0.5);
    body.add_cluster(&["tent_bl", "body", "tent_br"], 0.5);
    body.add_cluster(&["tent_fl", "body", "tent_bl"], 0.45);
    body.add_cluster(&["tent_fr", "body", "tent_br"], 0.45);
    body.add_cluster(&["tent_fl", "tip_fl", "body"], 0.35);
    body.add_cluster(&["tent_fr", "tip_fr", "body"], 0.35);
    body.add_cluster(&["tent_bl", "tip_bl", "body"], 0.35);
    body.add_cluster(&["tent_br", "tip_br", "body"], 0.35);
    body
}

/// Nyxal elder: dome dimmer, tentacle tips droopier.
pub fn nyxal_elder() -> SoftBody {
    let mut body = nyxal_adult();
    for bound in body.bounds.iter_mut() {
        let name = body.points[bound.child].name.clone();
        if name == "mantle_top" {
            bound.rest_offset.y += 1.0;
            bound.max_drift = 3.0;
        }
        if name.starts_with("tip_") {
            bound.max_drift = 8.0; // tips sag deep
        }
    }
    body
}
