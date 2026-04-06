//! Touch system — nervous system-based interaction with the creature.
//!
//! Detects mouse/touch clicks on body parts using raycasting from the camera.
//! Each body part has a sensitivity profile (pleasure, pain, warmth) that
//! determines how the creature reacts to being touched there.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::config::nervous_system as nerv;
use crate::creature::species::{BodyPartSlot, CreatureRoot};
use crate::genome::Genome;
use crate::mind::Mind;
use crate::visuals::species_behavior::BasePosition;

/// Event fired when the player touches a body part.
#[derive(Event)]
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
           .add_systems(Update, (detect_touch, apply_touch_effects).chain());
    }
}

/// Detects mouse clicks on creature body parts via distance-based hit testing.
fn detect_touch(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    root_q: Query<(&Transform, &Children), With<CreatureRoot>>,
    part_q: Query<(&BodyPartSlot, &GlobalTransform)>,
    genome: Res<Genome>,
    mut touch_events: EventWriter<TouchEvent>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    // Convert screen position to world coordinates
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    // Find the closest body part to the click
    let mut closest: Option<(String, f32)> = None;

    for (_root_transform, children) in root_q.iter() {
        for child in children.iter() {
            let Ok((slot, global_transform)) = part_q.get(child) else { continue };

            let part_pos = global_transform.translation().truncate();
            let distance = world_pos.distance(part_pos);

            if distance < nerv::HIT_RADIUS {
                if closest.as_ref().map_or(true, |(_, d)| distance < *d) {
                    closest = Some((slot.0.clone(), distance));
                }
            }
        }
    }

    if let Some((slot, _dist)) = closest {
        let sens = nerv::sensitivity(&genome.species, &slot);
        touch_events.write(TouchEvent {
            slot: slot.clone(),
            pleasure: sens.pleasure,
            pain: sens.pain,
            warmth: sens.warmth,
        });
        info!("Touch: {} (pleasure={:.1}, pain={:.1}, warmth={:.1})", slot, sens.pleasure, sens.pain, sens.warmth);
    }
}

/// Applies touch effects to creature stats.
fn apply_touch_effects(
    mut events: EventReader<TouchEvent>,
    mut mind: ResMut<Mind>,
    genome: Res<Genome>,
) {
    for event in events.read() {
        let sens = nerv::sensitivity(&genome.species, &event.slot);
        let happiness_delta = nerv::touch_happiness(&sens);
        let energy_delta = nerv::touch_energy(&sens);

        mind.pending_happiness += happiness_delta;
        mind.pending_energy += energy_delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::Species;

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
        let sens = nerv::sensitivity(&Species::Nyxal, "tentacle_front_left");
        assert!(sens.pleasure > 0.8);
    }
}
