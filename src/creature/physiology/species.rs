//! Species-specific physiology initializers.
//!
//! Each species has a unique skeleton, muscle layout, joint configuration,
//! and skin type. Genes modify the base values to produce individual variation.

use crate::config::anatomy as cfg;
use crate::genome::{Genome, Species};

use super::bone_health::{self, BoneHealth, BoneStructure};
use super::muscle_health::{self, MuscleHealth};
use super::joint_health::{self, JointHealth};
use super::skin_health::{SkinHealth, SkinCovering};
use super::{PhysiologyState, FatReserve};

impl PhysiologyState {
    /// Creates the physiology for a given species, using the genome for individual variation.
    pub fn new_for(species: &Species, genome: &Genome) -> Self {
        match species {
            Species::Moluun => moluun(genome),
            Species::Pylum  => pylum(genome),
            Species::Skael  => skael(genome),
            Species::Nyxal  => nyxal(genome),
        }
    }
}

fn moluun(genome: &Genome) -> PhysiologyState {
    let density = cfg::skeleton::DENSITY_MOLUUN * (0.9 + genome.resilience * 0.2);
    let mass = cfg::muscles::MASS_MOLUUN * (1.1 - genome.appetite * 0.2);
    let flex = cfg::joints::FLEX_MOLUUN * (0.95 + genome.curiosity * 0.1);

    PhysiologyState {
        bone_health: BoneHealth {
            structure: BoneStructure::Standard,
            density,
            overall: 1.0,
            hydrostatic_pressure: 0.0,
            bones: vec![
                bone_health::bone_record("skull",      "body", 1.2),
                bone_health::bone_record("spine",      "body", 1.0),
                bone_health::bone_record("ribcage",    "body", 1.1),
                bone_health::bone_record("pelvis",     "body", 1.0),
                bone_health::bone_record("limb_left",  "body", 0.8),
                bone_health::bone_record("limb_right", "body", 0.8),
            ],
        },
        muscle_health: MuscleHealth {
            mass,
            condition: 1.0,
            tone: mass,
            groups: vec![
                muscle_health::muscle_record("core",       "neck"),
                muscle_health::muscle_record("upper",      "shoulder_left"),
                muscle_health::muscle_record("legs_left",  "hip_left"),
                muscle_health::muscle_record("legs_right", "hip_right"),
            ],
        },
        joint_health: JointHealth {
            joints: vec![
                joint_health::joint_record("neck",           "skull",  "spine",      flex),
                joint_health::joint_record("shoulder_left",  "spine",  "limb_left",  flex),
                joint_health::joint_record("shoulder_right", "spine",  "limb_right", flex),
                joint_health::joint_record("hip_left",       "pelvis", "limb_left",  flex),
                joint_health::joint_record("hip_right",      "pelvis", "limb_right", flex),
            ],
        },
        skin_health: SkinHealth {
            covering: SkinCovering::Fur,
            integrity: 1.0,
            thickness: cfg::skin::THICKNESS_MOLUUN,
            hydration: 1.0,
        },
        fat: FatReserve {
            level: cfg::fat::LEVEL_MOLUUN,
            burn_rate: cfg::fat::BURN_RATE,
            store_rate: cfg::fat::STORE_RATE,
            insulation: cfg::fat::LEVEL_MOLUUN * cfg::fat::INSULATION_FACTOR,
        },
    }
}

fn pylum(genome: &Genome) -> PhysiologyState {
    let density = cfg::skeleton::DENSITY_PYLUM * (0.9 + genome.resilience * 0.2);
    let mass = cfg::muscles::MASS_PYLUM * (1.1 - genome.appetite * 0.2);
    let flex = cfg::joints::FLEX_PYLUM * (0.95 + genome.curiosity * 0.1);

    PhysiologyState {
        bone_health: BoneHealth {
            structure: BoneStructure::Hollow,
            density,
            overall: 1.0,
            hydrostatic_pressure: 0.0,
            bones: vec![
                bone_health::bone_record("skull",           "body",       1.1),
                bone_health::bone_record("spine",           "body",       1.0),
                bone_health::bone_record("ribcage",         "body",       1.0),
                bone_health::bone_record("keel",            "body",       1.2),
                bone_health::bone_record("wing_bone_left",  "wing_left",  0.7),
                bone_health::bone_record("wing_bone_right", "wing_right", 0.7),
                bone_health::bone_record("tail_bone",       "tail",       0.6),
            ],
        },
        muscle_health: MuscleHealth {
            mass,
            condition: 1.0,
            tone: mass,
            groups: vec![
                muscle_health::muscle_record("pectorals",    "wing_left"),
                muscle_health::muscle_record("core",         "neck"),
                muscle_health::muscle_record("tail_muscles", "tail"),
                muscle_health::muscle_record("legs",         "keel_joint"),
            ],
        },
        joint_health: JointHealth {
            joints: vec![
                joint_health::joint_record("neck",       "skull",   "spine",          flex),
                joint_health::joint_record("wing_left",  "spine",   "wing_bone_left", flex),
                joint_health::joint_record("wing_right", "spine",   "wing_bone_right",flex),
                joint_health::joint_record("keel_joint", "ribcage", "keel",           flex * 0.8),
                joint_health::joint_record("tail",       "spine",   "tail_bone",      flex),
            ],
        },
        skin_health: SkinHealth {
            covering: SkinCovering::Plumage,
            integrity: 1.0,
            thickness: cfg::skin::THICKNESS_PYLUM,
            hydration: 1.0,
        },
        fat: FatReserve {
            level: cfg::fat::LEVEL_PYLUM,
            burn_rate: cfg::fat::BURN_RATE,
            store_rate: cfg::fat::STORE_RATE,
            insulation: cfg::fat::LEVEL_PYLUM * cfg::fat::INSULATION_FACTOR,
        },
    }
}

fn skael(genome: &Genome) -> PhysiologyState {
    let density = cfg::skeleton::DENSITY_SKAEL * (0.9 + genome.resilience * 0.2);
    let mass = cfg::muscles::MASS_SKAEL * (1.1 - genome.appetite * 0.2);
    let flex = cfg::joints::FLEX_SKAEL * (0.95 + genome.curiosity * 0.1);

    PhysiologyState {
        bone_health: BoneHealth {
            structure: BoneStructure::Dense,
            density,
            overall: 1.0,
            hydrostatic_pressure: 0.0,
            bones: vec![
                bone_health::bone_record("skull",      "body", 1.3),
                bone_health::bone_record("spine",      "body", 1.0),
                bone_health::bone_record("ribcage",    "body", 1.2),
                bone_health::bone_record("pelvis",     "body", 1.1),
                bone_health::bone_record("limb_left",  "body", 0.9),
                bone_health::bone_record("limb_right", "body", 0.9),
                bone_health::bone_record("tail_bone",  "tail", 0.8),
            ],
        },
        muscle_health: MuscleHealth {
            mass,
            condition: 1.0,
            tone: mass,
            groups: vec![
                muscle_health::muscle_record("core",         "neck"),
                muscle_health::muscle_record("upper",        "shoulder_left"),
                muscle_health::muscle_record("legs_left",    "hip_left"),
                muscle_health::muscle_record("legs_right",   "hip_right"),
                muscle_health::muscle_record("tail_muscles", "tail"),
            ],
        },
        joint_health: JointHealth {
            joints: vec![
                joint_health::joint_record("neck",           "skull",  "spine",      flex),
                joint_health::joint_record("shoulder_left",  "spine",  "limb_left",  flex),
                joint_health::joint_record("shoulder_right", "spine",  "limb_right", flex),
                joint_health::joint_record("hip_left",       "pelvis", "limb_left",  flex),
                joint_health::joint_record("hip_right",      "pelvis", "limb_right", flex),
                joint_health::joint_record("tail",           "pelvis", "tail_bone",  flex * 1.2),
            ],
        },
        skin_health: SkinHealth {
            covering: SkinCovering::Scales,
            integrity: 1.0,
            thickness: cfg::skin::THICKNESS_SKAEL,
            hydration: 1.0,
        },
        fat: FatReserve {
            level: cfg::fat::LEVEL_SKAEL,
            burn_rate: cfg::fat::BURN_RATE,
            store_rate: cfg::fat::STORE_RATE,
            insulation: cfg::fat::LEVEL_SKAEL * cfg::fat::INSULATION_FACTOR,
        },
    }
}

fn nyxal(genome: &Genome) -> PhysiologyState {
    let mass = cfg::muscles::MASS_NYXAL * (1.1 - genome.appetite * 0.2);
    let flex = cfg::joints::FLEX_NYXAL * (0.95 + genome.curiosity * 0.1);
    let pressure = cfg::skeleton::HYDROSTATIC_PRESSURE_NYXAL * (0.9 + genome.resilience * 0.2);

    PhysiologyState {
        bone_health: BoneHealth {
            structure: BoneStructure::Hydrostatic,
            density: 0.0,
            overall: 1.0,
            hydrostatic_pressure: pressure,
            bones: vec![],
        },
        muscle_health: MuscleHealth {
            mass,
            condition: 1.0,
            tone: mass,
            groups: vec![
                muscle_health::muscle_record("mantle_muscles",   "mantle_base"),
                muscle_health::muscle_record("tentacle_muscles", "tentacle_front_left"),
                muscle_health::muscle_record("siphon",           "mantle_base"),
            ],
        },
        joint_health: JointHealth {
            joints: vec![
                joint_health::joint_record("mantle_base",          "mantle", "body",                 flex),
                joint_health::joint_record("tentacle_front_left",  "body",   "tentacle_front_left",  flex),
                joint_health::joint_record("tentacle_front_right", "body",   "tentacle_front_right", flex),
                joint_health::joint_record("tentacle_back",        "body",   "tentacle_back_left",   flex),
            ],
        },
        skin_health: SkinHealth {
            covering: SkinCovering::Membrane,
            integrity: 1.0,
            thickness: cfg::skin::THICKNESS_NYXAL,
            hydration: 1.0,
        },
        fat: FatReserve {
            level: cfg::fat::LEVEL_NYXAL,
            burn_rate: cfg::fat::BURN_RATE,
            store_rate: cfg::fat::STORE_RATE,
            insulation: cfg::fat::LEVEL_NYXAL * cfg::fat::INSULATION_FACTOR,
        },
    }
}
