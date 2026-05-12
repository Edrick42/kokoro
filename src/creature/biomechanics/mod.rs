//! Runtime biomechanics — Bevy-side wiring of `kokoro-rig` + `kokoro-body`.
//!
//! Each species has two files here:
//! - `<species>.rs` defines the **template**: bone names, genes → physical
//!   params, builders for `Skeleton`/`Body`, mood-driven muscle intent.
//!   No Bevy types — pure data + functions so the templates are usable
//!   in tests and snapshot tools.
//! - `<species>_runtime.rs` defines the **Bevy plugin** that owns the
//!   live `Body` resource for the active creature and steps it each
//!   frame using `Mind::mood`.
//!
//! Renderers (`src/visuals/skin/<species>.rs`) never invent motion —
//! they only paint the post-FK state of bodies built here.

#[cfg(feature = "dev")]
pub mod debug_overlay;
pub mod moluun;
pub mod moluun_runtime;
