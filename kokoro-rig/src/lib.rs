//! Skeletal rig for Kokoro creatures.
//!
//! ## What this crate is
//!
//! A **deterministic, headless** skeletal animation core: hierarchical bones,
//! forward kinematics, and (in future modules) inverse kinematics, pose
//! layers, and z-order rendering. No Bevy dependency, no global state, no
//! random side channels. Same input → same output, every time.
//!
//! ## Why a separate crate
//!
//! The visual layer in `kokoro` is currently coupled to Bevy resources
//! (`SoftBody`, `Mind`, etc.) and to the per-species draw modules. The rig is
//! supposed to be the **substrate** under that layer — bones decide where
//! limbs *are*, brushes only paint pixels at those positions. Keeping the rig
//! crate Bevy-free lets us:
//!
//! 1. Test rig math without spinning up a Bevy world.
//! 2. Reuse the same skeleton data in headless tools (snapshot generators,
//!    documentation diagrams, future Web Leptos rigging previews).
//! 3. Swap rendering backends later without rewriting the rig.
//!
//! ## Reference doctrine
//!
//! Whitlatch's *Science of Creature Design* sets the rule we follow:
//! **skeleton first, surface last.** A creature's silhouette is a consequence
//! of bones (where the joints are) plus surface tissue (fat, fur, scales,
//! feathers). This crate owns the first half. The painting/brush DSL in
//! `kokoro-art-palette` and the species draw modules own the second.

pub mod bone;
pub mod fabrik;
pub mod ik;
pub mod skeleton;

pub use bone::{Bone, BoneId, Vec2};
pub use fabrik::FabrikChain;
pub use ik::{solve_two_bone, BendDirection, TwoBoneResult};
pub use skeleton::Skeleton;
