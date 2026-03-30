//! Mood-driven sprite swapping.
//!
//! When the creature's mood changes, this system updates only the
//! mood-reactive body parts (eyes, mouth) by swapping their sprite
//! handles. Non-reactive parts (body, ears) remain untouched.
//!
//! If a mood-specific sprite doesn't exist for a part, it falls back
//! to the idle variant.

use bevy::prelude::*;
use crate::mind::Mind;
use super::body_parts::{BodyPartSlot, MoodReactive};
use super::creature_spawn::PartSpriteHandles;

/// Watches for mood changes and swaps sprites on mood-reactive parts.
///
/// Only runs when the `Mind` resource has actually changed (Bevy's
/// change detection via `is_changed()` prevents wasted work).
pub fn sync_mood_sprites(
    mind: Res<Mind>,
    handles: Option<Res<PartSpriteHandles>>,
    mut query: Query<(&BodyPartSlot, &mut Sprite), With<MoodReactive>>,
) {
    if !mind.is_changed() {
        return;
    }

    let Some(handles) = handles else { return };
    let mood_key = mind.mood.mood_key();

    for (slot, mut sprite) in query.iter_mut() {
        // Try mood-specific sprite first, then fall back to idle
        let new_handle = handles.handles
            .get(&(slot.0.clone(), mood_key.to_string()))
            .or_else(|| handles.handles.get(&(slot.0.clone(), "idle".to_string())));

        if let Some(handle) = new_handle {
            sprite.image = handle.clone();
        }
    }
}
