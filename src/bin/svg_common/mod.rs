//! SVG-based sprite generation toolkit.
//!
//! Builds creatures as layered SVG shapes (collage style):
//! darker shapes behind, lighter shapes on top. No gradients —
//! each layer is a solid color. Light source: upper-left.
//!
//! Pipeline: Rust builds SVG → resvg renders PNG → Bevy loads PNG.

use std::path::Path;

/// A single colored shape layer.
pub struct Layer {
    /// SVG path data (M, L, C, Z commands).
    pub path: String,
    /// Fill color as hex (#RRGGBB).
    pub fill: String,
    /// Optional stroke color.
    pub stroke: Option<String>,
    /// Stroke width.
    pub stroke_width: f32,
}

/// Builds an SVG document from layered shapes and renders to PNG.
pub struct SpriteBuilder {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
}

impl SpriteBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, layers: Vec::new() }
    }

    /// Add a polygon layer (list of (x,y) points, solid fill, optional outline).
    pub fn polygon(&mut self, points: &[(f32, f32)], fill: &str, outline: Option<&str>) {
        if points.len() < 3 { return; }
        let mut path = format!("M {:.1} {:.1}", points[0].0, points[0].1);
        for p in &points[1..] {
            path.push_str(&format!(" L {:.1} {:.1}", p.0, p.1));
        }
        path.push_str(" Z");

        self.layers.push(Layer {
            path,
            fill: fill.to_string(),
            stroke: outline.map(|s| s.to_string()),
            stroke_width: 2.5,
        });
    }

    /// Add an ellipse layer.
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, fill: &str, outline: Option<&str>) {
        // Approximate ellipse as SVG ellipse element — we'll handle it specially
        let path = format!("ELLIPSE {:.1} {:.1} {:.1} {:.1}", cx, cy, rx, ry);
        self.layers.push(Layer {
            path,
            fill: fill.to_string(),
            stroke: outline.map(|s| s.to_string()),
            stroke_width: 2.5,
        });
    }

    /// Add a circle layer.
    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, fill: &str, outline: Option<&str>) {
        self.ellipse(cx, cy, r, r, fill, outline);
    }

    /// Build the SVG string.
    pub fn to_svg(&self) -> String {
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            self.width, self.height, self.width, self.height
        );

        for layer in &self.layers {
            let stroke_attr = if let Some(ref s) = layer.stroke {
                format!(r#" stroke="{}" stroke-width="{:.1}" stroke-linejoin="round""#, s, layer.stroke_width)
            } else {
                String::new()
            };

            if layer.path.starts_with("ELLIPSE") {
                // Parse ellipse params
                let parts: Vec<&str> = layer.path.split_whitespace().collect();
                if parts.len() == 5 {
                    let cx = parts[1]; let cy = parts[2];
                    let rx = parts[3]; let ry = parts[4];
                    svg.push_str(&format!(
                        r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"{}/>"#,
                        cx, cy, rx, ry, layer.fill, stroke_attr
                    ));
                }
            } else {
                svg.push_str(&format!(
                    r#"<path d="{}" fill="{}"{}/>"#,
                    layer.path, layer.fill, stroke_attr
                ));
            }
        }

        svg.push_str("</svg>");
        svg
    }

    /// Render to PNG and save.
    pub fn save_png(&self, name: &str, out_dir: &Path) {
        std::fs::create_dir_all(out_dir).expect("Failed to create output directory");

        let svg_data = self.to_svg();

        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_str(&svg_data, &opt)
            .expect("Failed to parse SVG");

        let size = tree.size();
        let w = size.width() as u32;
        let h = size.height() as u32;

        let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)
            .expect("Failed to create pixmap");

        resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

        let path = out_dir.join(name);
        pixmap.save_png(&path).expect("Failed to save PNG");
        println!("  {name} ({w}x{h})");
    }
}

// ===================================================================
// COLOR HELPERS — collage shading from a base color
// ===================================================================

/// RGB color as (r, g, b) with values 0-255.
#[derive(Clone, Copy)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    /// Darken by factor (0.0 = black, 1.0 = unchanged).
    pub fn darken(&self, factor: f32) -> Rgb {
        Rgb(
            (self.0 as f32 * factor) as u8,
            (self.1 as f32 * factor) as u8,
            (self.2 as f32 * factor) as u8,
        )
    }

    /// Lighten by factor (0.0 = unchanged, 1.0 = white).
    pub fn lighten(&self, factor: f32) -> Rgb {
        Rgb(
            (self.0 as f32 + (255.0 - self.0 as f32) * factor) as u8,
            (self.1 as f32 + (255.0 - self.1 as f32) * factor) as u8,
            (self.2 as f32 + (255.0 - self.2 as f32) * factor) as u8,
        )
    }

    /// Outline color — very dark version of the base.
    pub fn outline(&self) -> Rgb {
        self.darken(0.25)
    }
}

/// Collage shading: returns (shadow, base, highlight) colors from a base.
/// Light from upper-left.
pub fn shade_colors(base: Rgb) -> (Rgb, Rgb, Rgb) {
    let shadow = base.darken(0.55);
    let highlight = base.lighten(0.25);
    (shadow, base, highlight)
}

/// Shrink a polygon toward its center by a factor (0.0 = center point, 1.0 = unchanged).
pub fn shrink_polygon(points: &[(f32, f32)], factor: f32) -> Vec<(f32, f32)> {
    let cx: f32 = points.iter().map(|p| p.0).sum::<f32>() / points.len() as f32;
    let cy: f32 = points.iter().map(|p| p.1).sum::<f32>() / points.len() as f32;
    points.iter().map(|p| (
        cx + (p.0 - cx) * factor,
        cy + (p.1 - cy) * factor,
    )).collect()
}

/// Offset a polygon (shift all points). Used for highlight offset toward light.
pub fn offset_polygon(points: &[(f32, f32)], dx: f32, dy: f32) -> Vec<(f32, f32)> {
    points.iter().map(|p| (p.0 + dx, p.1 + dy)).collect()
}

/// Add a solid-color body with outline.
pub fn solid_body(
    builder: &mut SpriteBuilder,
    points: &[(f32, f32)],
    color: Rgb,
    outline: bool,
) {
    let outline_col = if outline { Some(color.outline().hex()) } else { None };
    builder.polygon(points, &color.hex(), outline_col.as_deref());
}
