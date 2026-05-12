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
    /// Horizontal radius. With `ry == None` this also acts as the vertical
    /// radius (perfect circle).
    pub radius: u32,
    /// Optional independent vertical radius. `None` = circle (== radius).
    /// `Some(r)` makes the dome an ellipse, useful for eggs, oval bodies,
    /// elongated mantles.
    pub ry: Option<u32>,
    /// 0.0 = perfect circle/ellipse, 1.0 = strong bumps. Clamped at paint time.
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
            ry: None,
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

    /// Make the dome elliptical with an independent vertical radius. Useful
    /// for eggs (oval), wide bodies, elongated mantles.
    pub const fn with_height(mut self, ry: u32) -> Self {
        self.ry = Some(ry);
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
        let rx = self.radius.max(1) as f32;
        let ry = self.ry.unwrap_or(self.radius).max(1) as f32;
        // Bumps perturb the unit-circle distance; scale it by the smaller
        // axis so an ellipse doesn't get exaggerated bumps along the wider
        // direction.
        let max_perturb = rx.min(ry) * 0.25 * bumpiness;

        // Pre-compute per-sector perturbation factors. Stored as a
        // multiplier on the unit-distance threshold: sector_r[s] in
        // (1 - k .. 1 + k) range where k = max_perturb / min_axis.
        let mut sector_factor = [1.0_f32; 32];
        let n = bumps.min(32) as usize;
        let perturb_scale = max_perturb / rx.min(ry);
        for s in 0..n {
            let h = hash2(s as u32, self.seed);
            let perturb = ((h % 1000) as f32 / 1000.0 - 0.5) * 2.0 * perturb_scale;
            sector_factor[s] = 1.0 + perturb;
        }

        let bound_x = (self.radius as i32) + (max_perturb as i32) + 2;
        let bound_y = (self.ry.unwrap_or(self.radius) as i32) + (max_perturb as i32) + 2;
        let w = img.width() as i32;
        let h = img.height() as i32;
        let two_pi = std::f32::consts::TAU;

        for dy in -bound_y..=bound_y {
            for dx in -bound_x..=bound_x {
                let px = self.cx + dx;
                let py = self.cy + dy;
                if px < 0 || py < 0 || px >= w || py >= h {
                    continue;
                }

                // Normalised ellipse distance — squared, so 1.0 is the unit
                // boundary. Bumps move that boundary in/out per sector.
                let nx = dx as f32 / rx;
                let ny = dy as f32 / ry;
                let dist_sq = nx * nx + ny * ny;

                let angle = (dy as f32).atan2(dx as f32);
                let normalized = (angle + two_pi) % two_pi;
                let sector = ((normalized / two_pi) * bumps as f32) as usize % n;
                let factor = sector_factor[sector];
                let threshold = factor * factor; // compare on squared

                if dist_sq <= threshold {
                    img.put_pixel(px as u32, py as u32, body);
                } else if let Some(s_px) = shadow_pixel {
                    // 1px outward rim — bump factor + a small constant in
                    // normalized space.
                    let rim_threshold = (factor + 1.0 / rx.min(ry)).powi(2);
                    if dist_sq <= rim_threshold && dx + dy > 0 {
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

// =====================================================================
// TaperedTail — second primitive: a band that starts thick and narrows to a point
// =====================================================================

/// A line-shape that starts at `base` with `base_width` half-thickness and
/// tapers linearly to a single pixel at `tip`. Use for tails, beaks, claws,
/// horns, tentacles, leaf veins — anything that reads as "this end is heavy,
/// the other end is fine."
///
/// The tip is always a point; if you want a blunt-ended band, use a chain of
/// short TaperedTails or compose with BumpyDome.
#[derive(Debug, Copy, Clone)]
pub struct TaperedTail {
    pub base: (i32, i32),
    pub tip: (i32, i32),
    /// Half-thickness at the base, in pixels. The full base width is
    /// `2 * base_width + 1`. Tip is always 1px wide.
    pub base_width: u32,
    pub color: Palette,
    pub shadow: Option<Palette>,
}

impl TaperedTail {
    pub const fn new(base: (i32, i32), tip: (i32, i32), base_width: u32, color: Palette) -> Self {
        Self { base, tip, base_width, color, shadow: None }
    }

    pub const fn with_shadow(mut self, s: Palette) -> Self {
        self.shadow = Some(s);
        self
    }

    /// Same escape-hatch pattern as BumpyDome: pre-resolved Rgba bypassing the
    /// Palette type. Use when a runtime tint has already been applied.
    pub fn paint_with(&self, img: &mut RgbaImage, body: Rgba<u8>, shadow_pixel: Option<Rgba<u8>>) {
        let (sx, sy) = self.base;
        let (ex, ey) = self.tip;
        let dx = (ex - sx) as f32;
        let dy = (ey - sy) as f32;
        let length = (dx * dx + dy * dy).sqrt().max(1.0);

        // Walk along the spine at sub-pixel steps so we never leave gaps even
        // when the tail is steep. 2 samples per pixel of length is plenty for
        // pixel art and keeps the cost trivial.
        let steps = (length * 2.0) as i32 + 1;
        let perp_x = -dy / length;
        let perp_y = dx / length;
        let w_img = img.width() as i32;
        let h_img = img.height() as i32;

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let half_w = (self.base_width as f32 * (1.0 - t)).round() as i32;
            let cx = sx as f32 + dx * t;
            let cy = sy as f32 + dy * t;

            // Body band: −half_w..=half_w along the perpendicular.
            for off in -half_w..=half_w {
                let px = (cx + perp_x * off as f32).round() as i32;
                let py = (cy + perp_y * off as f32).round() as i32;
                if px >= 0 && py >= 0 && px < w_img && py < h_img {
                    img.put_pixel(px as u32, py as u32, body);
                }
            }

            // Shadow rim: one pixel beyond the band on the lower side.
            if let Some(s_px) = shadow_pixel {
                let off = half_w + 1;
                let px = (cx + perp_x * off as f32).round() as i32;
                let py = (cy + perp_y * off as f32).round() as i32;
                if px >= 0 && py >= 0 && px < w_img && py < h_img {
                    img.put_pixel(px as u32, py as u32, s_px);
                }
            }
        }
    }
}

impl Brush for TaperedTail {
    fn paint(&self, img: &mut RgbaImage) {
        self.paint_with(img, self.color.into(), self.shadow.map(Into::into));
    }
}

// =====================================================================
// BioluminescentSpeck — third primitive: a glowing pixel cluster
// =====================================================================

/// A small bright point with an optional haloed outer ring. Use for
/// bioluminescent specks (Nyxal kokoro-sac, deep-water orbs), magic
/// sparkles, fireflies, thermal-pit organs — anything that should read as
/// "this is a light source on a dark background."
///
/// Stays under 5 pixels of total diameter on purpose: a "speck" shouldn't
/// dominate a sprite; clusters of multiple specks do that job better.
#[derive(Debug, Copy, Clone)]
pub struct BioluminescentSpeck {
    pub cx: i32,
    pub cy: i32,
    /// Bright core radius. 0 = single pixel; 1 = plus-shape; 2 = 5x5 disk.
    pub core_radius: u32,
    pub color: Palette,
    /// Optional one-pixel halo painted on the ring just outside the core.
    pub halo: Option<Palette>,
}

impl BioluminescentSpeck {
    pub const fn new(cx: i32, cy: i32, color: Palette) -> Self {
        Self { cx, cy, core_radius: 1, color, halo: None }
    }

    pub const fn with_core_radius(mut self, r: u32) -> Self {
        self.core_radius = r;
        self
    }

    pub const fn with_halo(mut self, h: Palette) -> Self {
        self.halo = Some(h);
        self
    }

    pub fn paint_with(&self, img: &mut RgbaImage, body: Rgba<u8>, halo_pixel: Option<Rgba<u8>>) {
        let r = self.core_radius.min(3) as i32;
        let w_img = img.width() as i32;
        let h_img = img.height() as i32;
        let halo_r = r + 1;

        for dy in -halo_r..=halo_r {
            for dx in -halo_r..=halo_r {
                let px = self.cx + dx;
                let py = self.cy + dy;
                if px < 0 || py < 0 || px >= w_img || py >= h_img {
                    continue;
                }
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= r * r {
                    img.put_pixel(px as u32, py as u32, body);
                } else if dist_sq <= halo_r * halo_r {
                    if let Some(h_px) = halo_pixel {
                        img.put_pixel(px as u32, py as u32, h_px);
                    }
                }
            }
        }
    }
}

impl Brush for BioluminescentSpeck {
    fn paint(&self, img: &mut RgbaImage) {
        self.paint_with(img, self.color.into(), self.halo.map(Into::into));
    }
}

// =====================================================================
// KawaiiEye — fourth primitive: oval + glint, kawaii signature face piece
// =====================================================================

/// A pixel-art kawaii eye: filled oval with a single bright glint pixel in
/// one corner. Replaces ad-hoc rectangle eyes everywhere a creature needs
/// the cute-trigger Lorenz / Glocker face. Tiny on purpose — at 64×64
/// canvas the whole eye fits inside ~5×6 pixels.
///
/// Shape choice: ellipse (not rectangle) because perfect-rectangle eyes
/// read as "alien droid"; the ellipse softens the silhouette enough to
/// trigger care response while staying readable in pixel art.
///
/// Glint is a SINGLE pixel in one of the four corners of the eye box.
/// Multiple glints look wet, not kawaii. Both eyes on a creature must use
/// the SAME glint corner so they read as a coherent face — the convention
/// in this codebase is `Quadrant::TopRight`.
#[derive(Debug, Copy, Clone)]
pub struct KawaiiEye {
    pub cx: i32,
    pub cy: i32,
    /// Horizontal radius of the eye oval. Total visual width = 2*rx + 1.
    /// Typical cub eye: rx = 1 (3px wide).
    pub rx: u32,
    /// Vertical radius. Slightly larger than rx makes the eye taller than
    /// wide → reads more "wide-eyed surprise" / kawaii. Typical cub: ry = 2.
    pub ry: u32,
    /// Body of the eye — almost always Palette::NearBlack so the iris reads
    /// as solid no matter the species' palette.
    pub color: Palette,
    pub glint: Option<EyeGlint>,
}

/// Single-pixel highlight inside the eye, placed at one of the four corners.
#[derive(Debug, Copy, Clone)]
pub struct EyeGlint {
    pub corner: GlintCorner,
    pub color: Palette,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GlintCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl GlintCorner {
    /// (sign_x, sign_y): +1 = right/down, -1 = left/up. Multiplied by the
    /// eye's radii to find the corner pixel inside the oval.
    fn signs(self) -> (i32, i32) {
        match self {
            GlintCorner::TopLeft => (-1, -1),
            GlintCorner::TopRight => (1, -1),
            GlintCorner::BottomLeft => (-1, 1),
            GlintCorner::BottomRight => (1, 1),
        }
    }
}

impl KawaiiEye {
    pub const fn new(cx: i32, cy: i32, color: Palette) -> Self {
        Self { cx, cy, rx: 1, ry: 2, color, glint: None }
    }

    pub const fn with_size(mut self, rx: u32, ry: u32) -> Self {
        self.rx = rx;
        self.ry = ry;
        self
    }

    pub const fn with_glint(mut self, corner: GlintCorner, color: Palette) -> Self {
        self.glint = Some(EyeGlint { corner, color });
        self
    }

    /// Escape hatch matching the rest of the DSL: paint with a pre-resolved
    /// Rgba (e.g., after a runtime tint pass) instead of going through the
    /// Palette enum. New code should prefer the trait `paint`.
    pub fn paint_with(
        &self,
        img: &mut RgbaImage,
        body: Rgba<u8>,
        glint_pixel: Option<Rgba<u8>>,
    ) {
        let rx = self.rx.max(1) as i32;
        let ry = self.ry.max(1) as i32;
        let w = img.width() as i32;
        let h = img.height() as i32;

        // Paint the oval body first.
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                let nx = dx as f32 / rx as f32;
                let ny = dy as f32 / ry as f32;
                if nx * nx + ny * ny <= 1.0 {
                    let px = self.cx + dx;
                    let py = self.cy + dy;
                    if px >= 0 && py >= 0 && px < w && py < h {
                        img.put_pixel(px as u32, py as u32, body);
                    }
                }
            }
        }

        // Then paint the glint as a single pixel in the chosen corner. We
        // walk inward by 1px from the corner so the glint sits ON the eye,
        // not at its edge — corner pixels of the oval are usually unset.
        if let (Some(g), Some(gpx)) = (self.glint, glint_pixel) {
            let (sx, sy) = g.corner.signs();
            // Inset by 1 from the radius so the glint lands inside the body.
            let inset_x = (rx - 1).max(0);
            let inset_y = (ry - 1).max(0);
            let px = self.cx + sx * inset_x;
            let py = self.cy + sy * inset_y;
            if px >= 0 && py >= 0 && px < w && py < h {
                img.put_pixel(px as u32, py as u32, gpx);
            }
        }
    }
}

impl Brush for KawaiiEye {
    fn paint(&self, img: &mut RgbaImage) {
        self.paint_with(
            img,
            self.color.into(),
            self.glint.map(|g| g.color.into()),
        );
    }
}

// =====================================================================
// EarTuft — fifth primitive: large rounded ear with perpendicular tuft
// =====================================================================

/// A koala-style ear: outer rounded shell + inner darker patch + a row of
/// short perpendicular tuft strokes along the silhouette so the ear reads
/// as fluffy fur rather than a cardboard cutout.
///
/// Authored along an axis (`base` → `tip`) so it can be plugged into a
/// rig: pass the ear bone's `world_base` and `world_tip` and the brush
/// orients itself along that direction. Width controls the lateral
/// fluffiness independent of the bone's own length.
#[derive(Debug, Copy, Clone)]
pub struct EarTuft {
    pub base: (i32, i32),
    pub tip: (i32, i32),
    /// Half-width of the ear shell at its widest point (mid-bone). The ear
    /// tapers toward both ends.
    pub width: u32,
    /// Outer ear color (the shell).
    pub outer: Palette,
    /// Inner ear color (the darker patch in the middle of the shell).
    /// Skipped when None.
    pub inner: Option<Palette>,
    /// Perpendicular tuft strokes along the outside of the shell. Skipped
    /// when None — useful for non-furred species.
    pub tuft: Option<Palette>,
}

impl EarTuft {
    pub const fn new(base: (i32, i32), tip: (i32, i32), width: u32, outer: Palette) -> Self {
        Self { base, tip, width, outer, inner: None, tuft: None }
    }

    pub const fn with_inner(mut self, color: Palette) -> Self {
        self.inner = Some(color);
        self
    }

    pub const fn with_tuft(mut self, color: Palette) -> Self {
        self.tuft = Some(color);
        self
    }

    pub fn paint_with(
        &self,
        img: &mut RgbaImage,
        outer: Rgba<u8>,
        inner: Option<Rgba<u8>>,
        tuft: Option<Rgba<u8>>,
    ) {
        let (sx, sy) = self.base;
        let (ex, ey) = self.tip;
        let dx = (ex - sx) as f32;
        let dy = (ey - sy) as f32;
        let length = (dx * dx + dy * dy).sqrt().max(1.0);
        let perp_x = -dy / length;
        let perp_y = dx / length;
        let max_w = self.width.max(1) as f32;

        let w_img = img.width() as i32;
        let h_img = img.height() as i32;
        let steps = (length * 2.0) as i32 + 1;

        // 1. Shell + inner patch — walk the spine, paint a band whose
        // half-width follows a "leaf" profile (peaks at mid-bone, tapers
        // at the ends) so the ear reads as oval rather than a strip.
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            // Leaf profile: 4t(1-t) peaks at 1.0 at t=0.5, hits 0 at the
            // ends — a parabola scaled to the ear's max width.
            let half_w = (max_w * (4.0 * t * (1.0 - t))).round() as i32;
            let cx = sx as f32 + dx * t;
            let cy = sy as f32 + dy * t;

            for off in -half_w..=half_w {
                let px = (cx + perp_x * off as f32).round() as i32;
                let py = (cy + perp_y * off as f32).round() as i32;
                if px >= 0 && py >= 0 && px < w_img && py < h_img {
                    img.put_pixel(px as u32, py as u32, outer);
                }
            }

            // Inner patch: narrower band (60% of shell width), only in the
            // middle 60% of the ear length.
            if let Some(ic) = inner {
                if t > 0.2 && t < 0.8 {
                    let inner_half = (half_w as f32 * 0.6).round() as i32;
                    for off in -inner_half..=inner_half {
                        let px = (cx + perp_x * off as f32).round() as i32;
                        let py = (cy + perp_y * off as f32).round() as i32;
                        if px >= 0 && py >= 0 && px < w_img && py < h_img {
                            img.put_pixel(px as u32, py as u32, ic);
                        }
                    }
                }
            }
        }

        // 2. Tuft strokes — single-pixel pokes one step *outside* the
        // shell, every 2 spine samples, on both perpendicular sides.
        // These are what sells the ear as "fluffy" rather than glossy.
        if let Some(tc) = tuft {
            for i in (3..steps - 2).step_by(2) {
                let t = i as f32 / steps as f32;
                let half_w = (max_w * (4.0 * t * (1.0 - t))).round() as i32;
                let cx = sx as f32 + dx * t;
                let cy = sy as f32 + dy * t;
                for off in [half_w + 1, -(half_w + 1)] {
                    let px = (cx + perp_x * off as f32).round() as i32;
                    let py = (cy + perp_y * off as f32).round() as i32;
                    if px >= 0 && py >= 0 && px < w_img && py < h_img {
                        img.put_pixel(px as u32, py as u32, tc);
                    }
                }
            }
        }
    }
}

impl Brush for EarTuft {
    fn paint(&self, img: &mut RgbaImage) {
        self.paint_with(
            img,
            self.outer.into(),
            self.inner.map(Into::into),
            self.tuft.map(Into::into),
        );
    }
}

// =====================================================================
// RingedTail — sixth primitive: tapered banded tail along a polyline
// =====================================================================

/// A red-panda-style ringed tail painted along a polyline of joints (the
/// caller typically passes world positions of a chain of tail bones).
/// Tapers from `base_width` at the first joint to a single pixel at the
/// last; alternates between two colors at fixed intervals along the path
/// to produce the iconic ringed look.
///
/// Distinct from `TaperedTail` (which is a single straight segment): the
/// polyline lets the tail follow whatever curve the rig + soft-body
/// produces, and rings stay properly distributed along arc length even
/// when the path bends sharply.
#[derive(Debug, Clone)]
pub struct RingedTail {
    /// Spine joints in order, base → tip. Caller is responsible for
    /// providing at least 2.
    pub joints: Vec<(i32, i32)>,
    /// Half-width at the first joint (tapers to 1px at the last).
    pub base_width: u32,
    pub body: Palette,
    pub ring: Palette,
    /// Distance in pixels (along arc length) between consecutive ring-band
    /// CENTRES. Ring band starts at the cycle and is `ring_thickness` long.
    pub ring_period: u32,
    pub ring_thickness: u32,
}

impl RingedTail {
    pub fn new(joints: Vec<(i32, i32)>, base_width: u32, body: Palette, ring: Palette) -> Self {
        Self {
            joints,
            base_width,
            body,
            ring,
            ring_period: 4,
            ring_thickness: 2,
        }
    }

    pub fn with_rings(mut self, period: u32, thickness: u32) -> Self {
        self.ring_period = period;
        self.ring_thickness = thickness;
        self
    }

    pub fn paint_with(&self, img: &mut RgbaImage, body: Rgba<u8>, ring: Rgba<u8>) {
        if self.joints.len() < 2 {
            return;
        }

        // Pre-compute per-segment lengths and total arc length so taper +
        // ring phase use real distance (not naive parameter t per segment).
        let mut seg_lens = Vec::with_capacity(self.joints.len() - 1);
        let mut total = 0.0_f32;
        for w in self.joints.windows(2) {
            let dx = (w[1].0 - w[0].0) as f32;
            let dy = (w[1].1 - w[0].1) as f32;
            let l = (dx * dx + dy * dy).sqrt();
            seg_lens.push(l);
            total += l;
        }
        if total < 0.5 {
            return;
        }

        let max_w = self.base_width.max(1) as f32;
        let period = self.ring_period.max(2) as f32;
        let thickness = self.ring_thickness.max(1) as f32;
        let w_img = img.width() as i32;
        let h_img = img.height() as i32;

        let mut traveled = 0.0_f32;
        for (i, w) in self.joints.windows(2).enumerate() {
            let (sx, sy) = w[0];
            let (ex, ey) = w[1];
            let dx = (ex - sx) as f32;
            let dy = (ey - sy) as f32;
            let l = seg_lens[i].max(1e-3);
            let perp_x = -dy / l;
            let perp_y = dx / l;
            let steps = (l * 2.0) as i32 + 1;

            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let cur = traveled + l * t;
                let frac = (cur / total).clamp(0.0, 1.0);
                // Linear taper. +0.5 so 1.0 frac still renders at least a
                // single-pixel tip rather than nothing.
                let half_w = (max_w * (1.0 - frac) + 0.5).round() as i32;

                // Ring vs body color is decided by arc-length phase, so
                // the band thickness in pixels stays consistent regardless
                // of segment direction or how the spine bends.
                let phase = cur % period;
                let color = if phase < thickness { ring } else { body };

                let cx = sx as f32 + dx * t;
                let cy = sy as f32 + dy * t;
                for off in -half_w..=half_w {
                    let px = (cx + perp_x * off as f32).round() as i32;
                    let py = (cy + perp_y * off as f32).round() as i32;
                    if px >= 0 && py >= 0 && px < w_img && py < h_img {
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
            traveled += l;
        }
    }
}

impl Brush for RingedTail {
    fn paint(&self, img: &mut RgbaImage) {
        self.paint_with(img, self.body.into(), self.ring.into());
    }
}

// =====================================================================
// FusiformTail — banded tail with a thin → thick → thin profile
// =====================================================================

/// A cat-tail-style ringed tail painted along a polyline of joints. Width
/// follows a fusiform (spindle) profile: zero at the base, peaks near the
/// middle, tapers to zero at the tip. This is what distinguishes it from
/// `RingedTail`, which tapers monotonically thick-base → thin-tip.
///
/// Rings are anchored at specific joints of the polyline (every Nth joint)
/// so they ride the curvature naturally when the tail bends — the rings
/// stay parked on articulation points instead of drifting along arc-length
/// as the pose changes.
#[derive(Debug, Clone)]
pub struct FusiformTail {
    /// Spine joints in order, base → tip. Caller must supply at least 3
    /// (a fusiform profile needs an interior midpoint between endpoints).
    pub joints: Vec<(i32, i32)>,
    /// Half-width at the peak of the fusiform profile (~50% along arc).
    /// The profile reaches 0 at the base (joint 0) and tip (last joint).
    pub peak_half_width: u32,
    pub body: Palette,
    pub ring: Palette,
    /// Place a ring centred on every Nth joint starting from joint index
    /// `ring_joint_stride` — so the base (joint 0) and intermediate joints
    /// inside the stride never get a ring.
    pub ring_joint_stride: u32,
    /// Half-thickness of each ring band, in arc-length pixels. 1 means
    /// each ring covers ~3px of arc (centre ± 1).
    pub ring_half_thickness: u32,
}

impl FusiformTail {
    pub fn new(joints: Vec<(i32, i32)>, peak_half_width: u32, body: Palette, ring: Palette) -> Self {
        Self {
            joints,
            peak_half_width,
            body,
            ring,
            ring_joint_stride: 2,
            ring_half_thickness: 1,
        }
    }

    pub fn with_rings(mut self, joint_stride: u32, half_thickness: u32) -> Self {
        self.ring_joint_stride = joint_stride;
        self.ring_half_thickness = half_thickness;
        self
    }

    pub fn paint_with(&self, img: &mut RgbaImage, body: Rgba<u8>, ring: Rgba<u8>) {
        if self.joints.len() < 3 {
            return;
        }
        let n_joints = self.joints.len();

        // Per-segment lengths and per-joint cumulative arc-length so the
        // bell-curve width and ring centres both work in true arc units
        // (consistent under curved poses).
        let mut seg_lens = Vec::with_capacity(n_joints - 1);
        let mut joint_arc = Vec::with_capacity(n_joints);
        joint_arc.push(0.0_f32);
        let mut total = 0.0_f32;
        for w in self.joints.windows(2) {
            let dx = (w[1].0 - w[0].0) as f32;
            let dy = (w[1].1 - w[0].1) as f32;
            let l = (dx * dx + dy * dy).sqrt();
            seg_lens.push(l);
            total += l;
            joint_arc.push(total);
        }
        if total < 0.5 {
            return;
        }

        let stride = self.ring_joint_stride.max(1) as usize;
        let mut ring_centres: Vec<f32> = Vec::new();
        let mut j = stride;
        while j < n_joints {
            ring_centres.push(joint_arc[j]);
            j += stride;
        }

        let max_w = self.peak_half_width.max(1) as f32;
        let ring_half = self.ring_half_thickness.max(1) as f32;
        let w_img = img.width() as i32;
        let h_img = img.height() as i32;

        let mut traveled = 0.0_f32;
        for (i, w) in self.joints.windows(2).enumerate() {
            let (sx, sy) = w[0];
            let (ex, ey) = w[1];
            let dx = (ex - sx) as f32;
            let dy = (ey - sy) as f32;
            let l = seg_lens[i].max(1e-3);
            let perp_x = -dy / l;
            let perp_y = dx / l;
            let steps = (l * 2.0) as i32 + 1;

            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let cur = traveled + l * t;
                let frac = (cur / total).clamp(0.0, 1.0);
                // Parabolic bell — 1.0 at frac=0.5, 0.0 at the endpoints.
                let bell = 1.0 - (2.0 * frac - 1.0).powi(2);
                let half_w = (max_w * bell + 0.5).round() as i32;
                if half_w < 1 {
                    continue;
                }
                let is_ring = ring_centres.iter().any(|&rp| (cur - rp).abs() <= ring_half);
                let color = if is_ring { ring } else { body };

                let cx = sx as f32 + dx * t;
                let cy = sy as f32 + dy * t;
                for off in -half_w..=half_w {
                    let px = (cx + perp_x * off as f32).round() as i32;
                    let py = (cy + perp_y * off as f32).round() as i32;
                    if px >= 0 && py >= 0 && px < w_img && py < h_img {
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
            traveled += l;
        }
    }
}

impl Brush for FusiformTail {
    fn paint(&self, img: &mut RgbaImage) {
        self.paint_with(img, self.body.into(), self.ring.into());
    }
}

// =====================================================================
// FurFluff — seventh primitive: speckle cluster suggesting volumetric fur
// =====================================================================

/// A cluster of short outward-pointing strokes that read as fur volume on
/// top of a body silhouette. Use to break up a flat-color body so it
/// looks furry rather than rubber-skinned.
///
/// Painted around a centre point with a given outward angle: each stroke
/// pokes ~1-2px out from the silhouette edge in directions roughly
/// perpendicular to the body's local surface. Density and stroke length
/// are tunable. Deterministic — uses the same `hash2` as `BumpyDome` so
/// the same `seed` always produces the same fluff pattern.
///
/// Where it pokes: an annular ring of pixels around `(cx, cy)` between
/// `inner_radius` and `outer_radius`. Set `inner_radius` to roughly the
/// body radius so the fluff sits on top of the silhouette edge instead
/// of on top of the body's interior (which would look like pixel noise).
#[derive(Debug, Copy, Clone)]
pub struct FurFluff {
    pub cx: i32,
    pub cy: i32,
    /// Inner radius — fluff pixels start outside this radius. Match this
    /// to the underlying body's radius.
    pub inner_radius: u32,
    /// Outer radius — fluff pixels end inside this radius. Pick
    /// `inner_radius + 2` for short stubble, `inner_radius + 4` for
    /// fluffier cub down.
    pub outer_radius: u32,
    /// 0.0 = no pixels, 1.0 = every candidate pixel painted. Pixel-art
    /// fluff usually wants 0.30–0.55 — denser than that reads as a halo.
    pub density: f32,
    pub color: Palette,
    pub seed: u32,
}

impl FurFluff {
    pub const fn new(cx: i32, cy: i32, inner_radius: u32, outer_radius: u32, color: Palette) -> Self {
        Self {
            cx,
            cy,
            inner_radius,
            outer_radius,
            density: 0.4,
            color,
            seed: 0,
        }
    }

    pub const fn with_density(mut self, d: f32) -> Self {
        self.density = d;
        self
    }

    pub const fn with_seed(mut self, s: u32) -> Self {
        self.seed = s;
        self
    }

    pub fn paint_with(&self, img: &mut RgbaImage, color: Rgba<u8>) {
        let inner = self.inner_radius.max(0) as i32;
        let outer = self.outer_radius.max(self.inner_radius + 1) as i32;
        let inner_sq = inner * inner;
        let outer_sq = outer * outer;
        let density = self.density.clamp(0.0, 1.0);
        // Quantise density to 0..1000 so the deterministic hash can pick
        // pixels in/out without floats in the inner loop.
        let density_threshold = (density * 1000.0) as u32;
        let w = img.width() as i32;
        let h = img.height() as i32;

        for dy in -outer..=outer {
            for dx in -outer..=outer {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < inner_sq || dist_sq > outer_sq {
                    continue;
                }
                let px = self.cx + dx;
                let py = self.cy + dy;
                if px < 0 || py < 0 || px >= w || py >= h {
                    continue;
                }
                // Hash of (dx, dy, seed) decides whether to paint this
                // candidate pixel — same seed → same fluff every frame.
                let h2 = hash2(
                    ((dx + outer) as u32).wrapping_mul(73)
                        ^ ((dy + outer) as u32),
                    self.seed,
                );
                if (h2 % 1000) < density_threshold {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
}

impl Brush for FurFluff {
    fn paint(&self, img: &mut RgbaImage) {
        self.paint_with(img, self.color.into());
    }
}

// =====================================================================
// outline_silhouette — post-process: 1px dark border on every silhouette
// =====================================================================

/// Replace every opaque pixel that **belongs to a shape** and touches a
/// transparent neighbour (or the image edge) with `color`. Net effect: a
/// 1-pixel dark outline around each shape in the image, *inside* the
/// existing silhouette — the sprite doesn't grow.
///
/// "Belongs to a shape" means the pixel must have **at least one opaque
/// neighbour** in addition to having transparent ones. Isolated single
/// pixels (sparkles, speckles, fluff dots) are skipped: outlining them
/// would turn every loose pixel into a dark dot and read as noise.
///
/// Two-pass implementation: first scan the image read-only and collect
/// every pixel that needs to change, then paint them. A single-pass
/// implementation would propagate (a freshly-painted dark pixel becomes
/// "opaque" for the next pixel's neighbour check, walking the outline
/// inward across the body).
///
/// Use after every brush has finished. Per the project palette spec,
/// `Palette::DeepBrown` is the warm-species outline and `Palette::DeepTeal`
/// the cool-species outline; those are conventions, the function takes any
/// `Rgba<u8>`.
pub fn outline_silhouette(img: &mut RgbaImage, color: Rgba<u8>) {
    let w = img.width() as i32;
    let h = img.height() as i32;
    let mut to_paint: Vec<(u32, u32)> = Vec::new();
    let neighbours = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x as u32, y as u32);
            if p.0[3] == 0 {
                continue;
            }
            // Need at least one transparent neighbour (or canvas edge) to
            // qualify as "edge" — that's the existing rule.
            let mut touches_void = false;
            // AND at least one opaque neighbour to confirm we belong to a
            // shape — without this, every isolated speckle becomes a dark
            // dot.
            let mut touches_body = false;
            for (dx, dy) in neighbours {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    touches_void = true;
                    continue;
                }
                if img.get_pixel(nx as u32, ny as u32).0[3] == 0 {
                    touches_void = true;
                } else {
                    touches_body = true;
                }
            }
            if touches_void && touches_body {
                to_paint.push((x as u32, y as u32));
            }
        }
    }

    for (x, y) in to_paint {
        img.put_pixel(x, y, color);
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
    fn bumpy_dome_elliptical_paints_within_aspect() {
        // ry > rx => taller-than-wide ellipse (egg-like).
        let mut img = RgbaImage::new(64, 64);
        BumpyDome::new(32, 32, 10, Palette::Cream)
            .with_height(16)
            .with_bumpiness(0.0)
            .paint(&mut img);

        let cream: Rgba<u8> = Palette::Cream.into();

        // A point inside the y-axis but beyond the x radius should NOT be
        // painted: at (32, 32 + 14) the normalized dist = (0)² + (14/16)² ≈ 0.77 < 1, painted.
        // At (32 + 12, 32) the normalized dist = (12/10)² ≈ 1.44 > 1, should be background.
        assert_eq!(img.get_pixel(32, 32 + 14), &cream, "tall axis should reach +14");
        let outside_wide = img.get_pixel((32 + 12) as u32, 32);
        assert_ne!(outside_wide, &cream, "wide axis should not reach +12 (rx=10)");

        save_swatch(&img, "bumpy_dome_egg.png");
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

    #[test]
    fn tapered_tail_base_thicker_than_tip() {
        let mut img = RgbaImage::new(64, 64);
        TaperedTail::new((10, 32), (54, 32), 5, Palette::Teal).paint(&mut img);

        // Count painted pixels in a 3px-wide column near the base vs near the tip.
        let teal: Rgba<u8> = Palette::Teal.into();
        let count_in_col = |x: u32| -> u32 {
            (0..64).filter(|&y| img.get_pixel(x, y) == &teal).count() as u32
        };
        let near_base = count_in_col(12);
        let near_tip = count_in_col(52);
        assert!(
            near_base > near_tip,
            "base column ({near_base}) should have more painted pixels than tip column ({near_tip})"
        );

        save_swatch(&img, "tapered_tail_horizontal.png");
    }

    #[test]
    fn tapered_tail_tip_is_painted() {
        let mut img = RgbaImage::new(64, 64);
        TaperedTail::new((10, 10), (50, 50), 4, Palette::Red).paint(&mut img);
        let red: Rgba<u8> = Palette::Red.into();

        // Either the tip pixel itself or one of its 8 neighbours should carry red.
        let has_red_near_tip = (49..=51)
            .flat_map(|x| (49..=51).map(move |y| (x, y)))
            .any(|(x, y)| img.get_pixel(x, y) == &red);
        assert!(has_red_near_tip, "tip neighbourhood should contain red pixels");

        save_swatch(&img, "tapered_tail_diagonal.png");
    }

    #[test]
    fn tapered_tail_with_shadow_emits_shadow_pixels() {
        let mut img = RgbaImage::new(64, 64);
        TaperedTail::new((8, 32), (56, 38), 4, Palette::Red)
            .with_shadow(Palette::RedDark)
            .paint(&mut img);

        let shadow_px: Rgba<u8> = Palette::RedDark.into();
        let saw_shadow = img.pixels().any(|p| p == &shadow_px);
        assert!(saw_shadow, "expected at least one RedDark rim pixel");

        save_swatch(&img, "tapered_tail_shadowed.png");
    }

    #[test]
    fn bioluminescent_speck_paints_core_and_halo() {
        let mut img = RgbaImage::new(32, 32);
        BioluminescentSpeck::new(16, 16, Palette::CreamLight)
            .with_core_radius(2)
            .with_halo(Palette::CyanBright)
            .paint(&mut img);

        let core: Rgba<u8> = Palette::CreamLight.into();
        let halo: Rgba<u8> = Palette::CyanBright.into();

        // Core at (cx, cy) must be the bright color.
        assert_eq!(img.get_pixel(16, 16), &core);

        // Halo ring must produce at least one halo-color pixel.
        let saw_halo = img.pixels().any(|p| p == &halo);
        assert!(saw_halo, "expected at least one halo pixel");

        save_swatch(&img, "speck_with_halo.png");
    }

    #[test]
    fn bioluminescent_speck_radius_zero_is_single_pixel() {
        let mut img = RgbaImage::new(16, 16);
        BioluminescentSpeck::new(8, 8, Palette::CyanBright)
            .with_core_radius(0)
            .paint(&mut img);

        let cyan: Rgba<u8> = Palette::CyanBright.into();
        let painted = img.pixels().filter(|p| **p == cyan).count();
        // r=0, no halo: exactly one pixel painted.
        assert_eq!(painted, 1);

        save_swatch(&img, "speck_pinpoint.png");
    }

    #[test]
    fn kawaii_eye_paints_oval_in_eye_color() {
        let mut img = RgbaImage::new(16, 16);
        KawaiiEye::new(8, 8, Palette::NearBlack)
            .with_size(2, 3)
            .paint(&mut img);
        let dark: Rgba<u8> = Palette::NearBlack.into();
        // Centre pixel must be painted.
        assert_eq!(img.get_pixel(8, 8), &dark);
        // Pixel along long axis (y) should be painted at full vertical reach.
        assert_eq!(img.get_pixel(8, 11), &dark);
        // Pixel beyond horizontal radius should NOT be painted.
        let far_right = img.get_pixel(11, 8);
        assert_ne!(far_right, &dark);
        save_swatch(&img, "kawaii_eye_plain.png");
    }

    #[test]
    fn kawaii_eye_glint_lands_in_chosen_corner() {
        let mut img = RgbaImage::new(16, 16);
        KawaiiEye::new(8, 8, Palette::NearBlack)
            .with_size(2, 3)
            .with_glint(GlintCorner::TopRight, Palette::CreamLight)
            .paint(&mut img);

        let glint: Rgba<u8> = Palette::CreamLight.into();
        // Glint should be inside the upper-right quadrant. Inset (rx-1, ry-1)
        // = (1, 2) → pixel at (cx+1, cy-2) = (9, 6).
        assert_eq!(img.get_pixel(9, 6), &glint);
        // Top-left should NOT have a glint pixel.
        assert_ne!(img.get_pixel(7, 6), &glint);
        save_swatch(&img, "kawaii_eye_glinted.png");
    }

    #[test]
    fn kawaii_eye_min_size_does_not_crash() {
        // rx=0, ry=0 → clamped to 1 minimum, single pixel painted.
        let mut img = RgbaImage::new(8, 8);
        KawaiiEye::new(4, 4, Palette::NearBlack)
            .with_size(0, 0)
            .paint(&mut img);
        let dark: Rgba<u8> = Palette::NearBlack.into();
        assert_eq!(img.get_pixel(4, 4), &dark);
    }

    #[test]
    fn glint_corner_signs_match_compass_directions() {
        // Sanity check on the corner → sign mapping. Easy to flip a sign
        // and not notice without a test.
        assert_eq!(GlintCorner::TopLeft.signs(), (-1, -1));
        assert_eq!(GlintCorner::TopRight.signs(), (1, -1));
        assert_eq!(GlintCorner::BottomLeft.signs(), (-1, 1));
        assert_eq!(GlintCorner::BottomRight.signs(), (1, 1));
    }

    #[test]
    fn ear_tuft_paints_oval_shell_along_axis() {
        let mut img = RgbaImage::new(32, 32);
        EarTuft::new((10, 16), (28, 16), 4, Palette::Tan).paint(&mut img);
        let tan: Rgba<u8> = Palette::Tan.into();
        // Mid-bone (x=19) at the spine axis should be painted.
        assert_eq!(img.get_pixel(19, 16), &tan);
        // Mid-bone perpendicular reach should hit half_width pixels above
        // and below the axis (leaf profile peak ≈ width 4 at t=0.5).
        assert_eq!(img.get_pixel(19, 12), &tan);
        assert_eq!(img.get_pixel(19, 20), &tan);
        // Ends taper to nothing — pixel beyond shell width at the base
        // should NOT be painted.
        assert_ne!(img.get_pixel(10, 12), &tan);
        save_swatch(&img, "ear_tuft_plain.png");
    }

    #[test]
    fn ear_tuft_inner_paints_inside_shell() {
        let mut img = RgbaImage::new(32, 32);
        EarTuft::new((10, 16), (28, 16), 4, Palette::Tan)
            .with_inner(Palette::Brown)
            .paint(&mut img);
        let brown: Rgba<u8> = Palette::Brown.into();
        // Centre of ear (mid-bone) should be inner color, not outer.
        assert_eq!(img.get_pixel(19, 16), &brown);
    }

    #[test]
    fn ear_tuft_with_tuft_paints_pixels_outside_shell() {
        let mut img = RgbaImage::new(32, 32);
        EarTuft::new((10, 16), (28, 16), 4, Palette::Tan)
            .with_tuft(Palette::CreamLight)
            .paint(&mut img);
        let tuft: Rgba<u8> = Palette::CreamLight.into();
        // At least one tuft pixel should be visible in the image.
        let saw_tuft = img.pixels().any(|p| p == &tuft);
        assert!(saw_tuft, "expected at least one tuft pixel");
        save_swatch(&img, "ear_tuft_full.png");
    }

    #[test]
    fn ear_tuft_diagonal_axis_works() {
        // The brush should orient along whatever direction base→tip points.
        let mut img = RgbaImage::new(32, 32);
        EarTuft::new((8, 24), (24, 8), 3, Palette::Brown)
            .with_tuft(Palette::Cream)
            .paint(&mut img);
        let brown: Rgba<u8> = Palette::Brown.into();
        // Midpoint along diagonal ≈ (16, 16) — must be painted.
        assert_eq!(img.get_pixel(16, 16), &brown);
        save_swatch(&img, "ear_tuft_diagonal.png");
    }

    #[test]
    fn ringed_tail_paints_both_colors_along_path() {
        let mut img = RgbaImage::new(64, 32);
        let joints = vec![(8, 16), (24, 16), (40, 16), (56, 16)];
        RingedTail::new(joints, 4, Palette::Brown, Palette::OrangeBright)
            .with_rings(6, 2)
            .paint(&mut img);
        let brown: Rgba<u8> = Palette::Brown.into();
        let ring: Rgba<u8> = Palette::OrangeBright.into();
        let saw_brown = img.pixels().any(|p| p == &brown);
        let saw_ring = img.pixels().any(|p| p == &ring);
        assert!(saw_brown && saw_ring, "both body and ring colors must appear");
        save_swatch(&img, "ringed_tail_horizontal.png");
    }

    #[test]
    fn ringed_tail_tapers_to_a_point() {
        let mut img = RgbaImage::new(64, 32);
        let joints = vec![(8, 16), (56, 16)];
        RingedTail::new(joints, 5, Palette::Brown, Palette::OrangeBright).paint(&mut img);
        let brown: Rgba<u8> = Palette::Brown.into();
        let ring: Rgba<u8> = Palette::OrangeBright.into();
        let any_color = |x: u32, y: u32| {
            let p = img.get_pixel(x, y);
            p == &brown || p == &ring
        };
        // Count painted pixels in a vertical column near base vs near tip.
        let count_in_col = |x: u32| -> u32 {
            (0..32u32).filter(|&y| any_color(x, y)).count() as u32
        };
        let base_col = count_in_col(10);
        let tip_col = count_in_col(54);
        assert!(
            base_col > tip_col,
            "base column ({base_col}) should be thicker than tip ({tip_col})"
        );
    }

    #[test]
    fn ringed_tail_curved_path_paints_both_segments() {
        let mut img = RgbaImage::new(64, 64);
        // L-shaped path: along +x then turning down +y.
        let joints = vec![(8, 16), (40, 16), (40, 56)];
        RingedTail::new(joints, 3, Palette::Brown, Palette::OrangeBright)
            .with_rings(8, 3)
            .paint(&mut img);
        let brown: Rgba<u8> = Palette::Brown.into();
        let ring: Rgba<u8> = Palette::OrangeBright.into();
        let any_color = |x: u32, y: u32| {
            let p = img.get_pixel(x, y);
            p == &brown || p == &ring
        };
        // Pixel on the horizontal segment.
        assert!(any_color(20, 16));
        // Pixel on the vertical segment.
        assert!(any_color(40, 40));
        save_swatch(&img, "ringed_tail_l_shape.png");
    }

    #[test]
    fn ringed_tail_too_short_input_is_a_noop() {
        // Single joint → nothing to paint, must not panic.
        let mut img = RgbaImage::new(16, 16);
        RingedTail::new(vec![(8, 8)], 3, Palette::Brown, Palette::OrangeBright).paint(&mut img);
        // No pixels of either color should be set.
        let brown: Rgba<u8> = Palette::Brown.into();
        assert!(img.pixels().all(|p| p != &brown));
    }

    // -----------------------------------------------------------------
    // FusiformTail
    // -----------------------------------------------------------------

    /// Helper: count pixels of `color` in a vertical column at `x`. Used
    /// to measure tail thickness at different points along its arc.
    fn column_color_count(img: &RgbaImage, x: u32, color: Rgba<u8>) -> u32 {
        (0..img.height())
            .filter(|&y| *img.get_pixel(x, y) == color)
            .count() as u32
    }

    #[test]
    fn fusiform_tail_is_thicker_in_middle_than_at_endpoints() {
        // 30-px straight horizontal tail; 16 joints (15 segments × 2px each).
        let joints: Vec<(i32, i32)> =
            (0..=15).map(|i| (10 + i * 2, 30)).collect();
        let mut img = RgbaImage::new(64, 64);
        FusiformTail::new(joints, 6, Palette::Orange, Palette::OffWhite)
            .with_rings(99, 1) // stride 99 → effectively no rings, so we measure body only
            .paint(&mut img);

        let orange: Rgba<u8> = Palette::Orange.into();
        // Body extends from x=10 to x=40. Sample near the base, middle,
        // and tip; middle column must be the tallest (fusiform peak).
        let base = column_color_count(&img, 12, orange);
        let mid = column_color_count(&img, 25, orange);
        let tip = column_color_count(&img, 38, orange);
        assert!(
            mid > base && mid > tip,
            "expected fusiform peak (base={base}, mid={mid}, tip={tip})"
        );
        // And both endpoints should be thinner than the peak by a meaningful margin.
        assert!(mid >= base + 2, "middle should be clearly thicker than base");
        assert!(mid >= tip + 2,  "middle should be clearly thicker than tip");
        save_swatch(&img, "fusiform_tail_profile.png");
    }

    #[test]
    fn fusiform_tail_paints_rings_at_strided_joints() {
        // Straight horizontal tail with joints every 4px. Stride 2 →
        // rings at joints 2, 4, 6, 8 — arc-length 8, 16, 24, 32.
        let joints: Vec<(i32, i32)> =
            (0..=8).map(|i| (10 + i * 4, 30)).collect();
        let mut img = RgbaImage::new(64, 64);
        FusiformTail::new(joints, 5, Palette::Orange, Palette::OffWhite)
            .with_rings(2, 1)
            .paint(&mut img);

        let orange: Rgba<u8> = Palette::Orange.into();
        let off_white: Rgba<u8> = Palette::OffWhite.into();
        // Sample a column at each expected ring centre (joints 2/4/6/8 →
        // world x = 18, 26, 34, 42). Each must show off-white pixels.
        for x in [18, 26, 34, 42] {
            let ring_pixels = column_color_count(&img, x, off_white);
            assert!(
                ring_pixels >= 2,
                "expected ring at x={x}, found {ring_pixels} off-white pixels"
            );
        }
        // And between rings (joints 1/3/5/7 → x = 14, 22, 30, 38) the
        // dominant colour must be orange, not off-white.
        for x in [14, 22, 30, 38] {
            let body = column_color_count(&img, x, orange);
            let ring = column_color_count(&img, x, off_white);
            assert!(
                body > ring,
                "expected body-dominant at x={x}, got body={body} ring={ring}"
            );
        }
        save_swatch(&img, "fusiform_tail_rings.png");
    }

    #[test]
    fn fusiform_tail_too_short_input_is_a_noop() {
        // Fewer than 3 joints → no fusiform profile possible.
        let mut img = RgbaImage::new(16, 16);
        FusiformTail::new(vec![(2, 8), (12, 8)], 4, Palette::Orange, Palette::OffWhite)
            .paint(&mut img);
        let orange: Rgba<u8> = Palette::Orange.into();
        assert!(img.pixels().all(|p| p != &orange));
    }

    #[test]
    fn fur_fluff_paints_pixels_in_annular_ring() {
        let mut img = RgbaImage::new(32, 32);
        FurFluff::new(16, 16, 6, 9, Palette::Brown)
            .with_density(0.7)
            .with_seed(5)
            .paint(&mut img);
        let brown: Rgba<u8> = Palette::Brown.into();

        // No pixels inside the inner radius (centre region must be clean
        // so the underlying body shows through).
        for dy in -4..=4i32 {
            for dx in -4..=4i32 {
                if dx * dx + dy * dy < 25 {
                    let p = img.get_pixel((16 + dx) as u32, (16 + dy) as u32);
                    assert_ne!(p, &brown, "fluff bleeding into core at ({dx},{dy})");
                }
            }
        }
        // At least some pixels in the ring band should be painted.
        let painted = img.pixels().filter(|p| **p == brown).count();
        assert!(painted > 4, "expected several fluff pixels, got {painted}");
        save_swatch(&img, "fur_fluff_ring.png");
    }

    #[test]
    fn fur_fluff_density_zero_paints_nothing() {
        let mut img = RgbaImage::new(32, 32);
        FurFluff::new(16, 16, 4, 8, Palette::Brown)
            .with_density(0.0)
            .paint(&mut img);
        let brown: Rgba<u8> = Palette::Brown.into();
        assert_eq!(img.pixels().filter(|p| **p == brown).count(), 0);
    }

    #[test]
    fn fur_fluff_is_deterministic_per_seed() {
        let mut a = RgbaImage::new(32, 32);
        let mut b = RgbaImage::new(32, 32);
        FurFluff::new(16, 16, 5, 9, Palette::Brown)
            .with_density(0.5)
            .with_seed(11)
            .paint(&mut a);
        FurFluff::new(16, 16, 5, 9, Palette::Brown)
            .with_density(0.5)
            .with_seed(11)
            .paint(&mut b);
        // Same seed → identical pixels.
        let differ = a.pixels().zip(b.pixels()).filter(|(p, q)| p != q).count();
        assert_eq!(differ, 0, "same seed should produce same fluff pattern");
    }

    #[test]
    fn outline_silhouette_paints_edge_pixels_dark() {
        // 5×5 solid block at canvas centre. After outlining, the 1-pixel
        // outer ring of the block must be dark; the 3×3 interior must
        // keep its original colour.
        let mut img = RgbaImage::new(16, 16);
        let body: Rgba<u8> = Palette::Gold.into();
        for y in 5..10 {
            for x in 5..10 {
                img.put_pixel(x, y, body);
            }
        }
        let outline: Rgba<u8> = Palette::DeepBrown.into();
        outline_silhouette(&mut img, outline);

        // Outer ring: every edge pixel of the block is now dark.
        for x in 5..10 {
            assert_eq!(img.get_pixel(x, 5), &outline, "top edge ({x},5)");
            assert_eq!(img.get_pixel(x, 9), &outline, "bottom edge ({x},9)");
        }
        for y in 5..10 {
            assert_eq!(img.get_pixel(5, y), &outline, "left edge (5,{y})");
            assert_eq!(img.get_pixel(9, y), &outline, "right edge (9,{y})");
        }
        // Interior: 3×3 must still be Gold.
        for y in 6..9 {
            for x in 6..9 {
                assert_eq!(img.get_pixel(x, y), &body, "interior ({x},{y})");
            }
        }
        // Outside the silhouette: still transparent.
        assert_eq!(img.get_pixel(4, 4).0[3], 0);
    }

    #[test]
    fn outline_silhouette_treats_image_edge_as_transparent() {
        // A pixel on the canvas edge gets outlined even when the
        // out-of-bounds neighbour "isn't actually transparent". The pixel
        // must still belong to a shape — pair it with an adjacent body
        // pixel so it isn't isolated.
        let mut img = RgbaImage::new(8, 8);
        let body: Rgba<u8> = Palette::Gold.into();
        img.put_pixel(0, 0, body);
        img.put_pixel(1, 0, body); // adjacent → (0,0) belongs to a shape
        let outline: Rgba<u8> = Palette::DeepBrown.into();
        outline_silhouette(&mut img, outline);
        assert_eq!(img.get_pixel(0, 0), &outline);
    }

    #[test]
    fn outline_silhouette_skips_isolated_specks() {
        // Single pixel with all transparent neighbours: this is a fluff
        // speck or sparkle, NOT an edge. Outlining it would turn loose
        // FurFluff/glint pixels into dark noise dots.
        let mut img = RgbaImage::new(16, 16);
        let body: Rgba<u8> = Palette::OrangeBright.into();
        img.put_pixel(8, 8, body);
        outline_silhouette(&mut img, Palette::DeepBrown.into());
        // Speck retains its original colour.
        assert_eq!(img.get_pixel(8, 8), &body);
    }

    #[test]
    fn outline_silhouette_no_op_on_empty_image() {
        let mut img = RgbaImage::new(8, 8);
        outline_silhouette(&mut img, Palette::DeepBrown.into());
        assert!(img.pixels().all(|p| p.0[3] == 0));
    }

    #[test]
    fn outline_silhouette_does_not_propagate_through_body() {
        // Single-pass sanity: a wide block must keep its interior even
        // after outlining (a naïve in-place loop would walk inward
        // because freshly-painted dark pixels look opaque to later
        // neighbour checks).
        let mut img = RgbaImage::new(16, 16);
        let body: Rgba<u8> = Palette::Gold.into();
        for y in 4..12 {
            for x in 4..12 {
                img.put_pixel(x, y, body);
            }
        }
        outline_silhouette(&mut img, Palette::DeepBrown.into());
        // Centre pixel (way inside the 8×8 block) must still be Gold.
        assert_eq!(img.get_pixel(8, 8), &body);
        assert_eq!(img.get_pixel(7, 7), &body);
    }

    #[test]
    fn fur_fluff_seed_variation_changes_pattern() {
        let mut a = RgbaImage::new(32, 32);
        let mut b = RgbaImage::new(32, 32);
        FurFluff::new(16, 16, 5, 9, Palette::Brown)
            .with_density(0.6)
            .with_seed(1)
            .paint(&mut a);
        FurFluff::new(16, 16, 5, 9, Palette::Brown)
            .with_density(0.6)
            .with_seed(99)
            .paint(&mut b);
        let differ = a.pixels().zip(b.pixels()).filter(|(p, q)| p != q).count();
        assert!(differ > 3, "different seeds should produce different patterns ({differ} differing pixels)");
    }
}
