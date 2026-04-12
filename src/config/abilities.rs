//! Species ability configuration — cooldowns, costs, durations.

/// Moluun: Scent Trail — passive, marks territory with scent particles.
pub mod scent_trail {
    pub const TRIGGER_INTERVAL: u32 = 30;    // ticks between scent marks
    pub const PARTICLE_DURATION: u32 = 60;   // how long a mark lasts
    pub const COMFORT_RADIUS: f32 = 100.0;   // visual radius of scent effect
}

/// Pylum: Thermal Sight — active, briefly highlights warm areas.
pub mod thermal_sight {
    pub const DURATION: u32 = 15;            // ticks the overlay lasts
    pub const COOLDOWN: u32 = 120;           // ticks between uses
    pub const ENERGY_COST: f32 = 8.0;
}

/// Skael: Armored Curl — active defensive pose.
pub mod armored_curl {
    pub const DURATION: u32 = 20;            // ticks curled up
    pub const COOLDOWN: u32 = 90;
    pub const DAMAGE_REDUCTION: f32 = 0.7;   // 70% less damage
    pub const ENERGY_RECOVERY: f32 = 5.0;    // recovers energy while curled
}

/// Nyxal: Chromatophore Pulse — passive, body color shifts with mood.
pub mod chromatophore {
    pub const PULSE_SPEED: f32 = 0.5;        // color shift speed
    pub const INTENSITY_MIN: f32 = 0.2;      // base glow when calm
    pub const INTENSITY_MAX: f32 = 1.0;      // full glow when excited
}
