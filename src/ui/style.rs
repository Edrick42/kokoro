//! UI Design System — retro Game Boy aesthetic.
//!
//! Flat rectangles, color inversion on hover, pixel font.
//! All colors come from `config::ui::palette`.

use bevy::prelude::*;
use crate::config::ui::palette;

// Re-exports from config::ui for convenience.
pub use crate::config::ui::palette::*;
pub use crate::config::ui::buttons;
pub use crate::config::ui::fonts;

// ===================================================================
// BUTTON ANIMATION — retro inversion style
// ===================================================================

/// Marker for buttons that should animate on interaction.
/// Stores the button's resting background color so hover/press can invert
/// and then restore correctly (species buttons have colored backgrounds).
#[derive(Component)]
pub struct AnimatedButton;

/// Stores the original background color of a button (inserted at spawn time).
/// Used to restore after hover/press inversion.
#[derive(Component)]
pub struct ButtonRestColor(pub Color);

/// System: retro-style button feedback.
/// Hover inverts to NEAR_BLACK bg. Press adds scale squish.
/// Restores to the button's original color (from ButtonRestColor) on release.
pub fn animate_buttons(
    mut query: Query<
        (&Interaction, &mut Transform, &mut BackgroundColor, &mut BorderColor, Option<&ButtonRestColor>),
        (Changed<Interaction>, With<AnimatedButton>),
    >,
) {
    for (interaction, mut transform, mut bg, mut border, rest_color) in query.iter_mut() {
        let normal_bg = rest_color.map(|r| r.0).unwrap_or(palette::CREAM);

        match interaction {
            Interaction::Pressed => {
                transform.scale = Vec3::new(0.95, 0.95, 1.0);
                bg.0 = palette::NEAR_BLACK;
                border.0 = palette::CREAM;
            }
            Interaction::Hovered => {
                transform.scale = Vec3::ONE;
                bg.0 = palette::GRAY;
                border.0 = palette::CREAM;
            }
            Interaction::None => {
                transform.scale = Vec3::ONE;
                bg.0 = normal_bg;
                border.0 = palette::NEAR_BLACK;
            }
        }
    }
}

/// Smooth scale recovery for pressed buttons.
pub fn smooth_button_scale(
    time: Res<Time>,
    mut query: Query<(&Interaction, &mut Transform), With<AnimatedButton>>,
) {
    let dt = time.delta_secs();
    for (interaction, mut transform) in query.iter_mut() {
        let target = match interaction {
            Interaction::Pressed => Vec3::new(0.95, 0.95, 1.0),
            _ => Vec3::ONE,
        };
        let lerp_speed = 12.0 * dt;
        transform.scale = transform.scale + (target - transform.scale) * lerp_speed;
    }
}
