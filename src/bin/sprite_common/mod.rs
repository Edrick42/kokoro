//! Shared pixel-art sprite generation toolkit.
//!
//! Provides the core primitives used by all species sprite generators:
//! canvas creation, pixel setting, ellipse rasterization, flood outline,
//! color interpolation, and upscaled PNG saving.

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

/// Add a 1px outline around a filled shape.
pub fn flood_outline(img: &mut RgbaImage, shape: &Shape, color: Rgba<u8>) {
    let mut outline = Vec::new();
    let (w, h) = (img.width() as i32, img.height() as i32);
    for &(x, y) in shape {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let (nx, ny) = (x + dx, y + dy);
            if !shape.contains(&(nx, ny)) && nx >= 0 && nx < w && ny >= 0 && ny < h {
                outline.push((nx, ny));
            }
        }
    }
    px(img, &outline, color);
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
