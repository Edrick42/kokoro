use super::Genome;

impl Genome {
    /// Returns the body color derived from the `hue` gene.
    pub fn body_color(&self) -> bevy::color::Color {
        bevy::color::Color::hsl(self.hue, 0.7, 0.75)
    }

    /// Returns a tint color for sprite rendering.
    ///
    /// Unlike `body_color()` which is meant for procedural meshes, this returns
    /// a slightly lighter, more saturated color that looks good when multiplied
    /// onto a flat-colored pixel art sprite.
    pub fn tint_color(&self) -> bevy::color::Color {
        bevy::color::Color::hsl(self.hue, 0.65, 0.80)
    }
}
