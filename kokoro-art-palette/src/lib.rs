//! Kokoro art palette — 26 hand-curated colors organized in 9 ramps.
//!
//! Source of truth lives in `docs/aesthetic-targets.md` §1 and
//! `docs/art-direction.md`. Editing this file in isolation will drift the
//! game from its visual direction. Update the docs first, then the enum.
//!
//! Design rule, enforced by types:
//! pixel-rendering code MUST take a `Palette` value, never a raw `u32` or
//! `[u8; 3]`. Anything that wants to draw a pixel goes through this enum.

pub mod ramp;

#[cfg(feature = "image")]
pub mod dsl;

/// All 26 colors of the Kokoro master palette.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Palette {
    NearBlack,
    DeepBrown,
    DeepTeal,
    Charcoal,

    Cream,
    CreamLight,
    OffWhite,

    Tan,
    Brown,
    BrownDark,
    Sand,

    GoldDark,
    Gold,
    OrangeBright,

    OrangeDark,
    Orange,
    OrangeLight,

    RedDark,
    Red,
    CoralPink,

    TealDark,
    Teal,
    CyanBright,

    ForestDark,
    Forest,
    Sage,
}

impl Palette {
    /// Packed 0xRRGGBB hex value.
    pub const fn hex(self) -> u32 {
        match self {
            Palette::NearBlack    => 0x1B130D,
            Palette::DeepBrown    => 0x3B2418,
            Palette::DeepTeal     => 0x0A2A2D,
            Palette::Charcoal     => 0x2A2520,

            Palette::Cream        => 0xD9C7AE,
            Palette::CreamLight   => 0xF0E2C8,
            Palette::OffWhite     => 0xFBF4E2,

            Palette::Tan          => 0xC49870,
            Palette::Brown        => 0x8B5A3A,
            Palette::BrownDark    => 0x5A3622,
            Palette::Sand         => 0xE5C896,

            Palette::GoldDark     => 0xA07803,
            Palette::Gold         => 0xD9A404,
            Palette::OrangeBright => 0xF08828,

            Palette::OrangeDark   => 0xA04C03,
            Palette::Orange       => 0xD96704,
            Palette::OrangeLight  => 0xF0883D,

            Palette::RedDark      => 0xA00930,
            Palette::Red          => 0xD90D43,
            Palette::CoralPink    => 0xF06B85,

            Palette::TealDark     => 0x014045,
            Palette::Teal         => 0x016970,
            Palette::CyanBright   => 0x4DC3CC,

            Palette::ForestDark   => 0x1F3A28,
            Palette::Forest       => 0x3D6E45,
            Palette::Sage         => 0x87A878,
        }
    }

    pub const fn rgb(self) -> [u8; 3] {
        let h = self.hex();
        [(h >> 16) as u8, (h >> 8) as u8, h as u8]
    }

    pub const fn rgba(self, alpha: u8) -> [u8; 4] {
        let [r, g, b] = self.rgb();
        [r, g, b, alpha]
    }

    /// Stable ordering — useful for indexing, serialization, palette swatches.
    pub const ALL: [Palette; 26] = [
        Palette::NearBlack, Palette::DeepBrown, Palette::DeepTeal, Palette::Charcoal,
        Palette::Cream, Palette::CreamLight, Palette::OffWhite,
        Palette::Tan, Palette::Brown, Palette::BrownDark, Palette::Sand,
        Palette::GoldDark, Palette::Gold, Palette::OrangeBright,
        Palette::OrangeDark, Palette::Orange, Palette::OrangeLight,
        Palette::RedDark, Palette::Red, Palette::CoralPink,
        Palette::TealDark, Palette::Teal, Palette::CyanBright,
        Palette::ForestDark, Palette::Forest, Palette::Sage,
    ];
}

#[cfg(feature = "bevy")]
impl From<Palette> for bevy_color::Color {
    fn from(p: Palette) -> Self {
        let [r, g, b] = p.rgb();
        bevy_color::Color::srgb_u8(r, g, b)
    }
}

#[cfg(feature = "image")]
impl From<Palette> for image::Rgba<u8> {
    fn from(p: Palette) -> Self {
        image::Rgba(p.rgba(0xFF))
    }
}

#[cfg(feature = "image")]
impl From<Palette> for image::Rgb<u8> {
    fn from(p: Palette) -> Self {
        image::Rgb(p.rgb())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_26_colors_have_unique_hex() {
        let mut seen = std::collections::HashSet::new();
        for c in Palette::ALL {
            assert!(seen.insert(c.hex()), "duplicate hex for {:?}", c);
        }
        assert_eq!(seen.len(), 26);
    }

    #[test]
    fn rgb_decomposition_is_consistent() {
        for c in Palette::ALL {
            let [r, g, b] = c.rgb();
            let recomposed = (r as u32) << 16 | (g as u32) << 8 | b as u32;
            assert_eq!(recomposed, c.hex(), "{:?} round-trip failed", c);
        }
    }

    #[test]
    fn anchor_values_match_aesthetic_targets_doc() {
        // Sanity check: a few well-known anchors match the documented hex.
        assert_eq!(Palette::Gold.hex(),       0xD9A404);
        assert_eq!(Palette::Cream.hex(),      0xD9C7AE);
        assert_eq!(Palette::NearBlack.hex(),  0x1B130D);
        assert_eq!(Palette::Red.hex(),        0xD90D43);
        assert_eq!(Palette::Teal.hex(),       0x016970);
        assert_eq!(Palette::CoralPink.hex(),  0xF06B85);
        assert_eq!(Palette::Forest.hex(),     0x3D6E45);
    }
}
