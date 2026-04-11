//! UI design system — retro Game Boy aesthetic.
//!
//! Base palette of 7 colors that define the visual identity. Additional colors
//! are allowed for creatures, textures, and details — the palette is a foundation,
//! not a hard limit.
//!
//! ## Palette
//!
//! | Name       | Hex       | Role                                    |
//! |------------|-----------|----------------------------------------|
//! | Cream      | `#D9C7AE` | Background, panel fills, button BG      |
//! | Gray       | `#404040` | Button hover state (readable text)      |
//! | Near Black | `#1B130D` | Text, borders, dark panels              |
//! | Red        | `#D90D43` | Danger/vital, hunger, Nyxal body        |
//! | Teal       | `#016970` | Cool accent, energy, Skael body         |
//! | Gold       | `#D9A404` | Warm accent, happiness, Moluun body     |
//! | Orange     | `#D96704` | Energy accent, Pylum body               |

use bevy::prelude::*;

/// The 6-color retro palette.
pub mod palette {
    use bevy::prelude::Color;

    pub const CREAM: Color      = Color::srgb(0.851, 0.780, 0.682);
    pub const GRAY: Color       = Color::srgb(0.25, 0.25, 0.25);
    pub const NEAR_BLACK: Color = Color::srgb(0.106, 0.075, 0.051);
    pub const RED: Color        = Color::srgb(0.851, 0.051, 0.263);
    pub const TEAL: Color       = Color::srgb(0.004, 0.412, 0.439);
    pub const GOLD: Color       = Color::srgb(0.851, 0.643, 0.016);
    pub const ORANGE: Color     = Color::srgb(0.851, 0.404, 0.016);

    /// Panel background — cream with near-black border (retro style).
    pub const PANEL_BG: Color = Color::srgba(0.851, 0.780, 0.682, 0.95);
}

/// Stat display colors (mapped from palette).
pub mod stats {
    use bevy::prelude::Color;
    pub const HUNGER: Color    = super::palette::RED;
    pub const HAPPINESS: Color = super::palette::GOLD;
    pub const ENERGY: Color    = super::palette::TEAL;
}

/// Species ↔ palette color mapping.
pub mod species_colors {
    use bevy::prelude::Color;
    pub const MOLUUN: Color = super::palette::GOLD;
    pub const PYLUM: Color  = super::palette::ORANGE;
    pub const SKAEL: Color  = super::palette::TEAL;
    pub const NYXAL: Color  = super::palette::RED;
}

/// Button dimensions — flat retro rectangles.
pub mod buttons {
    pub const HEIGHT: f32 = 36.0;
    pub const BORDER_WIDTH: f32 = 2.0;
    /// Zero border radius = flat Game Boy rectangles.
    pub const RADIUS: f32 = 0.0;
    pub const GAP: f32 = 5.0;
    pub const FOOD_SIZE: f32 = 44.0;
}

/// Font sizes for the pixel font.
pub mod fonts {
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 11.0;
    pub const LG: f32 = 14.0;
}

/// Pixel font resource — loaded at startup from `assets/fonts/pixel.ttf`.
#[derive(Resource)]
pub struct PixelFont(pub Handle<Font>);

/// Plugin that loads the pixel font and makes it available as a resource.
pub struct RetroFontPlugin;

impl Plugin for RetroFontPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_pixel_font);
    }
}

fn load_pixel_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/pixel.ttf");
    commands.insert_resource(PixelFont(font));
}
