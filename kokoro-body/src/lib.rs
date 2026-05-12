//! Neuromuscular layer for Kokoro creatures.
//!
//! ## What this crate is
//!
//! The body sits **above** `kokoro-rig` (bones + joints + physics
//! integrator) and **below** the mind (intent / behavior). Its job is to
//! turn intentions into joint torques, with the same physical-causality
//! discipline as the rig: every motion must have a traceable physical
//! cause — nerve signal, muscle contraction, ligament spring, contact.
//!
//! ## The five-layer model
//!
//! ```text
//!     Mind (intent: "flick the tail", "tuck legs")
//!       │
//!       ▼
//!     Nerve (carries signal with latency, attenuation, health)
//!       │
//!       ▼
//!     Muscle (force = max_force × activation × (1 − fatigue))
//!       │
//!       ▼
//!     Joint (sums muscle torques + spring + friction + gravity + contact;
//!            integrates angle)
//!       │
//!       ▼
//!     Bone (rigid structural element; FK propagates joint angles to
//!           world positions for the renderer)
//! ```
//!
//! Each layer is its own struct, never folded into another. Pathology
//! (torn ligament, weak nerve, atrophied muscle, fractured bone) targets
//! a single layer without leaking into the others.
//!
//! ## Forward-compatibility
//!
//! - [`Muscle`] holds a [`MuscleComposition`] enum (currently
//!   `Aggregate(max_force)`; future variant `Fibers(Vec<Fiber>)` for
//!   fibre-level detail).
//! - [`Nerve`] holds a [`NervePathway`] enum (currently
//!   `Direct { latency_ms, attenuation }`; future variant
//!   `Network(...)` for individual neurons).
//! - Adding variants to these enums is a forward-compatible change.

pub mod muscle;
pub mod nerve;
pub mod pair;
pub mod actuation;
pub mod body;

pub use muscle::{Muscle, MuscleAttachment, MuscleComposition};
pub use nerve::{Nerve, NervePathway};
pub use pair::MusclePair;
pub use actuation::actuate;
pub use body::Body;
