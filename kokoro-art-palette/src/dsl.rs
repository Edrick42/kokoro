//! Pixel-art shape DSL — primitives that pack artistic intent.
//!
//! Where `image::imageproc` would give you `draw_filled_circle`, this module
//! gives you `BumpyDome` — a circle with deliberate, deterministic bumps along
//! its silhouette so it reads as "tree canopy" or "mushroom cap" or "fluffy
//! creature head" rather than "geometric primitive".
//!
//! Source of truth for shape language: `docs/aesthetic-targets.md` §3 and the
//! Phase 2 plan in §0 (roadmap of 9 weeks).
//!
//! Every primitive in this module:
//! - implements [`Brush`] so callers can compose them through a single trait
//! - takes [`Palette`] values, never raw [`u8; 3]` or `Color`
//! - paints with hard pixel edges, no anti-aliasing (retro constraint)
//! - is deterministic for a given input — no global RNG, no time-of-day side
//!   channels. Variation comes from explicit `seed` parameters.

use crate::Palette;
use image::{Rgba, RgbaImage};

/// Paints into an [`RgbaImage`]. The unifying surface of the shape DSL.
///
/// Implementors describe **what** they want drawn; this trait specifies
/// **where** the pixels land. A composition is a sequence of brushes painted
/// in order — later brushes occlude earlier ones.
pub trait Brush {
    fn paint(&self, img: &mut RgbaImage);
}

// =====================================================================
// BumpyDome — the first non-trivial primitive
// =====================================================================

/// A round shape with deliberate silhouette bumps. Use for tree canopies,
/// mushroom caps, fluffy cub heads, fungal blooms — anything organic that
/// shouldn't read as geometric.
///
/// The dome is divided into `bumps` angular sectors. Each sector's outer
/// radius is perturbed by a value derived from `(sector, seed)`, then the
/// pixel is painted if its distance from the center falls inside that
/// sector's effective radius.
///
/// At `bumpiness = 0.0` the dome collapses to a flat-edged circle. At
/// `bumpiness = 1.0` the bumps reach `±radius / 4` of perturbation —
/// strong enough to read as a knobbly silhouette without breaking
/// recognition of the underlying round form.
#[derive(Debug, Copy, Clone)]
pub struct BumpyDome {
    pub cx: i32,
    pub cy: i32,
    pub radius: u32,
    /// 0.0 = perfect circle, 1.0 = strong bumps. Clamped at paint time.
    pub bumpiness: f32,
    /// How many bumps run around the silhouette. 6–10 reads as organic.
    pub bumps: u32,
    /// Per-instance variation. Same `seed` = same silhouette every frame.
    pub seed: u32,
    pub color: Palette,
    /// Optional one-pixel-thick rim painted on the bottom-right of every
    /// sector. Skipped when `None`.
    pub shadow: Option<Palette>,
}

impl BumpyDome {
    pub const fn new(cx: i32, cy: i32, radius: u32, color: Palette) -> Self {
        Self {
            cx,
            cy,
            radius,
            bumpiness: 0.4,
            bumps: 8,
            seed: 0,
            color,
            shadow: None,
        }
    }

    pub const fn with_bumpiness(mut self, b: f32) -> Self {
        self.bumpiness = b;
        self
    }

    pub const fn with_bumps(mut self, n: u32) -> Self {
        self.bumps = n;
        self
    }

    pub const fn with_seed(mut self, s: u32) -> Self {
        self.seed = s;
        self
    }

    pub const fn with_shadow(mut self, s: Palette) -> Self {
        self.shadow = Some(s);
        self
    }
}

impl BumpyDome {
    /// Escape hatch when the caller already has a fully resolved Rgba — for
    /// instance when a runtime tint (day/night cycle) has been applied to the
    /// palette color before drawing. Prefer the trait `paint` for new code so
    /// the type system keeps enforcing the palette constraint.
    pub fn paint_with(&self, img: &mut RgbaImage, body: Rgba<u8>, shadow_pixel: Option<Rgba<u8>>) {
        let bumpiness = self.bumpiness.clamp(0.0, 1.0);
        let bumps = self.bumps.max(1);
        let r = self.radius as f32;
        let max_perturb = r * 0.25 * bumpiness;

        // Pre-compute per-sector radii so we don't recalc inside the pixel loop.
        let mut sector_radii = [0.0_f32; 32];
        let n = bumps.min(32) as usize;
        for s in 0..n {
            let h = hash2(s as u32, self.seed);
            let perturb = ((h % 1000) as f32 / 1000.0 - 0.5) * 2.0 * max_perturb;
            sector_radii[s] = r + perturb;
        }

        let bound = (self.radius as i32) + (max_perturb as i32) + 2;
        let w = img.width() as i32;
        let h = img.height() as i32;
        let two_pi = std::f32::consts::TAU;

        for dy in -bound..=bound {
            for dx in -bound..=bound {
                let px = self.cx + dx;
                let py = self.cy + dy;
                if px < 0 || py < 0 || px >= w || py >= h {
                    continue;
                }

                let dist_sq = (dx * dx + dy * dy) as f32;
                let angle = (dy as f32).atan2(dx as f32);
                let normalized = (angle + two_pi) % two_pi;
                let sector = ((normalized / two_pi) * bumps as f32) as usize % n;
                let eff_r = sector_radii[sector];

                if dist_sq <= eff_r * eff_r {
                    img.put_pixel(px as u32, py as u32, body);
                } else if let Some(s_px) = shadow_pixel {
                    let rim_outer = eff_r + 1.0;
                    if dist_sq <= rim_outer * rim_outer && dx + dy > 0 {
                        img.put_pixel(px as u32, py as u32, s_px);
                    }
                }
            }
        }
    }
}

impl Brush for BumpyDome {
    fn paint(&self, img: &mut RgbaImage) {
        self.paint_with(img, self.color.into(), self.shadow.map(Into::into));
    }
}

/// Tiny deterministic hash. Splitmix-style folded to u32. Enough entropy for
/// per-sector perturbation; not cryptographic.
const fn hash2(a: u32, b: u32) -> u32 {
    let mut x = a.wrapping_mul(0x9E37_79B9).wrapping_add(b);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2_AE35);
    x ^= x >> 16;
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    fn save_swatch(img: &RgbaImage, name: &str) {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/dsl-swatches");
        let _ = std::fs::create_dir_all(&dir);
        let _ = img.save(dir.join(name));
    }

    #[test]
    fn bumpy_dome_paints_within_bounds() {
        let mut img = RgbaImage::new(64, 64);
        BumpyDome::new(32, 32, 20, Palette::Forest)
            .with_bumpiness(0.5)
            .with_bumps(8)
            .with_seed(7)
            .paint(&mut img);

        // At least one body pixel must exist near the center.
        let center = img.get_pixel(32, 32);
        assert_eq!(center, &Rgba(Palette::Forest.rgba(255)));

        // No pixel outside bounding box should have been touched (still alpha 0).
        let outside = img.get_pixel(0, 0);
        assert_eq!(outside.0[3], 0);

        save_swatch(&img, "bumpy_dome_forest_seed7.png");
    }

    #[test]
    fn bumpy_dome_seed_variation_changes_silhouette() {
        // Same params + different seed => the painted pixels differ.
        let mut a = RgbaImage::new(64, 64);
        let mut b = RgbaImage::new(64, 64);

        BumpyDome::new(32, 32, 20, Palette::Forest)
            .with_bumpiness(0.8)
            .with_bumps(7)
            .with_seed(1)
            .paint(&mut a);

        BumpyDome::new(32, 32, 20, Palette::Forest)
            .with_bumpiness(0.8)
            .with_bumps(7)
            .with_seed(42)
            .paint(&mut b);

        let differing = a
            .pixels()
            .zip(b.pixels())
            .filter(|(p, q)| p != q)
            .count();
        assert!(
            differing > 5,
            "expected silhouettes to differ across seeds, got {differing} differing pixels"
        );

        save_swatch(&a, "bumpy_dome_seed1.png");
        save_swatch(&b, "bumpy_dome_seed42.png");
    }

    #[test]
    fn bumpy_dome_zero_bumpiness_is_a_circle() {
        // bumpiness=0 should produce a near-perfect filled circle.
        let mut img = RgbaImage::new(64, 64);
        BumpyDome::new(32, 32, 18, Palette::Gold)
            .with_bumpiness(0.0)
            .paint(&mut img);

        // Inside r-1: every pixel must be body color.
        for dy in -16..=16i32 {
            for dx in -16..=16i32 {
                if dx * dx + dy * dy > 16 * 16 {
                    continue;
                }
                let p = img.get_pixel((32 + dx) as u32, (32 + dy) as u32);
                assert_eq!(
                    p,
                    &Rgba(Palette::Gold.rgba(255)),
                    "pixel ({dx},{dy}) inside core radius should be Gold"
                );
            }
        }

        save_swatch(&img, "bumpy_dome_circle.png");
    }

    #[test]
    fn bumpy_dome_with_shadow_paints_rim_pixels() {
        let mut img = RgbaImage::new(64, 64);
        BumpyDome::new(32, 32, 18, Palette::Gold)
            .with_bumpiness(0.3)
            .with_shadow(Palette::GoldDark)
            .paint(&mut img);

        let shadow_px: Rgba<u8> = Palette::GoldDark.into();
        let saw_shadow = img.pixels().any(|p| p == &shadow_px);
        assert!(saw_shadow, "expected at least one GoldDark rim pixel");

        save_swatch(&img, "bumpy_dome_shadowed.png");
    }
}
