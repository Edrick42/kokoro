//! Anatomy system — skeleton, muscles, joints, skin.
//!
//! Four interconnected layers that work like real biology:
//!
//! 1. **Skeleton** — structural frame. Bone density and integrity determine
//!    the creature's maximum health ceiling. Species archetype matters:
//!    Pylum has hollow bones (light, fragile), Skael has dense bones (tough),
//!    Nyxal has NO bones (hydrostatic pressure instead).
//!
//! 2. **Joints** — connect bones. Flexibility and lubrication affect movement.
//!    Stiff joints (low lubrication) cost extra energy. Very stiff joints
//!    prevent the creature from being playful.
//!
//! 3. **Muscles** — provide force. Mass and condition determine efficiency.
//!    Fatigued muscles drain energy faster. Muscles recover during sleep
//!    and atrophy when malnourished.
//!
//! 4. **Skin** — protection and sensation. Integrity and hydration affect
//!    pain sensitivity (touch system). Species-specific covering: fur,
//!    plumage, scales, or membrane.
//!
//! ## Cascade
//!
//! Damage flows through layers: broken bone → connected joints lock →
//! connected muscles weaken → energy costs rise → creature suffers.
//! Good care reverses the cycle gradually.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::anatomy as cfg;
use crate::genome::{Genome, Species};
use crate::mind::{Mind, MoodState};

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct AnatomyPlugin;

impl Plugin for AnatomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, anatomy_tick_system);
    }
}

// ---------------------------------------------------------------------------
// Data Model
// ---------------------------------------------------------------------------

/// Complete anatomy state for one creature.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct AnatomyState {
    pub skeleton: Skeleton,
    pub muscles: MuscleSystem,
    pub joints: JointSystem,
    pub skin: SkinLayer,
}

// --- Skeleton ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skeleton {
    pub structure_type: SkeletonType,
    pub bone_density: f32,
    pub bone_health: f32,
    /// Nyxal only: replaces bone density for boneless creatures.
    #[serde(default)]
    pub hydrostatic_pressure: f32,
    pub bones: Vec<Bone>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkeletonType {
    Standard,
    Hollow,
    Dense,
    Hydrostatic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bone {
    pub name: String,
    pub slot: String,
    pub integrity: f32,
    pub density_modifier: f32,
}

// --- Muscles ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscleSystem {
    pub mass: f32,
    pub condition: f32,
    pub tone: f32,
    pub groups: Vec<MuscleGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscleGroup {
    pub name: String,
    pub joint: String,
    pub strength: f32,
    pub fatigue: f32,
}

// --- Joints ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointSystem {
    pub joints: Vec<Joint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    pub name: String,
    pub bone_a: String,
    pub bone_b: String,
    pub flexibility: f32,
    pub lubrication: f32,
    pub integrity: f32,
}

// --- Skin ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinLayer {
    pub covering: SkinCovering,
    pub integrity: f32,
    pub thickness: f32,
    pub hydration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkinCovering {
    Fur,
    Plumage,
    Scales,
    Membrane,
}

// ---------------------------------------------------------------------------
// Species initializers
// ---------------------------------------------------------------------------

impl AnatomyState {
    /// Creates the anatomy for a given species, using the genome for individual variation.
    pub fn new_for(species: &Species, genome: &Genome) -> Self {
        match species {
            Species::Moluun => Self::moluun(genome),
            Species::Pylum  => Self::pylum(genome),
            Species::Skael  => Self::skael(genome),
            Species::Nyxal  => Self::nyxal(genome),
        }
    }

    fn moluun(genome: &Genome) -> Self {
        let density = cfg::skeleton::DENSITY_MOLUUN * (0.9 + genome.resilience * 0.2);
        let mass = cfg::muscles::MASS_MOLUUN * (1.1 - genome.appetite * 0.2);
        let flex = cfg::joints::FLEX_MOLUUN * (0.95 + genome.curiosity * 0.1);

        Self {
            skeleton: Skeleton {
                structure_type: SkeletonType::Standard,
                bone_density: density,
                bone_health: 1.0,
                hydrostatic_pressure: 0.0,
                bones: vec![
                    bone("skull",     "body",       1.2),
                    bone("spine",     "body",       1.0),
                    bone("ribcage",   "body",       1.1),
                    bone("pelvis",    "body",       1.0),
                    bone("limb_left", "body",       0.8),
                    bone("limb_right","body",       0.8),
                ],
            },
            muscles: MuscleSystem {
                mass,
                condition: 1.0,
                tone: mass,
                groups: vec![
                    muscle("core",       "neck"),
                    muscle("upper",      "shoulder_left"),
                    muscle("legs_left",  "hip_left"),
                    muscle("legs_right", "hip_right"),
                ],
            },
            joints: JointSystem {
                joints: vec![
                    joint("neck",           "skull",     "spine",     flex),
                    joint("shoulder_left",  "spine",     "limb_left", flex),
                    joint("shoulder_right", "spine",     "limb_right",flex),
                    joint("hip_left",       "pelvis",    "limb_left", flex),
                    joint("hip_right",      "pelvis",    "limb_right",flex),
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

    fn pylum(genome: &Genome) -> Self {
        let density = cfg::skeleton::DENSITY_PYLUM * (0.9 + genome.resilience * 0.2);
        let mass = cfg::muscles::MASS_PYLUM * (1.1 - genome.appetite * 0.2);
        let flex = cfg::joints::FLEX_PYLUM * (0.95 + genome.curiosity * 0.1);

        Self {
            skeleton: Skeleton {
                structure_type: SkeletonType::Hollow,
                bone_density: density,
                bone_health: 1.0,
                hydrostatic_pressure: 0.0,
                bones: vec![
                    bone("skull",           "body",       1.1),
                    bone("spine",           "body",       1.0),
                    bone("ribcage",         "body",       1.0),
                    bone("keel",            "body",       1.2),
                    bone("wing_bone_left",  "wing_left",  0.7),
                    bone("wing_bone_right", "wing_right", 0.7),
                    bone("tail_bone",       "tail",       0.6),
                ],
            },
            muscles: MuscleSystem {
                mass,
                condition: 1.0,
                tone: mass,
                groups: vec![
                    muscle("pectorals",    "wing_left"),
                    muscle("core",         "neck"),
                    muscle("tail_muscles", "tail"),
                    muscle("legs",         "keel_joint"),
                ],
            },
            joints: JointSystem {
                joints: vec![
                    joint("neck",       "skull",           "spine",           flex),
                    joint("wing_left",  "spine",           "wing_bone_left",  flex),
                    joint("wing_right", "spine",           "wing_bone_right", flex),
                    joint("keel_joint", "ribcage",         "keel",            flex * 0.8),
                    joint("tail",       "spine",           "tail_bone",       flex),
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

    fn skael(genome: &Genome) -> Self {
        let density = cfg::skeleton::DENSITY_SKAEL * (0.9 + genome.resilience * 0.2);
        let mass = cfg::muscles::MASS_SKAEL * (1.1 - genome.appetite * 0.2);
        let flex = cfg::joints::FLEX_SKAEL * (0.95 + genome.curiosity * 0.1);

        Self {
            skeleton: Skeleton {
                structure_type: SkeletonType::Dense,
                bone_density: density,
                bone_health: 1.0,
                hydrostatic_pressure: 0.0,
                bones: vec![
                    bone("skull",      "body",  1.3),
                    bone("spine",      "body",  1.0),
                    bone("ribcage",    "body",  1.2),
                    bone("pelvis",     "body",  1.1),
                    bone("limb_left",  "body",  0.9),
                    bone("limb_right", "body",  0.9),
                    bone("tail_bone",  "tail",  0.8),
                ],
            },
            muscles: MuscleSystem {
                mass,
                condition: 1.0,
                tone: mass,
                groups: vec![
                    muscle("core",         "neck"),
                    muscle("upper",        "shoulder_left"),
                    muscle("legs_left",    "hip_left"),
                    muscle("legs_right",   "hip_right"),
                    muscle("tail_muscles", "tail"),
                ],
            },
            joints: JointSystem {
                joints: vec![
                    joint("neck",           "skull",   "spine",      flex),
                    joint("shoulder_left",  "spine",   "limb_left",  flex),
                    joint("shoulder_right", "spine",   "limb_right", flex),
                    joint("hip_left",       "pelvis",  "limb_left",  flex),
                    joint("hip_right",      "pelvis",  "limb_right", flex),
                    joint("tail",           "pelvis",  "tail_bone",  flex * 1.2),
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

    fn nyxal(genome: &Genome) -> Self {
        let mass = cfg::muscles::MASS_NYXAL * (1.1 - genome.appetite * 0.2);
        let flex = cfg::joints::FLEX_NYXAL * (0.95 + genome.curiosity * 0.1);
        let pressure = cfg::skeleton::HYDROSTATIC_PRESSURE_NYXAL * (0.9 + genome.resilience * 0.2);

        Self {
            skeleton: Skeleton {
                structure_type: SkeletonType::Hydrostatic,
                bone_density: 0.0,
                bone_health: 1.0,
                hydrostatic_pressure: pressure,
                bones: vec![], // No bones!
            },
            muscles: MuscleSystem {
                mass,
                condition: 1.0,
                tone: mass,
                groups: vec![
                    muscle("mantle_muscles",   "mantle_base"),
                    muscle("tentacle_muscles", "tentacle_front_left"),
                    muscle("siphon",           "mantle_base"),
                ],
            },
            joints: JointSystem {
                joints: vec![
                    joint("mantle_base",          "mantle", "body",                    flex),
                    joint("tentacle_front_left",  "body",   "tentacle_front_left",     flex),
                    joint("tentacle_front_right", "body",   "tentacle_front_right",    flex),
                    joint("tentacle_back",        "body",   "tentacle_back_left",      flex),
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
}

// ---------------------------------------------------------------------------
// Helpers — reduce boilerplate in species initializers
// ---------------------------------------------------------------------------

fn bone(name: &str, slot: &str, density_modifier: f32) -> Bone {
    Bone {
        name: name.to_string(),
        slot: slot.to_string(),
        integrity: 1.0,
        density_modifier,
    }
}

fn muscle(name: &str, joint: &str) -> MuscleGroup {
    MuscleGroup {
        name: name.to_string(),
        joint: joint.to_string(),
        strength: 1.0,
        fatigue: 0.0,
    }
}

fn joint(name: &str, bone_a: &str, bone_b: &str, flexibility: f32) -> Joint {
    Joint {
        name: name.to_string(),
        bone_a: bone_a.to_string(),
        bone_b: bone_b.to_string(),
        flexibility: flexibility.clamp(0.0, 1.0),
        lubrication: 1.0,
        integrity: 1.0,
    }
}

// ---------------------------------------------------------------------------
// Anatomy helpers — averages and aggregate stats
// ---------------------------------------------------------------------------

impl AnatomyState {
    /// Average bone integrity across all bones (1.0 for Nyxal with no bones).
    pub fn avg_bone_integrity(&self) -> f32 {
        if self.skeleton.bones.is_empty() {
            return self.skeleton.hydrostatic_pressure;
        }
        let sum: f32 = self.skeleton.bones.iter().map(|b| b.integrity).sum();
        sum / self.skeleton.bones.len() as f32
    }

    /// Maximum health ceiling based on skeletal health.
    pub fn health_ceiling(&self) -> f32 {
        let structural = if self.skeleton.structure_type == SkeletonType::Hydrostatic {
            self.skeleton.hydrostatic_pressure
        } else {
            self.skeleton.bone_health * self.avg_bone_integrity()
        };
        structural * cfg::skeleton::HEALTH_CEILING_MULTIPLIER
    }

    /// Average joint lubrication.
    pub fn avg_lubrication(&self) -> f32 {
        if self.joints.joints.is_empty() { return 1.0; }
        let sum: f32 = self.joints.joints.iter().map(|j| j.lubrication).sum();
        sum / self.joints.joints.len() as f32
    }

    /// Average joint flexibility.
    pub fn avg_flexibility(&self) -> f32 {
        if self.joints.joints.is_empty() { return 1.0; }
        let sum: f32 = self.joints.joints.iter().map(|j| j.flexibility).sum();
        sum / self.joints.joints.len() as f32
    }

    /// Average muscle fatigue.
    #[allow(dead_code)]
    pub fn avg_fatigue(&self) -> f32 {
        if self.muscles.groups.is_empty() { return 0.0; }
        let sum: f32 = self.muscles.groups.iter().map(|m| m.fatigue).sum();
        sum / self.muscles.groups.len() as f32
    }
}

// ---------------------------------------------------------------------------
// Tick system
// ---------------------------------------------------------------------------

fn anatomy_tick_system(
    mut anatomy: ResMut<AnatomyState>,
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
) {
    // Only run on game ticks (same cadence as mind.update_mood)
    // The tick timer fires mind.is_changed(), so we piggyback on that.
    if !mind.is_changed() {
        return;
    }

    let is_sick = mind.mood == MoodState::Sick;
    let is_sleeping = mind.mood == MoodState::Sleeping;
    let hunger = mind.stats.hunger;

    // --- 1. Skeleton maintenance ---
    update_skeleton(&mut anatomy.skeleton, is_sick, hunger);

    // --- 2. Muscle maintenance ---
    update_muscles(&mut anatomy.muscles, is_sick, is_sleeping, hunger);

    // --- 3. Joint maintenance ---
    let is_elder = mind.age_ticks > 8_500_000;
    update_joints(&mut anatomy.joints, hunger, is_elder);

    // --- 4. Skin maintenance ---
    update_skin(&mut anatomy.skin, is_sick, hunger);

    // --- 5. Apply anatomy effects to vital stats ---
    apply_health_ceiling(&anatomy, &mut mind);
    apply_energy_penalties(&anatomy, &mut mind);
    apply_mood_overrides(&anatomy, &mut mind, &genome);

    // --- 6. Damage cascade check ---
    check_bone_breaks(&mut anatomy, &mut mind);
}

fn update_skeleton(skeleton: &mut Skeleton, is_sick: bool, hunger: f32) {
    if skeleton.structure_type == SkeletonType::Hydrostatic {
        // Nyxal: hydrostatic pressure replaces bone health
        if is_sick || hunger > cfg::HUNGER_DECAY_THRESHOLD {
            skeleton.hydrostatic_pressure =
                (skeleton.hydrostatic_pressure - cfg::skeleton::HYDROSTATIC_DECAY).max(0.0);
        } else if hunger < cfg::HUNGER_REPAIR_THRESHOLD {
            skeleton.hydrostatic_pressure =
                (skeleton.hydrostatic_pressure + cfg::skeleton::HYDROSTATIC_REPAIR).min(1.0);
        }
        return;
    }

    let decay_mult = match skeleton.structure_type {
        SkeletonType::Hollow => cfg::skeleton::HOLLOW_DAMAGE_MULTIPLIER,
        SkeletonType::Dense  => cfg::skeleton::DENSE_RESISTANCE_MULTIPLIER,
        _ => 1.0,
    };

    if is_sick || hunger > cfg::HUNGER_DECAY_THRESHOLD {
        skeleton.bone_health =
            (skeleton.bone_health - cfg::skeleton::BONE_HEALTH_DECAY * decay_mult).max(0.0);
        for bone in &mut skeleton.bones {
            bone.integrity =
                (bone.integrity - cfg::skeleton::BONE_INTEGRITY_DECAY * decay_mult).max(0.0);
        }
    } else if hunger < cfg::HUNGER_REPAIR_THRESHOLD {
        skeleton.bone_health =
            (skeleton.bone_health + cfg::skeleton::BONE_HEALTH_REPAIR).min(1.0);
        for bone in &mut skeleton.bones {
            bone.integrity =
                (bone.integrity + cfg::skeleton::BONE_INTEGRITY_REPAIR).min(1.0);
        }
        // Dense bones get a small repair bonus
        if skeleton.structure_type == SkeletonType::Dense {
            skeleton.bone_health =
                (skeleton.bone_health + cfg::skeleton::DENSE_REPAIR_BONUS).min(1.0);
        }
    }
}

fn update_muscles(muscles: &mut MuscleSystem, is_sick: bool, is_sleeping: bool, hunger: f32) {
    if is_sleeping {
        // Sleep recovers fatigue and rebuilds condition
        for group in &mut muscles.groups {
            group.fatigue = (group.fatigue - cfg::muscles::FATIGUE_RECOVERY).max(0.0);
        }
        muscles.condition = (muscles.condition + cfg::muscles::CONDITION_REPAIR).min(1.0);
    } else {
        // Awake: fatigue accumulates
        for group in &mut muscles.groups {
            group.fatigue = (group.fatigue + cfg::muscles::FATIGUE_ACCUMULATION).min(1.0);
        }
    }

    // Condition degrades when malnourished or sick
    if is_sick || hunger > cfg::HUNGER_MUSCLE_THRESHOLD {
        muscles.condition = (muscles.condition - cfg::muscles::CONDITION_DECAY).max(0.0);
    } else if hunger < cfg::HUNGER_MUSCLE_REPAIR {
        muscles.condition = (muscles.condition + cfg::muscles::CONDITION_REPAIR).min(1.0);
    }

    // Tone slowly converges toward condition
    muscles.tone += (muscles.condition - muscles.tone) * cfg::muscles::TONE_CONVERGENCE;
}

fn update_joints(joints: &mut JointSystem, hunger: f32, is_elder: bool) {
    for joint in &mut joints.joints {
        // Lubrication
        if hunger > cfg::HUNGER_JOINT_THRESHOLD {
            joint.lubrication =
                (joint.lubrication - cfg::joints::LUBRICATION_DECAY).max(0.0);
        } else if hunger < cfg::HUNGER_REPAIR_THRESHOLD {
            joint.lubrication =
                (joint.lubrication + cfg::joints::LUBRICATION_REPAIR).min(1.0);
        }

        // Elders lose flexibility irreversibly
        if is_elder {
            joint.flexibility =
                (joint.flexibility - cfg::joints::ELDER_FLEXIBILITY_DECAY).max(0.0);
        }
    }
}

fn update_skin(skin: &mut SkinLayer, is_sick: bool, hunger: f32) {
    if hunger > cfg::HUNGER_SKIN_THRESHOLD {
        skin.hydration = (skin.hydration - cfg::skin::HYDRATION_DECAY).max(0.0);
    } else if hunger < cfg::HUNGER_REPAIR_THRESHOLD {
        skin.hydration = (skin.hydration + cfg::skin::HYDRATION_REPAIR).min(1.0);
    }

    if is_sick {
        skin.integrity = (skin.integrity - cfg::skin::INTEGRITY_DECAY).max(0.0);
    } else {
        skin.integrity = (skin.integrity + cfg::skin::INTEGRITY_REPAIR).min(1.0);
    }
}

fn apply_health_ceiling(anatomy: &AnatomyState, mind: &mut Mind) {
    let ceiling = anatomy.health_ceiling();
    if mind.stats.health > ceiling {
        mind.stats.health = ceiling;
    }
}

fn apply_energy_penalties(anatomy: &AnatomyState, mind: &mut Mind) {
    // Weak muscles cost extra energy
    if anatomy.muscles.condition < cfg::muscles::LOW_CONDITION_THRESHOLD {
        let penalty = (cfg::muscles::LOW_CONDITION_THRESHOLD - anatomy.muscles.condition)
            * cfg::muscles::ENERGY_PENALTY_FACTOR;
        mind.stats.energy = (mind.stats.energy - penalty).max(0.0);
    }

    // Stiff joints cost extra energy
    if anatomy.avg_lubrication() < cfg::joints::STIFFNESS_THRESHOLD {
        mind.stats.energy =
            (mind.stats.energy - cfg::joints::STIFFNESS_ENERGY_PENALTY).max(0.0);
    }
}

fn apply_mood_overrides(anatomy: &AnatomyState, mind: &mut Mind, genome: &Genome) {
    let avg_flex = anatomy.avg_flexibility();

    // Too stiff to play — override Playful back to Happy
    if mind.mood == MoodState::Playful && avg_flex < cfg::joints::PLAYFUL_FLEX_BLOCK {
        mind.mood = MoodState::Happy;
    }

    // Flexible + curious creatures have lower playfulness threshold
    if mind.mood == MoodState::Happy
        && avg_flex > 0.7
        && genome.curiosity > 0.4
        && mind.stats.happiness > 70.0
        && mind.stats.energy > 50.0
    {
        mind.mood = MoodState::Playful;
    }
}

fn check_bone_breaks(anatomy: &mut AnatomyState, mind: &mut Mind) {
    // Find any newly broken bones
    let broken_bone_names: Vec<String> = anatomy.skeleton.bones.iter()
        .filter(|b| b.integrity <= 0.0)
        .map(|b| b.name.clone())
        .collect();

    if broken_bone_names.is_empty() {
        return;
    }

    for broken_name in &broken_bone_names {
        // Lock connected joints
        for joint in &mut anatomy.joints.joints {
            if joint.bone_a == *broken_name || joint.bone_b == *broken_name {
                joint.flexibility = joint.flexibility.min(cfg::skeleton::BREAK_JOINT_FLEX_MIN);
                joint.integrity = joint.integrity.min(cfg::skeleton::BREAK_JOINT_INTEGRITY_CAP);

                // Weaken muscles that actuate this joint
                let joint_name = joint.name.clone();
                for group in &mut anatomy.muscles.groups {
                    if group.joint == joint_name {
                        group.strength *= cfg::skeleton::BREAK_MUSCLE_STRENGTH_FACTOR;
                    }
                }
            }
        }

        // Health penalty
        mind.stats.health =
            (mind.stats.health - cfg::skeleton::BREAK_HEALTH_PENALTY).max(0.0);
    }

    // Force Sick mood (critical override)
    if !mind.mood.is_critical() {
        mind.mood = MoodState::Sick;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_genome(species: Species) -> Genome {
        Genome::random_for(species)
    }

    #[test]
    fn moluun_anatomy_counts() {
        let g = test_genome(Species::Moluun);
        let a = AnatomyState::new_for(&Species::Moluun, &g);
        assert_eq!(a.skeleton.bones.len(), 6);
        assert_eq!(a.joints.joints.len(), 5);
        assert_eq!(a.muscles.groups.len(), 4);
        assert_eq!(a.skeleton.structure_type, SkeletonType::Standard);
        assert_eq!(a.skin.covering, SkinCovering::Fur);
    }

    #[test]
    fn pylum_anatomy_counts() {
        let g = test_genome(Species::Pylum);
        let a = AnatomyState::new_for(&Species::Pylum, &g);
        assert_eq!(a.skeleton.bones.len(), 7);
        assert_eq!(a.joints.joints.len(), 5);
        assert_eq!(a.muscles.groups.len(), 4);
        assert_eq!(a.skeleton.structure_type, SkeletonType::Hollow);
        assert_eq!(a.skin.covering, SkinCovering::Plumage);
    }

    #[test]
    fn skael_anatomy_counts() {
        let g = test_genome(Species::Skael);
        let a = AnatomyState::new_for(&Species::Skael, &g);
        assert_eq!(a.skeleton.bones.len(), 7);
        assert_eq!(a.joints.joints.len(), 6);
        assert_eq!(a.muscles.groups.len(), 5);
        assert_eq!(a.skeleton.structure_type, SkeletonType::Dense);
        assert_eq!(a.skin.covering, SkinCovering::Scales);
    }

    #[test]
    fn nyxal_has_no_bones() {
        let g = test_genome(Species::Nyxal);
        let a = AnatomyState::new_for(&Species::Nyxal, &g);
        assert_eq!(a.skeleton.bones.len(), 0);
        assert_eq!(a.skeleton.structure_type, SkeletonType::Hydrostatic);
        assert!(a.skeleton.hydrostatic_pressure > 0.0);
        assert_eq!(a.skin.covering, SkinCovering::Membrane);
    }

    #[test]
    fn resilience_increases_bone_density() {
        let mut g_low = Genome::random_for(Species::Moluun);
        g_low.resilience = 0.0;
        let mut g_high = Genome::random_for(Species::Moluun);
        g_high.resilience = 1.0;

        let a_low = AnatomyState::new_for(&Species::Moluun, &g_low);
        let a_high = AnatomyState::new_for(&Species::Moluun, &g_high);

        assert!(a_high.skeleton.bone_density > a_low.skeleton.bone_density,
            "high resilience should produce higher bone density");
    }

    #[test]
    fn health_ceiling_based_on_bones() {
        let g = test_genome(Species::Moluun);
        let mut a = AnatomyState::new_for(&Species::Moluun, &g);

        // Full health: ceiling should be ~100
        let ceiling_full = a.health_ceiling();
        assert!(ceiling_full > 90.0, "full bone health ceiling = {}", ceiling_full);

        // Half bone health: ceiling should be ~50
        a.skeleton.bone_health = 0.5;
        let ceiling_half = a.health_ceiling();
        assert!(ceiling_half < 60.0, "half bone health ceiling = {}", ceiling_half);
    }

    #[test]
    fn nyxal_health_ceiling_based_on_pressure() {
        let g = test_genome(Species::Nyxal);
        let mut a = AnatomyState::new_for(&Species::Nyxal, &g);

        let ceiling_full = a.health_ceiling();
        assert!(ceiling_full > 70.0);

        a.skeleton.hydrostatic_pressure = 0.3;
        let ceiling_low = a.health_ceiling();
        assert!(ceiling_low < 40.0, "low pressure ceiling = {}", ceiling_low);
    }

    #[test]
    fn bone_break_cascade() {
        let g = test_genome(Species::Moluun);
        let mut a = AnatomyState::new_for(&Species::Moluun, &g);
        let mut mind = Mind::new();

        // Break the spine
        if let Some(spine) = a.skeleton.bones.iter_mut().find(|b| b.name == "spine") {
            spine.integrity = 0.0;
        }

        check_bone_breaks(&mut a, &mut mind);

        // Neck joint (skull-spine) should be locked
        let neck = a.joints.joints.iter().find(|j| j.name == "neck").unwrap();
        assert!(neck.flexibility <= cfg::skeleton::BREAK_JOINT_FLEX_MIN + 0.01);

        // Core muscle (neck joint) should be weakened
        let core = a.muscles.groups.iter().find(|m| m.name == "core").unwrap();
        assert!(core.strength < 1.0);

        // Mood should be Sick
        assert_eq!(mind.mood, MoodState::Sick);

        // Health should have dropped
        assert!(mind.stats.health < 100.0);
    }

    #[test]
    fn all_species_start_healthy() {
        let species = [Species::Moluun, Species::Pylum, Species::Skael, Species::Nyxal];
        for sp in &species {
            let g = Genome::random_for(sp.clone());
            let a = AnatomyState::new_for(sp, &g);
            assert_eq!(a.skeleton.bone_health, 1.0);
            assert_eq!(a.muscles.condition, 1.0);
            assert_eq!(a.skin.integrity, 1.0);
            assert_eq!(a.skin.hydration, 1.0);
            for bone in &a.skeleton.bones {
                assert_eq!(bone.integrity, 1.0);
            }
            for joint in &a.joints.joints {
                assert_eq!(joint.lubrication, 1.0);
                assert_eq!(joint.integrity, 1.0);
            }
            for group in &a.muscles.groups {
                assert_eq!(group.strength, 1.0);
                assert_eq!(group.fatigue, 0.0);
            }
        }
    }

    #[test]
    fn avg_helpers_work() {
        let g = test_genome(Species::Moluun);
        let a = AnatomyState::new_for(&Species::Moluun, &g);
        assert!((a.avg_bone_integrity() - 1.0).abs() < 0.01);
        assert!((a.avg_lubrication() - 1.0).abs() < 0.01);
        assert!(a.avg_flexibility() > 0.0 && a.avg_flexibility() <= 1.0);
        assert!((a.avg_fatigue() - 0.0).abs() < 0.01);
    }
}
