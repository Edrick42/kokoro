//! Anatomy tick system — per-tick maintenance, stat integration, and damage cascade.
//!
//! Runs after `mind.update_mood()` each game tick. Handles:
//! 1. Skeleton maintenance (bone health, hydrostatic pressure)
//! 2. Muscle maintenance (fatigue, condition, tone)
//! 3. Joint maintenance (lubrication, elder stiffening)
//! 4. Skin maintenance (hydration, integrity)
//! 5. Stat effects (health ceiling, energy penalties, mood overrides)
//! 6. Damage cascade (bone break → joint lock → muscle weakness)

use bevy::prelude::*;

use crate::config::anatomy as cfg;
use crate::genome::Genome;
use crate::mind::{Mind, MoodState};

use super::AnatomyState;
use super::skeleton::SkeletonType;

/// Bevy system: updates anatomy each game tick and cascades into vital stats.
pub fn anatomy_tick_system(
    mut anatomy: ResMut<AnatomyState>,
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
) {
    if !mind.is_changed() {
        return;
    }

    let is_sick = mind.mood == MoodState::Sick;
    let is_sleeping = mind.mood == MoodState::Sleeping;
    let hunger = mind.stats.hunger;

    update_skeleton(&mut anatomy, is_sick, hunger);
    update_muscles(&mut anatomy, is_sick, is_sleeping, hunger);

    let is_elder = mind.age_ticks > 8_500_000;
    update_joints(&mut anatomy, hunger, is_elder);
    update_skin(&mut anatomy, is_sick, hunger);

    apply_health_ceiling(&anatomy, &mut mind);
    apply_energy_penalties(&anatomy, &mut mind);
    apply_mood_overrides(&anatomy, &mut mind, &genome);
    check_bone_breaks(&mut anatomy, &mut mind);
}

fn update_skeleton(anatomy: &mut AnatomyState, is_sick: bool, hunger: f32) {
    let skeleton = &mut anatomy.skeleton;

    if skeleton.structure_type == SkeletonType::Hydrostatic {
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
        if skeleton.structure_type == SkeletonType::Dense {
            skeleton.bone_health =
                (skeleton.bone_health + cfg::skeleton::DENSE_REPAIR_BONUS).min(1.0);
        }
    }
}

fn update_muscles(anatomy: &mut AnatomyState, is_sick: bool, is_sleeping: bool, hunger: f32) {
    let muscles = &mut anatomy.muscles;

    if is_sleeping {
        for group in &mut muscles.groups {
            group.fatigue = (group.fatigue - cfg::muscles::FATIGUE_RECOVERY).max(0.0);
        }
        muscles.condition = (muscles.condition + cfg::muscles::CONDITION_REPAIR).min(1.0);
    } else {
        for group in &mut muscles.groups {
            group.fatigue = (group.fatigue + cfg::muscles::FATIGUE_ACCUMULATION).min(1.0);
        }
    }

    if is_sick || hunger > cfg::HUNGER_MUSCLE_THRESHOLD {
        muscles.condition = (muscles.condition - cfg::muscles::CONDITION_DECAY).max(0.0);
    } else if hunger < cfg::HUNGER_MUSCLE_REPAIR {
        muscles.condition = (muscles.condition + cfg::muscles::CONDITION_REPAIR).min(1.0);
    }

    muscles.tone += (muscles.condition - muscles.tone) * cfg::muscles::TONE_CONVERGENCE;
}

fn update_joints(anatomy: &mut AnatomyState, hunger: f32, is_elder: bool) {
    for joint in &mut anatomy.joints.joints {
        if hunger > cfg::HUNGER_JOINT_THRESHOLD {
            joint.lubrication =
                (joint.lubrication - cfg::joints::LUBRICATION_DECAY).max(0.0);
        } else if hunger < cfg::HUNGER_REPAIR_THRESHOLD {
            joint.lubrication =
                (joint.lubrication + cfg::joints::LUBRICATION_REPAIR).min(1.0);
        }

        if is_elder {
            joint.flexibility =
                (joint.flexibility - cfg::joints::ELDER_FLEXIBILITY_DECAY).max(0.0);
        }
    }
}

fn update_skin(anatomy: &mut AnatomyState, is_sick: bool, hunger: f32) {
    let skin = &mut anatomy.skin;

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
    if anatomy.muscles.condition < cfg::muscles::LOW_CONDITION_THRESHOLD {
        let penalty = (cfg::muscles::LOW_CONDITION_THRESHOLD - anatomy.muscles.condition)
            * cfg::muscles::ENERGY_PENALTY_FACTOR;
        mind.stats.energy = (mind.stats.energy - penalty).max(0.0);
    }

    if anatomy.avg_lubrication() < cfg::joints::STIFFNESS_THRESHOLD {
        mind.stats.energy =
            (mind.stats.energy - cfg::joints::STIFFNESS_ENERGY_PENALTY).max(0.0);
    }
}

fn apply_mood_overrides(anatomy: &AnatomyState, mind: &mut Mind, genome: &Genome) {
    let avg_flex = anatomy.avg_flexibility();

    if mind.mood == MoodState::Playful && avg_flex < cfg::joints::PLAYFUL_FLEX_BLOCK {
        mind.mood = MoodState::Happy;
    }

    if mind.mood == MoodState::Happy
        && avg_flex > 0.7
        && genome.curiosity > 0.4
        && mind.stats.happiness > 70.0
        && mind.stats.energy > 50.0
    {
        mind.mood = MoodState::Playful;
    }
}

/// Checks for broken bones and cascades damage through connected joints and muscles.
pub fn check_bone_breaks(anatomy: &mut AnatomyState, mind: &mut Mind) {
    let broken_bone_names: Vec<String> = anatomy.skeleton.bones.iter()
        .filter(|b| b.integrity <= 0.0)
        .map(|b| b.name.clone())
        .collect();

    if broken_bone_names.is_empty() {
        return;
    }

    for broken_name in &broken_bone_names {
        for joint in &mut anatomy.joints.joints {
            if joint.bone_a == *broken_name || joint.bone_b == *broken_name {
                joint.flexibility = joint.flexibility.min(cfg::skeleton::BREAK_JOINT_FLEX_MIN);
                joint.integrity = joint.integrity.min(cfg::skeleton::BREAK_JOINT_INTEGRITY_CAP);

                let joint_name = joint.name.clone();
                for group in &mut anatomy.muscles.groups {
                    if group.joint == joint_name {
                        group.strength *= cfg::skeleton::BREAK_MUSCLE_STRENGTH_FACTOR;
                    }
                }
            }
        }

        mind.stats.health =
            (mind.stats.health - cfg::skeleton::BREAK_HEALTH_PENALTY).max(0.0);
    }

    if !mind.mood.is_critical() {
        mind.mood = MoodState::Sick;
    }
}
