//! Species-specific action values — how each species responds to feed, play, sleep.

/// (hunger_relief, happiness_boost) when fed.
pub mod feed {
    pub const MOLUUN: (f32, f32) = (25.0, 12.0);
    pub const PYLUM: (f32, f32) = (18.0, 5.0);
    pub const SKAEL: (f32, f32) = (35.0, 4.0);
    pub const NYXAL: (f32, f32) = (15.0, 8.0);
}

/// (happiness_boost, energy_cost, hunger_cost) when played with.
pub mod play {
    pub const MOLUUN: (f32, f32, f32) = (18.0, 8.0, 5.0);
    pub const PYLUM: (f32, f32, f32) = (15.0, 12.0, 8.0);
    pub const SKAEL: (f32, f32, f32) = (8.0, 5.0, 3.0);
    pub const NYXAL: (f32, f32, f32) = (12.0, 6.0, 4.0);
}

/// Energy restored when put to sleep.
pub mod sleep {
    pub const MOLUUN: f32 = 30.0;
    pub const PYLUM: f32 = 22.0;
    pub const SKAEL: f32 = 38.0;
    pub const NYXAL: f32 = 28.0;
}
