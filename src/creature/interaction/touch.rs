//! Touch system — nervous system-based interaction with the creature.
//!
//! Detects mouse clicks in the creature's local soft-body space (64×64).
//! The nearest soft-body point within a hit radius is "touched":
//! - Its sensitivity (pleasure/pain/warmth) updates the creature's stats
//! - The click direction is applied as an impulse — the point physically
//!   moves away from the finger (or toward it, for pulls)
//! - A reaction event fires (Petted or Flinched) depending on pleasure vs pain

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::soft_body::SoftBody;
use crate::config::nervous_system as nerv;
use crate::creature::identity::species::CreatureRoot;
use crate::game::state::AppState;
use crate::genome::{Genome, Species};
use crate::mind::Mind;

/// Event fired when the player touches a body part.
#[derive(Event)]
#[allow(dead_code)]
pub struct TouchEvent {
    pub slot: String,
    pub pleasure: f32,
    pub pain: f32,
    pub warmth: f32,
}

pub struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TouchEvent>()
           .add_systems(Update, (detect_touch, apply_touch_effects)
                .chain()
                .run_if(in_state(AppState::Gameplay)));
    }
}

/// DISPLAY_SCALE must match the one in `visuals::skin`.
const DISPLAY_SCALE: f32 = 5.0;
/// Soft-body canvas origin (center point in the 64×64 buffer).
const CANVAS_CENTER: f32 = 32.0;
/// Maximum distance (in soft-body pixels) a click can be from a point to register.
const HIT_RADIUS_SB: f32 = 10.0;
/// Impulse magnitude from a touch poke.
const POKE_IMPULSE: f32 = 18.0;

/// Converts a world-space click into soft-body-local coords (64×64, top-left origin).
fn world_to_soft_body(world: Vec2, root: Vec2) -> Vec2 {
    let local = (world - root) / DISPLAY_SCALE;
    // Flip Y: world up = +y, soft-body up = -y (image top is y=0)
    Vec2::new(CANVAS_CENTER + local.x, CANVAS_CENTER - local.y)
}

/// Finds the closest soft-body point to a given soft-body-local position.
/// Returns (point name, distance) if any point is within `HIT_RADIUS_SB`.
fn nearest_point(body: &SoftBody, target: Vec2) -> Option<(String, f32)> {
    body.points.iter()
        .map(|p| (p.name.clone(), p.position.distance(target)))
        .filter(|(_, d)| *d <= HIT_RADIUS_SB)
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
}

/// Translates a soft-body point name into a nervous-system sensitivity slot.
/// Different species have different body plans — this maps points to the
/// sensitivity profiles defined in `config::nervous_system`.
fn point_to_slot(species: &Species, point: &str) -> &'static str {
    // Universal: face features map to the same slots for every species.
    // Eyes always painful, mouth painful for some species.
    match point {
        "eye_l" => return "eye_left",
        "eye_r" => return "eye_right",
        "mouth" => return match species {
            Species::Moluun => "mouth",
            Species::Pylum  => "beak",
            Species::Skael  => "snout",
            Species::Nyxal  => "body", // no specific mouth sensitivity for cephalopod
        },
        _ => {}
    }
    match (species, point) {
        // Moluun mammal
        (Species::Moluun, "ear_anchor") => "ear_left",
        (Species::Moluun, "head") => "body",
        // Pylum bird
        (Species::Pylum, "head" | "casque") => "beak",
        (Species::Pylum, "wing_l" | "wingtip_l") => "wing_left",
        (Species::Pylum, "wing_r" | "wingtip_r") => "wing_right",
        (Species::Pylum, "tail") => "tail",
        // Skael reptile
        (Species::Skael, "horn_l") => "crest_left",
        (Species::Skael, "horn_r") => "crest_right",
        (Species::Skael, "head") => "snout",
        (Species::Skael, s) if s.starts_with("tail_") => "tail",
        // Nyxal cephalopod
        (Species::Nyxal, "mantle_top") => "mantle",
        (Species::Nyxal, s) if s.starts_with("tent_f") || s.starts_with("tip_f") => "tentacle_front_left",
        (Species::Nyxal, s) if s.starts_with("tent_b") || s.starts_with("tip_b") => "tentacle_back_left",
        // Everything else → generic body
        _ => "body",
    }
}

/// Detects mouse clicks, finds the nearest soft-body point, fires touch + impulse.
fn detect_touch(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    root_q: Query<&GlobalTransform, With<CreatureRoot>>,
    genome: Res<Genome>,
    mut soft_body: Option<ResMut<SoftBody>>,
    mut touch_events: EventWriter<TouchEvent>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }

    let Some(ref mut body) = soft_body else { return };

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };
    let Ok(root_transform) = root_q.single() else { return };

    let root_pos = root_transform.translation().truncate();
    let sb_pos = world_to_soft_body(world_pos, root_pos);

    let Some((point_name, _dist)) = nearest_point(body, sb_pos) else { return };

    // Apply a poke: push the point AWAY from the click (outward from the cursor).
    // In soft-body coords, "away from click" = from sb_pos toward the point's position.
    let point = body.point(&point_name);
    let push_dir = (point.position - sb_pos).normalize_or_zero();
    // World Y points up, soft-body Y points down — keep impulses in soft-body frame here.
    body.impulse(&point_name, push_dir * POKE_IMPULSE);

    // Sensitivity lookup using the translated slot name
    let slot = point_to_slot(&genome.species, &point_name);
    let sens = nerv::sensitivity(&genome.species, slot);

    touch_events.write(TouchEvent {
        slot: point_name,
        pleasure: sens.pleasure,
        pain: sens.pain,
        warmth: sens.warmth,
    });
}

/// Applies touch effects to creature stats and fires visual reaction.
fn apply_touch_effects(
    mut events: EventReader<TouchEvent>,
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
    mut involuntary: ResMut<crate::creature::behavior::involuntary::InvoluntaryState>,
    mut reaction_events: EventWriter<crate::creature::behavior::reactions::CreatureReaction>,
) {
    for event in events.read() {
        let slot = point_to_slot(&genome.species, &event.slot);
        let sens = nerv::sensitivity(&genome.species, slot);
        let happiness_delta = nerv::touch_happiness(&sens);
        let energy_delta = nerv::touch_energy(&sens);

        mind.pending_happiness += happiness_delta;
        mind.pending_energy += energy_delta;

        if event.pain > 0.5 {
            crate::creature::behavior::involuntary::trigger_startle(&mut involuntary);
            reaction_events.write(crate::creature::behavior::reactions::CreatureReaction::Flinched {
                pain: event.pain,
            });
        } else if event.pleasure > 0.3 {
            reaction_events.write(crate::creature::behavior::reactions::CreatureReaction::Petted {
                pleasure: event.pleasure,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moluun_ear_is_sweet_spot() {
        let sens = nerv::sensitivity(&Species::Moluun, "ear_left");
        assert!(sens.pleasure > 0.8);
        assert!(nerv::touch_happiness(&sens) > 0.0);
    }

    #[test]
    fn eye_touch_is_painful() {
        let sens = nerv::sensitivity(&Species::Moluun, "eye_left");
        assert!(sens.pain > 0.8);
        assert!(nerv::touch_happiness(&sens) < 0.0);
    }

    #[test]
    fn nyxal_tentacle_tip_is_pleasure() {
        // Point "tip_fl" → slot "tentacle_front_left"
        let slot = point_to_slot(&Species::Nyxal, "tip_fl");
        assert_eq!(slot, "tentacle_front_left");
        let sens = nerv::sensitivity(&Species::Nyxal, slot);
        assert!(sens.pleasure > 0.8);
    }

    #[test]
    fn moluun_ear_anchor_maps_to_ear_slot() {
        let slot = point_to_slot(&Species::Moluun, "ear_anchor");
        assert_eq!(slot, "ear_left");
    }

    #[test]
    fn eye_points_map_to_eye_slots_all_species() {
        for species in &[Species::Moluun, Species::Pylum, Species::Skael, Species::Nyxal] {
            assert_eq!(point_to_slot(species, "eye_l"), "eye_left");
            assert_eq!(point_to_slot(species, "eye_r"), "eye_right");
        }
    }

    #[test]
    fn eye_touch_triggers_pain_via_soft_body() {
        // Eye touch returns high pain for all species (sensitivity defined in nervous_system.rs)
        for species in &[Species::Moluun, Species::Pylum, Species::Skael, Species::Nyxal] {
            let slot = point_to_slot(species, "eye_l");
            let sens = nerv::sensitivity(species, slot);
            assert!(sens.pain > 0.7, "{:?} eye_l should be painful", species);
        }
    }
}
