//! Per-species skeletal rigs.
//!
//! Each module here defines the bone hierarchy for one species at one (or
//! more) life stages. Skeletons are pure data + a `kokoro_rig::Skeleton`
//! constructor; brushes paint at the bones' world positions, the
//! `softbody::reconcile` pass marries skeleton state with the live
//! soft-body simulation each frame.
//!
//! Rigs are deliberately separated from skin/draw modules: anatomy is one
//! concern (where the joints are), surface treatment is another (what the
//! pixels look like). Whitlatch's *Science of Creature Design* puts this
//! plainly — skeleton first, skin last.

pub mod moluun;
