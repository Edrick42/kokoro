//! UI Design System — consistent visual language for all buttons and panels.
//!
//! All UI elements use this module for colors, sizes, and animation.
//! Minimalist, clean, fun. Dark background, colorful accents.

use bevy::prelude::*;

// ===================================================================
// COLORS — dark theme with vibrant accents
// ===================================================================

/// Panel background (semi-transparent dark).
pub const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.12, 0.88);
/// Panel border radius.
pub const PANEL_RADIUS: f32 = 12.0;

/// Button colors by state.
pub const BTN_BORDER: Color = Color::srgb(0.25, 0.25, 0.30);
pub const BTN_BORDER_HOVER: Color = Color::srgb(0.45, 0.45, 0.55);
pub const BTN_BORDER_PRESS: Color = Color::srgb(0.60, 0.60, 0.70);

/// Toggle button (menu "...").
pub const TOGGLE_BG: Color = Color::srgba(0.15, 0.15, 0.20, 0.85);
pub const TOGGLE_TEXT: Color = Color::srgb(0.80, 0.80, 0.80);

/// Text colors.
pub const TEXT_PRIMARY: Color = Color::WHITE;
pub const TEXT_SECONDARY: Color = Color::srgb(0.70, 0.70, 0.75);

// ===================================================================
// SIZES — consistent dimensions
// ===================================================================

/// Standard button sizes.
pub const BTN_HEIGHT: f32 = 32.0;
pub const BTN_ICON_SIZE: f32 = 28.0;
pub const BTN_RADIUS: f32 = 8.0;
pub const BTN_BORDER_WIDTH: f32 = 2.0;
pub const BTN_GAP: f32 = 5.0;

/// Food button (square with icon).
pub const FOOD_BTN_SIZE: f32 = 44.0;

/// Font sizes.
pub const FONT_SM: f32 = 10.0;
pub const FONT_MD: f32 = 13.0;
pub const FONT_LG: f32 = 16.0;

// ===================================================================
// BUTTON ANIMATION — handles hover/press visual feedback
// ===================================================================

/// Marker for buttons that should animate on interaction.
#[derive(Component)]
pub struct AnimatedButton;

/// System that animates button scale and border on interaction.
pub fn animate_buttons(
    mut query: Query<
        (&Interaction, &mut Transform, &mut BorderColor),
        (Changed<Interaction>, With<AnimatedButton>),
    >,
) {
    for (interaction, mut transform, mut border) in query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                // Squish down — feels like a real button press
                transform.scale = Vec3::new(0.92, 0.92, 1.0);
                border.0 = BTN_BORDER_PRESS;
            }
            Interaction::Hovered => {
                // Slight grow — invites clicking
                transform.scale = Vec3::new(1.05, 1.05, 1.0);
                border.0 = BTN_BORDER_HOVER;
            }
            Interaction::None => {
                // Return to normal
                transform.scale = Vec3::ONE;
                border.0 = BTN_BORDER;
            }
        }
    }
}

/// Smooth scale recovery — lerps back to 1.0 so transitions feel organic.
pub fn smooth_button_scale(
    time: Res<Time>,
    mut query: Query<(&Interaction, &mut Transform), With<AnimatedButton>>,
) {
    let dt = time.delta_secs();
    for (interaction, mut transform) in query.iter_mut() {
        let target = match interaction {
            Interaction::Pressed => Vec3::new(0.92, 0.92, 1.0),
            Interaction::Hovered => Vec3::new(1.05, 1.05, 1.0),
            Interaction::None => Vec3::ONE,
        };
        let lerp_speed = 12.0 * dt;
        transform.scale = transform.scale + (target - transform.scale) * lerp_speed;
    }
}
