//! Anatomy system constants — skeleton, muscles, joints, skin.
//!
//! Each creature has a four-layer anatomy that works like real biology:
//! Skeleton (structure) → Joints (connections) → Muscles (force) → Skin (protection).
//! These constants define species baselines, decay/repair rates, and thresholds.

/// Bone structure constants per species.
pub mod skeleton {
    /// Base bone density per species (0.0-1.0).
    pub const DENSITY_MOLUUN: f32 = 0.6;
    pub const DENSITY_PYLUM: f32 = 0.3;
    pub const DENSITY_SKAEL: f32 = 0.85;
    pub const DENSITY_NYXAL: f32 = 0.0; // hydrostatic — no bones

    /// Nyxal uses hydrostatic pressure instead of bone density.
    pub const HYDROSTATIC_PRESSURE_NYXAL: f32 = 0.8;

    /// Skeleton type damage/resistance modifiers.
    pub const HOLLOW_DAMAGE_MULTIPLIER: f32 = 1.5;
    pub const DENSE_RESISTANCE_MULTIPLIER: f32 = 0.6;
    pub const DENSE_REPAIR_BONUS: f32 = 0.00002;

    /// Per-tick decay rates (when malnourished/sick).
    pub const BONE_HEALTH_DECAY: f32 = 0.0003;
    pub const BONE_INTEGRITY_DECAY: f32 = 0.0001;

    /// Per-tick repair rates (when well-fed).
    pub const BONE_HEALTH_REPAIR: f32 = 0.0001;
    pub const BONE_INTEGRITY_REPAIR: f32 = 0.00005;

    /// Nyxal hydrostatic pressure decay when dehydrated.
    pub const HYDROSTATIC_DECAY: f32 = 0.001;
    pub const HYDROSTATIC_REPAIR: f32 = 0.0005;

    /// Health ceiling: max_health = bone_health × avg_integrity × this.
    pub const HEALTH_CEILING_MULTIPLIER: f32 = 100.0;

    /// Bone break cascade effects.
    pub const BREAK_HEALTH_PENALTY: f32 = 10.0;
    pub const BREAK_JOINT_FLEX_MIN: f32 = 0.1;
    pub const BREAK_JOINT_INTEGRITY_CAP: f32 = 0.3;
    pub const BREAK_MUSCLE_STRENGTH_FACTOR: f32 = 0.5;
}

/// Muscle system constants per species.
pub mod muscles {
    /// Base muscle mass per species (0.0-1.0).
    pub const MASS_MOLUUN: f32 = 0.5;
    pub const MASS_PYLUM: f32 = 0.4;
    pub const MASS_SKAEL: f32 = 0.7;
    pub const MASS_NYXAL: f32 = 0.3;

    /// Per-tick rates.
    pub const CONDITION_DECAY: f32 = 0.0004;
    pub const CONDITION_REPAIR: f32 = 0.0002;
    pub const FATIGUE_ACCUMULATION: f32 = 0.0008;
    pub const FATIGUE_RECOVERY: f32 = 0.005;

    /// Tone converges toward condition at this rate per tick.
    pub const TONE_CONVERGENCE: f32 = 0.001;

    /// Below this condition, energy penalty applies.
    pub const LOW_CONDITION_THRESHOLD: f32 = 0.5;
    /// Energy penalty per tick = (threshold - condition) × this.
    pub const ENERGY_PENALTY_FACTOR: f32 = 0.02;
}

/// Joint system constants per species.
pub mod joints {
    /// Base joint flexibility per species (0.0-1.0).
    pub const FLEX_MOLUUN: f32 = 0.7;
    pub const FLEX_PYLUM: f32 = 0.8;
    pub const FLEX_SKAEL: f32 = 0.4;
    pub const FLEX_NYXAL: f32 = 0.95;

    /// Per-tick rates.
    pub const LUBRICATION_DECAY: f32 = 0.0002;
    pub const LUBRICATION_REPAIR: f32 = 0.0001;

    /// Below this lubrication, stiffness energy penalty applies.
    pub const STIFFNESS_THRESHOLD: f32 = 0.3;
    pub const STIFFNESS_ENERGY_PENALTY: f32 = 0.01;

    /// Elders lose flexibility irreversibly at this rate per tick.
    pub const ELDER_FLEXIBILITY_DECAY: f32 = 0.00001;

    /// Below this flexibility, creature cannot enter Playful mood.
    pub const PLAYFUL_FLEX_BLOCK: f32 = 0.3;
}

/// Skin layer constants per species.
pub mod skin {
    /// Base skin thickness per species (0.0-1.0).
    pub const THICKNESS_MOLUUN: f32 = 0.5;
    pub const THICKNESS_PYLUM: f32 = 0.3;
    pub const THICKNESS_SKAEL: f32 = 0.8;
    pub const THICKNESS_NYXAL: f32 = 0.15;

    /// Per-tick rates.
    pub const HYDRATION_DECAY: f32 = 0.0003;
    pub const HYDRATION_REPAIR: f32 = 0.0002;
    pub const INTEGRITY_DECAY: f32 = 0.0002;
    pub const INTEGRITY_REPAIR: f32 = 0.00015;
}

/// Fat reserve constants per species.
pub mod fat {
    /// Base fat level for each species (0.0-1.0).
    pub const LEVEL_MOLUUN: f32 = 0.5;  // moderate reserves
    pub const LEVEL_PYLUM: f32 = 0.3;   // lean (flight efficiency)
    pub const LEVEL_SKAEL: f32 = 0.6;   // higher reserves (cold caves)
    pub const LEVEL_NYXAL: f32 = 0.4;   // moderate (deep sea pressure)

    /// Per-tick burn rate when hungry.
    pub const BURN_RATE: f32 = 0.0005;
    /// Per-tick store rate when well-fed.
    pub const STORE_RATE: f32 = 0.0003;
    /// Fat threshold below which muscles start atrophying.
    pub const MUSCLE_PROTECTION_THRESHOLD: f32 = 0.2;
    /// Insulation factor per unit of fat.
    pub const INSULATION_FACTOR: f32 = 0.3;
}

/// Growth stage multipliers for anatomy values.
/// (bone_density, muscle_mass, joint_flexibility, skin_thickness)
pub mod growth_anatomy {
    pub const CUB:   (f32, f32, f32, f32) = (0.5, 0.3, 1.3, 0.6);
    pub const YOUNG: (f32, f32, f32, f32) = (0.7, 0.6, 1.15, 0.8);
    pub const ADULT: (f32, f32, f32, f32) = (1.0, 1.0, 1.0, 1.0);
    pub const ELDER: (f32, f32, f32, f32) = (0.75, 0.7, 0.6, 0.85);
}

/// Hunger-based proxy thresholds (used until full nutrition system exists).
/// When hunger exceeds these, anatomy degrades. Below repair threshold, it heals.
pub const HUNGER_DECAY_THRESHOLD: f32 = 80.0;
pub const HUNGER_JOINT_THRESHOLD: f32 = 60.0;
pub const HUNGER_SKIN_THRESHOLD: f32 = 70.0;
pub const HUNGER_REPAIR_THRESHOLD: f32 = 50.0;
pub const HUNGER_MUSCLE_THRESHOLD: f32 = 75.0;
pub const HUNGER_MUSCLE_REPAIR: f32 = 40.0;

/// Ribcage integrity threshold — below this, kokoro-sac resonance dampened.
pub const RIBCAGE_RESONANCE_THRESHOLD: f32 = 0.5;
/// Happiness gain reduction when ribcage damaged.
pub const RIBCAGE_HAPPINESS_PENALTY: f32 = 0.2;
