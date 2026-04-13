//! Creature systems — everything that makes a Kobara alive.

// Body simulation (skeleton, joints, muscles, skin, fat)
pub mod anatomy;

// Behavior (pose, reactions, idle, involuntary)
pub mod behavior;

// Lifecycle (egg, collection, spawn, reproduction)
pub mod lifecycle;

// Physical interaction (gravity, collision, touch)
pub mod interaction;

// Visual identity (species templates, proportional rig)
pub mod identity;

// Species-unique powers
#[allow(dead_code, unused_imports, unused_variables)]
pub mod abilities;
