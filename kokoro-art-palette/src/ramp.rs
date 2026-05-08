//! Ramps — three-tone bands of related colors used together.
//!
//! Source of truth: `docs/aesthetic-targets.md` §1 "Paleta master".
//! Each named ramp packages an artistic decision (which 3 colors form
//! a coherent shading band for a given material).

use crate::Palette;

/// A 3-tone shading band: shadow → mid → highlight.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Ramp {
    pub shadow: Palette,
    pub mid: Palette,
    pub highlight: Palette,
}

impl Ramp {
    pub const fn new(shadow: Palette, mid: Palette, highlight: Palette) -> Self {
        Self { shadow, mid, highlight }
    }

    /// Index 0 = shadow, 1 = mid, 2 = highlight. Out-of-range returns mid.
    pub const fn at(&self, i: usize) -> Palette {
        match i {
            0 => self.shadow,
            2 => self.highlight,
            _ => self.mid,
        }
    }

    pub const fn as_array(&self) -> [Palette; 3] {
        [self.shadow, self.mid, self.highlight]
    }
}

/// Pre-defined material ramps. These are the canonical bands referenced
/// throughout `docs/aesthetic-targets.md` §4 (per-species mappings).
pub mod materials {
    use super::*;
    use Palette::*;

    pub const GOLD:   Ramp = Ramp::new(GoldDark,   Gold,   OrangeBright);
    pub const ORANGE: Ramp = Ramp::new(OrangeDark, Orange, OrangeLight);
    pub const RED:    Ramp = Ramp::new(RedDark,    Red,    CoralPink);
    pub const TEAL:   Ramp = Ramp::new(TealDark,   Teal,   CyanBright);
    pub const FOREST: Ramp = Ramp::new(ForestDark, Forest, Sage);
    pub const FUR:    Ramp = Ramp::new(BrownDark,  Brown,  Tan);
    pub const CREAM:  Ramp = Ramp::new(Cream,      CreamLight, OffWhite);
}

#[cfg(test)]
mod tests {
    use super::*;
    use materials::*;

    #[test]
    fn ramps_use_only_palette_colors() {
        for r in [GOLD, ORANGE, RED, TEAL, FOREST, FUR, CREAM] {
            for c in r.as_array() {
                assert!(Palette::ALL.contains(&c));
            }
        }
    }

    #[test]
    fn at_indexing() {
        assert_eq!(GOLD.at(0), Palette::GoldDark);
        assert_eq!(GOLD.at(1), Palette::Gold);
        assert_eq!(GOLD.at(2), Palette::OrangeBright);
        assert_eq!(GOLD.at(99), Palette::Gold); // out-of-range falls back to mid
    }
}
