//! Game configuration — all tunable constants organized by domain.
//!
//! If you need to tweak a value, find the relevant submodule.
//! No magic numbers should exist outside this module.

pub mod stats;
pub mod species;
pub mod physics;
pub mod biology;
pub mod communication;
pub mod lifecycle;
pub mod nervous_system;
pub mod nutrition;
pub mod timing;
pub mod absence;
pub mod slots;
pub mod anatomy;

// Re-export at top level so existing `crate::config::X` paths keep working.
pub use stats::{initial_stats, mood_thresholds, stat_decay, mood};
pub use species::{feed, play, sleep};
pub use biology::{breathing, heartbeat, resonance, growth, egg};
pub use timing::{TICK_INTERVAL, AUTOSAVE_INTERVAL};
