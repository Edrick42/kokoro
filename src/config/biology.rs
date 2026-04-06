//! Biological system constants — breathing, heartbeat, resonance, growth.

pub mod breathing {
    pub const DEFAULT_RATE: f32 = 0.22;
    pub const DEFAULT_AMPLITUDE: f32 = 0.012;

    pub const RATE_SLEEPING: f32 = 0.12;
    pub const RATE_TIRED: f32 = 0.18;
    pub const RATE_HAPPY: f32 = 0.22;
    pub const RATE_HUNGRY: f32 = 0.25;
    pub const RATE_SICK: f32 = 0.30;
    pub const RATE_PLAYFUL: f32 = 0.40;

    pub const AMPLITUDE_BASE: f32 = 0.008;
    pub const AMPLITUDE_ENERGY_FACTOR: f32 = 0.012;
    pub const Y_AMPLITUDE_RATIO: f32 = 0.7;
}

pub mod heartbeat {
    pub const DEFAULT_BPM: f32 = 72.0;
    pub const BASE_BPM: f32 = 50.0;
    pub const HEALTH_BPM_FACTOR: f32 = 0.3;

    pub const BPM_PLAYFUL: f32 = 15.0;
    pub const BPM_SLEEPING: f32 = -15.0;
    pub const BPM_SICK: f32 = 8.0;

    pub const MIN_BPM: f32 = 30.0;
    pub const SICK_IRREGULARITY: f32 = 0.4;
    pub const IRREGULARITY_JITTER: f32 = 0.3;
    pub const PULSE_DURATION: f32 = 0.12;
}

pub mod resonance {
    pub const FREQ_SLEEPING: f32 = 0.3;
    pub const FREQ_TIRED: f32 = 0.8;
    pub const FREQ_HAPPY: f32 = 1.5;
    pub const FREQ_PLAYFUL: f32 = 2.0;
    pub const FREQ_HUNGRY: f32 = 3.0;
    pub const FREQ_LONELY: f32 = 4.0;
    pub const FREQ_SICK: f32 = 5.0;

    pub const INTENSITY_SLEEPING: f32 = 0.3;
    pub const INTENSITY_TIRED: f32 = 0.4;
    pub const INTENSITY_HAPPY: f32 = 0.6;
    pub const INTENSITY_HUNGRY: f32 = 0.7;
    pub const INTENSITY_LONELY: f32 = 0.8;
    pub const INTENSITY_PLAYFUL: f32 = 0.8;
    pub const INTENSITY_SICK: f32 = 0.9;

    pub const SICK_JITTER_FREQ: f32 = 3.7;
    pub const SICK_JITTER_AMP: f32 = 0.4;
    pub const SCALE_AMP: f32 = 0.15;
}

pub mod growth {
    /// Age thresholds in ticks (1 tick = 1 real second).
    pub const CUB_MAX: u64 = 1_200_000;     // Cub: ~14 days (2 weeks)
    pub const YOUNG_MAX: u64 = 3_800_000;   // Young: ~44 days (~1.5 months)
    pub const ADULT_MAX: u64 = 8_500_000;   // Adult: ~98 days (~3.3 months)

    pub const CUB_SCALE: f32 = 0.6;
    pub const YOUNG_SCALE: f32 = 0.8;
    pub const ADULT_SCALE: f32 = 1.0;
    pub const ELDER_SCALE: f32 = 0.95;

    pub const SCALE_LERP_SPEED: f32 = 0.5;
}

pub mod egg {
    /// Natural incubation without interaction: ~3 days.
    pub const NATURAL_INCUBATION_TICKS: f32 = 259_200.0; // 3 days in seconds
    /// Each player tap adds ~0.5% progress.
    pub const TAP_BOOST: f32 = 0.005;
}
