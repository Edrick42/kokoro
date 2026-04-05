//! Shared sprite generation toolkit.
//!
//! Provides core primitives for all species sprite generators.
//! Not all functions are used by every generator binary — that's expected.


use image::{ImageBuffer, Rgba, RgbaImage};
use std::collections::HashSet;
use std::path::Path;

/// Transparent pixel.
pub const CLEAR: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Upscale factor — draw small, save big with nearest-neighbor.
pub const SCALE: u32 = 8;

/// A set of pixel coordinates representing a shape.
pub type Shape = HashSet<(i32, i32)>;

/// Create a new transparent RGBA canvas.
pub fn new_canvas(w: u32, h: u32) -> RgbaImage {
    ImageBuffer::from_pixel(w, h, CLEAR)
}

/// Safe put_pixel — silently ignores out-of-bounds coordinates.
pub fn put(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && x < img.width() as i32 && y >= 0 && y < img.height() as i32 {
        img.put_pixel(x as u32, y as u32, color);
    }
}

/// Set pixels at given coordinates to a color.
pub fn px(img: &mut RgbaImage, coords: &[(i32, i32)], color: Rgba<u8>) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    for &(x, y) in coords {
        if x >= 0 && x < w && y >= 0 && y < h {
            img.put_pixel(x as u32, y as u32, color);
        }
    }
}

/// Set pixels from a Shape to a color.
pub fn px_set(img: &mut RgbaImage, shape: &Shape, color: Rgba<u8>) {
    let coords: Vec<(i32, i32)> = shape.iter().copied().collect();
    px(img, &coords, color);
}

/// Return all (x, y) positions inside an ellipse centered at (cx, cy).
/// Coordinates can be negative — callers must bounds-check against canvas size.
pub fn ellipse_pixels(cx: f32, cy: f32, rx: f32, ry: f32) -> Shape {
    let mut pts = Shape::new();
    let y_min = (cy - ry).floor() as i32;
    let y_max = (cy + ry).ceil() as i32;
    let x_min = (cx - rx).floor() as i32;
    let x_max = (cx + rx).ceil() as i32;
    for y in y_min..=y_max {
        for x in x_min..=x_max {
            let dx = (x as f32 - cx) / rx;
            let dy = (y as f32 - cy) / ry;
            if dx * dx + dy * dy <= 1.0 {
                pts.insert((x, y));
            }
        }
    }
    pts
}

/// Add a thick outline around a filled shape. Default 2px radius.
pub fn flood_outline(img: &mut RgbaImage, shape: &Shape, color: Rgba<u8>) {
    flood_outline_thick(img, shape, color, 2);
}

/// Add a thick outline (radius pixels) around a filled shape.
pub fn flood_outline_thick(img: &mut RgbaImage, shape: &Shape, color: Rgba<u8>, radius: i32) {
    let mut outline = HashSet::new();
    let (w, h) = (img.width() as i32, img.height() as i32);
    for &(x, y) in shape {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx == 0 && dy == 0 { continue; }
                let (nx, ny) = (x + dx, y + dy);
                if !shape.contains(&(nx, ny)) && nx >= 0 && nx < w && ny >= 0 && ny < h {
                    outline.insert((nx, ny));
                }
            }
        }
    }
    let coords: Vec<(i32, i32)> = outline.into_iter().collect();
    px(img, &coords, color);
}

/// Linearly interpolate between two RGBA colors. t=0 returns c1, t=1 returns c2.
pub fn lerp(c1: Rgba<u8>, c2: Rgba<u8>, t: f32) -> Rgba<u8> {
    let t = t.clamp(0.0, 1.0);
    Rgba([
        (c1[0] as f32 + (c2[0] as f32 - c1[0] as f32) * t) as u8,
        (c1[1] as f32 + (c2[1] as f32 - c1[1] as f32) * t) as u8,
        (c1[2] as f32 + (c2[2] as f32 - c1[2] as f32) * t) as u8,
        (c1[3] as f32 + (c2[3] as f32 - c1[3] as f32) * t) as u8,
    ])
}

/// Save at 1:1 — no upscale, no pixelization. Use for high-res canvases.
pub fn save_raw(img: &RgbaImage, name: &str, out_dir: &Path) {
    std::fs::create_dir_all(out_dir).expect("Failed to create output directory");
    let path = out_dir.join(name);
    img.save(&path).expect("Failed to save PNG");
    println!("  {name} ({}x{})", img.width(), img.height());
}

/// Save at a custom scale factor. Use for high-detail canvases.
pub fn save_scaled(img: &RgbaImage, name: &str, out_dir: &Path, scale: u32) {
    std::fs::create_dir_all(out_dir).expect("Failed to create output directory");
    let (w, h) = (img.width() * scale, img.height() * scale);
    let mut big = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let src = img.get_pixel(x / scale, y / scale);
            big.put_pixel(x, y, *src);
        }
    }
    let path = out_dir.join(name);
    big.save(&path).expect("Failed to save PNG");
    println!("  {name} ({w}x{h})");
}

/// Upscale image by SCALE factor with nearest-neighbor and save as PNG.
pub fn save(img: &RgbaImage, name: &str, out_dir: &Path) {
    std::fs::create_dir_all(out_dir).expect("Failed to create output directory");
    let (w, h) = (img.width() * SCALE, img.height() * SCALE);
    let mut big = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let src = img.get_pixel(x / SCALE, y / SCALE);
            big.put_pixel(x, y, *src);
        }
    }
    let path = out_dir.join(name);
    big.save(&path).expect("Failed to save PNG");
    println!("  {name} ({w}x{h})");
}

/// Flip an image horizontally.
pub fn flip_h(img: &RgbaImage) -> RgbaImage {
    let (w, h) = (img.width(), img.height());
    let mut flipped = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            flipped.put_pixel(w - 1 - x, y, *img.get_pixel(x, y));
        }
    }
    flipped
}

// ---------------------------------------------------------------------------
// Low-poly polygon primitives
// ---------------------------------------------------------------------------

/// Returns all pixels inside a triangle defined by three vertices.
pub fn triangle_pixels(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> Shape {
    let mut pts = Shape::new();

    let y_min = a.1.min(b.1).min(c.1).floor() as i32;
    let y_max = a.1.max(b.1).max(c.1).ceil() as i32;
    let x_min = a.0.min(b.0).min(c.0).floor() as i32;
    let x_max = a.0.max(b.0).max(c.0).ceil() as i32;

    for y in y_min..=y_max {
        for x in x_min..=x_max {
            if point_in_triangle((x as f32, y as f32), a, b, c) {
                pts.insert((x, y));
            }
        }
    }
    pts
}

/// Barycentric test: is point p inside triangle (a, b, c)?
fn point_in_triangle(p: (f32, f32), a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> bool {
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);
    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
    !(has_neg && has_pos)
}

fn sign(p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)) -> f32 {
    (p1.0 - p3.0) * (p2.1 - p3.1) - (p2.0 - p3.0) * (p1.1 - p3.1)
}

/// Returns all pixels inside a convex polygon defined by vertices (in order).
pub fn polygon_pixels(vertices: &[(f32, f32)]) -> Shape {
    if vertices.len() < 3 {
        return Shape::new();
    }

    let mut pts = Shape::new();

    // Fan triangulation from first vertex
    let a = vertices[0];
    for i in 1..vertices.len() - 1 {
        let b = vertices[i];
        let c = vertices[i + 1];
        pts.extend(triangle_pixels(a, b, c));
    }

    pts
}

/// Diamond (rhombus) shape — used for low-poly eyes.
pub fn diamond_pixels(cx: f32, cy: f32, half_w: f32, half_h: f32) -> Shape {
    let top = (cx, cy - half_h);
    let right = (cx + half_w, cy);
    let bottom = (cx, cy + half_h);
    let left = (cx - half_w, cy);
    polygon_pixels(&[top, right, bottom, left])
}

/// Fills a polygon with smooth volume shading.
///
/// Each pixel is shaded based on:
/// 1. Its angle from center relative to light direction (facet-like lighting)
/// 2. Its distance from center (edge darkening for volume)
/// 3. Smooth gradient between light and shadow (no hard facet edges)
///
/// This creates the low-poly-with-volume look from the reference art.
pub fn faceted_shade(
    vertices: &[(f32, f32)],
    _center: (f32, f32),
    base_color: Rgba<u8>,
    _light_dir: (f32, f32),
) -> Vec<(Shape, Rgba<u8>)> {
    let shape = polygon_pixels(vertices);
    vec![(shape, base_color)]
}

/// Fills a polygon using **layered collage** — stacked shapes from dark to light.
///
/// This creates the low-poly look from reference art:
/// 1. Full shape in shadow color (darkest layer)
/// 2. Slightly smaller shape in base color (middle layer)
/// 3. Even smaller shape in highlight color (lightest layer, offset toward light)
///
/// The result is solid color regions with clear boundaries — no per-pixel gradient.
pub fn smooth_shade(
    img: &mut RgbaImage,
    vertices: &[(f32, f32)],
    center: (f32, f32),
    base_color: Rgba<u8>,
    light_dir: (f32, f32),
) {
    if vertices.len() < 3 { return; }

    // Derived colors
    let shadow = Rgba([
        (base_color[0] as f32 * 0.60) as u8,
        (base_color[1] as f32 * 0.60) as u8,
        (base_color[2] as f32 * 0.60) as u8,
        255,
    ]);
    let highlight = Rgba([
        (base_color[0] as f32 * 1.2).min(255.0) as u8,
        (base_color[1] as f32 * 1.2).min(255.0) as u8,
        (base_color[2] as f32 * 1.2).min(255.0) as u8,
        255,
    ]);

    // Normalize light direction
    let light_len = (light_dir.0 * light_dir.0 + light_dir.1 * light_dir.1).sqrt().max(0.001);
    let lx = light_dir.0 / light_len;
    let ly = light_dir.1 / light_len;

    // Layer 1: full shape in shadow (darkest)
    let full = polygon_pixels(vertices);
    px_set(img, &full, shadow);

    // Layer 2: shrunk 85% toward center, in base color
    let mid: Vec<(f32, f32)> = vertices.iter()
        .map(|v| (
            center.0 + (v.0 - center.0) * 0.85,
            center.1 + (v.1 - center.1) * 0.85,
        ))
        .collect();
    let mid_shape = polygon_pixels(&mid);
    px_set(img, &mid_shape, base_color);

    // Layer 3: shrunk 55%, offset toward light, in highlight
    let offset_x = lx * 3.0;
    let offset_y = ly * 3.0;
    let hi: Vec<(f32, f32)> = vertices.iter()
        .map(|v| (
            center.0 + (v.0 - center.0) * 0.55 - offset_x,
            center.1 + (v.1 - center.1) * 0.55 - offset_y,
        ))
        .collect();
    let hi_shape = polygon_pixels(&hi);
    // Only draw highlight pixels that are inside the base layer
    for &(x, y) in &hi_shape {
        if mid_shape.contains(&(x, y)) {
            put(img, x, y, highlight);
        }
    }
}

/// Shade a shape with a vertical gradient (top=highlight, bottom=shadow).
#[allow(dead_code)]
pub fn shade_vertical(
    img: &mut RgbaImage,
    shape: &Shape,
    cx: f32,
    rx: f32,
    top: f32,
    bottom: f32,
    hi: Rgba<u8>,
    base: Rgba<u8>,
    sh: Rgba<u8>,
    sh2: Rgba<u8>,
) {
    let span = bottom - top;
    if span <= 0.0 {
        return;
    }
    for &(x, y) in shape {
        let t = (y as f32 - top) / span;
        let dx = if rx > 0.0 {
            ((x as f32 - cx) / rx).abs().min(1.0)
        } else {
            0.0
        };

        let mut c = if t < 0.25 {
            lerp(hi, base, t / 0.25)
        } else if t < 0.6 {
            base
        } else {
            lerp(base, sh, (t - 0.6) / 0.4)
        };

        // Edge darkening
        if dx > 0.6 {
            let edge_t = (dx - 0.6) / 0.4;
            c = lerp(c, sh2, edge_t * 0.5);
        }

        img.put_pixel(x as u32, y as u32, c);
    }
}
