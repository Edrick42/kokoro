//! # Persistence
//!
//! Handles saving and loading the Kobara's full state to a local SQLite database.
//!
//! ## Database location
//! - Linux/macOS: `~/.config/kokoro/save.db`
//! - Windows:     `%APPDATA%\kokoro\save.db`
//!
//! ## Schema — 3 tables
//!
//! ### `creature`
//! One row only. Stores the current vital stats and mood.
//!
//! ### `genome`
//! One row only. Stores the immutable genetic blueprint of this Kobara.
//! Written once on creation, never overwritten.
//!
//! ### `event_log`
//! Append-only log of significant events (fed, played, mood changes, etc.).
//! This becomes the training data for the Phase 4 neural network.

pub mod db;
pub mod plugin;
pub mod save;
pub mod load;
