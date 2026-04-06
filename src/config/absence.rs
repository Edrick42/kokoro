//! Mirror Bond — absence time brackets and reunion behavior.

/// Time brackets for absence effects (seconds).
pub const TRIVIAL: u64 = 60;       // < 1 min: no effect
pub const SHORT: u64 = 1800;       // 30 min
pub const MEDIUM: u64 = 14400;     // 4 hours
pub const LONG: u64 = 86400;       // 24 hours

/// Ticks of reunion animation after returning.
pub const REUNION_TICKS: u32 = 5;
