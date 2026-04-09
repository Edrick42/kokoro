//! Mood synchronization.
//!
//! With the pixel art renderer, mood-driven visual changes are handled
//! by pixel_creature::update_pixel_creature(). This function is a no-op.

/// Placeholder — mood sync is now handled by PixelCreaturePlugin.
pub fn sync_mood_sprites() {
    // No-op: pixel_creature redraws automatically on mind.is_changed()
}
