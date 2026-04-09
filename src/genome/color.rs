#[allow(dead_code)]
use super::Genome;

#[allow(dead_code)]
impl Genome {
    /// Returns the body color derived from the `hue` gene.
    pub fn body_color(&self) -> bevy::color::Color {
        bevy::color::Color::hsl(self.hue, 0.7, 0.75)
    }

    /// Returns a tint color for sprite rendering.
    pub fn tint_color(&self) -> bevy::color::Color {
        bevy::color::Color::hsl(self.hue, 0.65, 0.80)
    }
}
