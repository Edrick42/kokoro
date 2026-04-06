//! Physics constants — gravity, collision, buoyancy, impulses.

/// Ground plane Y position.
pub const GROUND_Y: f32 = -60.0;
/// Max horizontal position before soft wall bounce.
pub const MAX_X: f32 = 160.0;
/// Wall bounce energy retention.
pub const WALL_BOUNCE: f32 = 0.3;
/// Maximum frame delta time (skip physics on huge spikes).
pub const MAX_DT: f32 = 0.2;

/// Land creature physics defaults.
pub mod land {
    pub const GRAVITY: f32 = 400.0;
    pub const BOUNCE_FACTOR: f32 = 0.3;
    pub const FRICTION: f32 = 0.85;
}

/// Aquatic creature physics defaults.
pub mod aquatic {
    pub const BUOYANCY_STRENGTH: f32 = 120.0;
    pub const FRICTION: f32 = 0.92;
    pub const DAMPING: f32 = 0.98;
    pub const FLOOR_OFFSET: f32 = 100.0;
    pub const BOUNCE_FACTOR: f32 = 0.1;
}

/// Mood-triggered impulses.
pub mod impulse {
    pub const PLAYFUL_JUMP_MIN: f32 = 80.0;
    pub const PLAYFUL_JUMP_MAX: f32 = 150.0;
    pub const SLEEPING_SLUMP: f32 = -30.0;
    pub const SICK_STUMBLE_MIN: f32 = -50.0;
    pub const SICK_STUMBLE_MAX: f32 = 50.0;
}

/// Velocity thresholds for state transitions.
pub mod threshold {
    pub const BOUNCE_MIN: f32 = -10.0;
    pub const GROUNDED_MIN: f32 = 1.0;
    pub const FRICTION_STOP: f32 = 0.5;
}
