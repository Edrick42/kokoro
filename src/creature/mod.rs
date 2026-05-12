//! Creature systems — everything that makes a Kobara alive.

// Slow-biology health (bone density, muscle conditioning, joint integrity, skin).
// Distinct from kokoro-rig / kokoro-body (per-frame physics).
pub mod physiology;

// Per-species rig templates + Bevy plugins that step them each frame.
// Lives here (not under visuals/) because rigs produce motion, not pixels.
pub mod biomechanics;

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
