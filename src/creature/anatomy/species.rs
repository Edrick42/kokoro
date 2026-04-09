//! Species-specific anatomy initializers.
//!
//! Each species has a unique skeleton, muscle layout, joint configuration,
//! and skin type. Genes modify the base values to produce individual variation.

use crate::config::anatomy as cfg;
use crate::genome::{Genome, Species};

use super::skeleton::{self, Skeleton, SkeletonType};
use super::muscles::{self, MuscleSystem};
use super::joints::{self, JointSystem};
use super::skin::{SkinLayer, SkinCovering};
use super::AnatomyState;

impl AnatomyState {
    /// Creates the anatomy for a given species, using the genome for individual variation.
    pub fn new_for(species: &Species, genome: &Genome) -> Self {
        match species {
            Species::Moluun => moluun(genome),
            Species::Pylum  => pylum(genome),
            Species::Skael  => skael(genome),
            Species::Nyxal  => nyxal(genome),
        }
    }
}

fn moluun(genome: &Genome) -> AnatomyState {
    let density = cfg::skeleton::DENSITY_MOLUUN * (0.9 + genome.resilience * 0.2);
    let mass = cfg::muscles::MASS_MOLUUN * (1.1 - genome.appetite * 0.2);
    let flex = cfg::joints::FLEX_MOLUUN * (0.95 + genome.curiosity * 0.1);

    AnatomyState {
        skeleton: Skeleton {
            structure_type: SkeletonType::Standard,
            bone_density: density,
            bone_health: 1.0,
            hydrostatic_pressure: 0.0,
            bones: vec![
                skeleton::bone("skull",      "body", 1.2),
                skeleton::bone("spine",      "body", 1.0),
                skeleton::bone("ribcage",    "body", 1.1),
                skeleton::bone("pelvis",     "body", 1.0),
                skeleton::bone("limb_left",  "body", 0.8),
                skeleton::bone("limb_right", "body", 0.8),
            ],
        },
        muscles: MuscleSystem {
            mass,
            condition: 1.0,
            tone: mass,
            groups: vec![
                muscles::muscle("core",       "neck"),
                muscles::muscle("upper",      "shoulder_left"),
                muscles::muscle("legs_left",  "hip_left"),
                muscles::muscle("legs_right", "hip_right"),
            ],
        },
        joints: JointSystem {
            joints: vec![
                joints::joint("neck",           "skull",  "spine",      flex),
                joints::joint("shoulder_left",  "spine",  "limb_left",  flex),
                joints::joint("shoulder_right", "spine",  "limb_right", flex),
                joints::joint("hip_left",       "pelvis", "limb_left",  flex),
                joints::joint("hip_right",      "pelvis", "limb_right", flex),
            ],
        },
        skin: SkinLayer {
            covering: SkinCovering::Fur,
            integrity: 1.0,
            thickness: cfg::skin::THICKNESS_MOLUUN,
            hydration: 1.0,
        },
    }
}

fn pylum(genome: &Genome) -> AnatomyState {
    let density = cfg::skeleton::DENSITY_PYLUM * (0.9 + genome.resilience * 0.2);
    let mass = cfg::muscles::MASS_PYLUM * (1.1 - genome.appetite * 0.2);
    let flex = cfg::joints::FLEX_PYLUM * (0.95 + genome.curiosity * 0.1);

    AnatomyState {
        skeleton: Skeleton {
            structure_type: SkeletonType::Hollow,
            bone_density: density,
            bone_health: 1.0,
            hydrostatic_pressure: 0.0,
            bones: vec![
                skeleton::bone("skull",           "body",       1.1),
                skeleton::bone("spine",           "body",       1.0),
                skeleton::bone("ribcage",         "body",       1.0),
                skeleton::bone("keel",            "body",       1.2),
                skeleton::bone("wing_bone_left",  "wing_left",  0.7),
                skeleton::bone("wing_bone_right", "wing_right", 0.7),
                skeleton::bone("tail_bone",       "tail",       0.6),
            ],
        },
        muscles: MuscleSystem {
            mass,
            condition: 1.0,
            tone: mass,
            groups: vec![
                muscles::muscle("pectorals",    "wing_left"),
                muscles::muscle("core",         "neck"),
                muscles::muscle("tail_muscles", "tail"),
                muscles::muscle("legs",         "keel_joint"),
            ],
        },
        joints: JointSystem {
            joints: vec![
                joints::joint("neck",       "skull",   "spine",          flex),
                joints::joint("wing_left",  "spine",   "wing_bone_left", flex),
                joints::joint("wing_right", "spine",   "wing_bone_right",flex),
                joints::joint("keel_joint", "ribcage", "keel",           flex * 0.8),
                joints::joint("tail",       "spine",   "tail_bone",      flex),
            ],
        },
        skin: SkinLayer {
            covering: SkinCovering::Plumage,
            integrity: 1.0,
            thickness: cfg::skin::THICKNESS_PYLUM,
            hydration: 1.0,
        },
    }
}

fn skael(genome: &Genome) -> AnatomyState {
    let density = cfg::skeleton::DENSITY_SKAEL * (0.9 + genome.resilience * 0.2);
    let mass = cfg::muscles::MASS_SKAEL * (1.1 - genome.appetite * 0.2);
    let flex = cfg::joints::FLEX_SKAEL * (0.95 + genome.curiosity * 0.1);

    AnatomyState {
        skeleton: Skeleton {
            structure_type: SkeletonType::Dense,
            bone_density: density,
            bone_health: 1.0,
            hydrostatic_pressure: 0.0,
            bones: vec![
                skeleton::bone("skull",      "body", 1.3),
                skeleton::bone("spine",      "body", 1.0),
                skeleton::bone("ribcage",    "body", 1.2),
                skeleton::bone("pelvis",     "body", 1.1),
                skeleton::bone("limb_left",  "body", 0.9),
                skeleton::bone("limb_right", "body", 0.9),
                skeleton::bone("tail_bone",  "tail", 0.8),
            ],
        },
        muscles: MuscleSystem {
            mass,
            condition: 1.0,
            tone: mass,
            groups: vec![
                muscles::muscle("core",         "neck"),
                muscles::muscle("upper",        "shoulder_left"),
                muscles::muscle("legs_left",    "hip_left"),
                muscles::muscle("legs_right",   "hip_right"),
                muscles::muscle("tail_muscles", "tail"),
            ],
        },
        joints: JointSystem {
            joints: vec![
                joints::joint("neck",           "skull",  "spine",      flex),
                joints::joint("shoulder_left",  "spine",  "limb_left",  flex),
                joints::joint("shoulder_right", "spine",  "limb_right", flex),
                joints::joint("hip_left",       "pelvis", "limb_left",  flex),
                joints::joint("hip_right",      "pelvis", "limb_right", flex),
                joints::joint("tail",           "pelvis", "tail_bone",  flex * 1.2),
            ],
        },
        skin: SkinLayer {
            covering: SkinCovering::Scales,
            integrity: 1.0,
            thickness: cfg::skin::THICKNESS_SKAEL,
            hydration: 1.0,
        },
    }
}

fn nyxal(genome: &Genome) -> AnatomyState {
    let mass = cfg::muscles::MASS_NYXAL * (1.1 - genome.appetite * 0.2);
    let flex = cfg::joints::FLEX_NYXAL * (0.95 + genome.curiosity * 0.1);
    let pressure = cfg::skeleton::HYDROSTATIC_PRESSURE_NYXAL * (0.9 + genome.resilience * 0.2);

    AnatomyState {
        skeleton: Skeleton {
            structure_type: SkeletonType::Hydrostatic,
            bone_density: 0.0,
            bone_health: 1.0,
            hydrostatic_pressure: pressure,
            bones: vec![],
        },
        muscles: MuscleSystem {
            mass,
            condition: 1.0,
            tone: mass,
            groups: vec![
                muscles::muscle("mantle_muscles",   "mantle_base"),
                muscles::muscle("tentacle_muscles", "tentacle_front_left"),
                muscles::muscle("siphon",           "mantle_base"),
            ],
        },
        joints: JointSystem {
            joints: vec![
                joints::joint("mantle_base",          "mantle", "body",                 flex),
                joints::joint("tentacle_front_left",  "body",   "tentacle_front_left",  flex),
                joints::joint("tentacle_front_right", "body",   "tentacle_front_right", flex),
                joints::joint("tentacle_back",        "body",   "tentacle_back_left",   flex),
            ],
        },
        skin: SkinLayer {
            covering: SkinCovering::Membrane,
            integrity: 1.0,
            thickness: cfg::skin::THICKNESS_NYXAL,
            hydration: 1.0,
        },
    }
}
