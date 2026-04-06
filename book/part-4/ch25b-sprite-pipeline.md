# Chapter 25b: The Sprite Pipeline — Generating Game Art with Rust

> *This chapter is a practical guide to generating all the sprites Kokoro needs. No artistic talent required — just Rust and an understanding of how the modular body system works.*

## Why Generate Sprites with Code?

Game art is usually drawn by hand. But for prototyping, iteration, and procedural generation, **scripting your sprites** has huge advantages:

- **Instant iteration** — change a color, rerun, see it everywhere
- **Consistency** — every mood variant has the same proportions and style
- **Modular output** — each body part is a separate PNG, exactly what our rig system expects
- **Version control** — art is code, so it lives in git with everything else
- **One language** — no context-switching between Python and Rust. Everything is `cargo run`

The sprites in Kokoro are generated using Rust with the **`image`** crate. Each species has its own binary that outputs all the parts the game engine expects.

## The Contract: What the Game Expects

Before writing a single pixel, you need to understand what the Bevy game engine will look for. This is defined in `src/creature/species.rs` and `src/creature/spawn.rs`.

### File naming convention

```
assets/sprites/{species_dir}/{slot}_{mood_key}.png
```

### Parts per species

| Species | species_dir | Parts |
|---------|------------|-------|
| **Moluun** | `moluun` | body, ear_left, ear_right, eye_left, eye_right, mouth |
| **Pylum** | `pylum` | body, wing_left, wing_right, eye_left, eye_right, beak, tail |
| **Skael** | `skael` | body, crest_left, crest_right, eye_left, eye_right, snout, tail |
| **Nyxal** | `nyxal` | body, mantle, eye_left, eye_right, tentacle_front_left, tentacle_front_right, tentacle_back_left, tentacle_back_right |

### Mood variants

**Mood-reactive parts** (eyes + mouth/beak/snout) need one PNG per mood:
- `idle`, `hungry`, `tired`, `lonely`, `playful`, `sick`, `sleeping`

**Static parts** (body, ears/wings/crests, tail) only need `_idle`:
- `body_idle.png`, `ear_left_idle.png`, `tail_idle.png`, etc.

### Tinting

The game applies a **hue tint** (from the genome) to parts marked as `tinted` (body, ears, wings, crests, tail). These sprites should be drawn in a neutral base color — the tint will shift it in-game.

Parts marked as **not tinted** (eyes, mouth, beak, snout) display exactly as drawn. Use their final colors directly.

### Transparency

All sprites must be **RGBA PNGs** with transparent backgrounds. The game composites them on top of each other using z-depth ordering.

---

## Setting Up

Add the `image` crate to `Cargo.toml` — we only need PNG support:

```toml
[dependencies]
image = { version = "0.25", default-features = false, features = ["png"] }
```

By disabling default features, we avoid pulling in decoders for JPEG, TIFF, WebP, etc. — we only need to *write* PNGs.

The generators live in `src/bin/`, making them separate binaries:

```
src/bin/
├── sprite_common.rs               # Shared toolkit (canvas, shapes, colors)
├── generate_moluun_sprites.rs      # Moluun generator
├── generate_pylum_sprites.rs     # Pylum generator
└── generate_skael_sprites.rs       # Skael generator
```

Run them with:

```bash
cargo run --bin generate_moluun_sprites
cargo run --bin generate_pylum_sprites
cargo run --bin generate_skael_sprites
```

## The Toolkit: `sprite_common.rs`

Every sprite generator imports a shared module with core primitives. Let's build it piece by piece — this is where you'll learn how Rust handles image manipulation at the pixel level.

### Canvas and Colors

```rust
use image::{ImageBuffer, Rgba, RgbaImage};
use std::collections::HashSet;

/// Transparent pixel — every canvas starts filled with this.
pub const CLEAR: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Upscale factor — draw at pixel-art resolution, save 8x bigger.
pub const SCALE: u32 = 8;

/// A set of pixel coordinates representing a shape.
pub type Shape = HashSet<(i32, i32)>;

/// Create a new transparent RGBA canvas.
pub fn new_canvas(w: u32, h: u32) -> RgbaImage {
    ImageBuffer::from_pixel(w, h, CLEAR)
}
```

**Rust concepts**: `type` aliases, `const` declarations, and the `image` crate's `RgbaImage` type (an alias for `ImageBuffer<Rgba<u8>, Vec<u8>>`). Notice how colors are `Rgba<u8>` — a struct with an array of 4 bytes `[R, G, B, A]`.

### Safe Pixel Setting

```rust
/// Safe put_pixel — silently ignores out-of-bounds coordinates.
pub fn put(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if x >= 0 && x < img.width() as i32 && y >= 0 && y < img.height() as i32 {
        img.put_pixel(x as u32, y as u32, color);
    }
}

/// Set multiple pixels at once.
pub fn px(img: &mut RgbaImage, coords: &[(i32, i32)], color: Rgba<u8>) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    for &(x, y) in coords {
        if x >= 0 && x < w && y >= 0 && y < h {
            img.put_pixel(x as u32, y as u32, color);
        }
    }
}
```

**Why `i32` instead of `u32`?** Pixel math often goes negative (e.g., `center - radius`). Using signed integers lets us compute freely and bounds-check at the end, rather than dealing with underflow. This is a common pattern in graphics code.

### Ellipse Rasterization

The fundamental shape primitive. Every round body, eye, wing, and beak is built from ellipses:

```rust
/// Return all (x, y) positions inside an ellipse centered at (cx, cy).
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
```

**The math**: a point is inside the ellipse if `(x/rx)² + (y/ry)² ≤ 1`. We scan a bounding box and keep every integer point that satisfies this equation.

**Why return a `HashSet`?** Three reasons:
1. **Flood outline** — we need O(1) "is this pixel in the shape?" checks
2. **Shape union** — combine shapes with `a.union(&b)` or `a.extend(&b)`
3. **Overlap detection** — `if body.contains(&(x, y))` for seamless connections

### Flood Outline

The signature pixel-art look comes from 1px outlines around every shape:

```rust
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
```

For each pixel in the shape, check its 4 neighbors. If a neighbor is **not** in the shape, it's an outline pixel. Simple, but it produces clean outlines for any shape — circles, triangles, complex unions.

### Color Interpolation

For gradient shading (highlight → base → shadow):

```rust
/// Linearly interpolate between two RGBA colors.
/// t=0 returns c1, t=1 returns c2, t=0.5 is halfway.
pub fn lerp(c1: Rgba<u8>, c2: Rgba<u8>, t: f32) -> Rgba<u8> {
    let t = t.clamp(0.0, 1.0);
    Rgba([
        (c1[0] as f32 + (c2[0] as f32 - c1[0] as f32) * t) as u8,
        (c1[1] as f32 + (c2[1] as f32 - c1[1] as f32) * t) as u8,
        (c1[2] as f32 + (c2[2] as f32 - c1[2] as f32) * t) as u8,
        (c1[3] as f32 + (c2[3] as f32 - c1[3] as f32) * t) as u8,
    ])
}
```

**Rust concept**: the `as` cast. `u8 as f32` widens safely. `f32 as u8` truncates — which is fine here because we clamped `t` to `[0, 1]`, so results stay in `[0, 255]`.

### Saving with Upscale

```rust
/// Upscale image by 8x with nearest-neighbor and save as PNG.
pub fn save(img: &RgbaImage, name: &str, out_dir: &Path) {
    std::fs::create_dir_all(out_dir).expect("Failed to create output dir");
    let (w, h) = (img.width() * SCALE, img.height() * SCALE);
    let mut big = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let src = img.get_pixel(x / SCALE, y / SCALE);
            big.put_pixel(x, y, *src);
        }
    }
    big.save(out_dir.join(name)).expect("Failed to save PNG");
}
```

**Why manual upscale?** We want **nearest-neighbor** scaling — each pixel becomes an 8x8 block with no blending. This preserves the crisp pixel-art look. Bilinear or bicubic scaling would blur the edges.

### Horizontal Flip

```rust
/// Flip an image horizontally — draw left ear, get right ear for free.
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
```

---

## Species 1: Moluun (Round Mammal)

**Binary**: `src/bin/generate_moluun_sprites.rs`

**Visual identity**: Round, chunky, soft. Think hamster meets Kirby. Blue-toned with heavy outlines and gradient shading.

### Color Palette

```rust
// Sampled from the reference art (idle_00.png)
const BODY:      Rgba<u8> = Rgba([156, 232, 252, 255]); // Main body — light blue
const BODY_HI:   Rgba<u8> = Rgba([180, 240, 255, 255]); // Highlight — top of head
const BODY_SH:   Rgba<u8> = Rgba([120, 195, 225, 255]); // Shadow — sides and bottom
const BODY_SH2:  Rgba<u8> = Rgba([100, 170, 205, 255]); // Deep shadow — under body
const OUTLINE:   Rgba<u8> = Rgba([40, 45, 55, 255]);    // Dark charcoal (not pure black)
```

**Design note**: The outline is `[40, 45, 55]`, not pure black. Pure black outlines look harsh in pixel art. A dark charcoal reads as "outline" but feels softer. Each species has its own outline color that complements its palette.

### Body (52x52 pixels)

The body combines a torso ellipse with overlapping feet and arms, then applies gradient shading:

```rust
fn gen_body(dir: &Path) {
    let mut img = new_canvas(52, 52);

    // Main body — wider than tall (squat, round look)
    let body = ellipse_pixels(26.0, 23.0, 19.0, 18.0);

    // Appendages overlap — treated as one connected shape
    let feet_l = ellipse_pixels(18.0, 40.0, 7.0, 5.0);
    let feet_r = ellipse_pixels(34.0, 41.0, 7.0, 5.0);
    let arm_l  = ellipse_pixels(8.0, 29.0, 4.0, 5.0);
    let arm_r  = ellipse_pixels(44.0, 28.0, 4.0, 5.0);

    // Union all parts into one blob
    let all: Shape = body.iter()
        .chain(&feet_l).chain(&feet_r)
        .chain(&arm_l).chain(&arm_r)
        .copied().collect();
```

**Rust concept**: `.chain()` joins iterators. We chain five `HashSet` iterators into one, then `.copied()` converts `&(i32,i32)` to `(i32,i32)`, and `.collect()` builds the new `HashSet`. Zero allocations for intermediate collections.

**Vertical gradient shading**:

```rust
    for &(x, y) in &all {
        let t = (y as f32 - top) / span;     // 0 = top, 1 = bottom
        let dx = ((x as f32 - cx) / rx).abs(); // 0 = center, 1 = edge

        let mut c = if t < 0.18 {
            lerp(BODY_HI, BODY, t / 0.18)       // Highlight zone
        } else if t < 0.45 {
            BODY                                  // Main body color
        } else if t < 0.70 {
            lerp(BODY, BODY_SH, (t - 0.45) / 0.25) // Shadow transition
        } else {
            lerp(BODY_SH, BODY_SH2, (t - 0.70) / 0.30) // Deep shadow
        };

        // Edge darkening — pixels near the edge are shaded more
        if dx > 0.60 {
            c = lerp(c, BODY_SH, (dx - 0.60) / 0.40 * 0.6);
        }

        put(&mut img, x, y, c);
    }
```

Each pixel's color depends on two things: **vertical position** (light on top, dark on bottom) and **horizontal distance from center** (darker at edges). The result looks lit from above — all from simple position-based math.

### Eyes — Parameterized Builder

Eyes are the most expressive part. A single struct + builder function produces all 8 mood variants:

```rust
struct EyeParams {
    w: u32, h: u32,
    lid_rows: i32,     // How droopy (0=wide open, 4=barely open)
    pupil_y_off: i32,  // Where the pupil looks (-1=up, +2=down)
    pupil_h: i32,      // Pupil height (2=tiny, 4=large)
    sparkle: bool,     // White highlight dot = "alive" look
    tear: bool,        // Tear drop on the side
}
```

| Mood | lid_rows | pupil_y_off | pupil_h | sparkle | tear | Effect |
|------|----------|-------------|---------|---------|------|--------|
| **idle** | 3 | +1 | 3 | yes | no | Half-lidded, relaxed |
| **happy** | 1 | 0 | 4 | yes | no | Wide open, bright |
| **hungry** | 0 | -1 | 4 | yes | yes | Big, pleading, looking up |
| **tired** | 4 | +1 | 2 | no | no | Very droopy, barely open |
| **sleeping** | *custom* | — | — | no | no | Just a curved line |
| **lonely** | 2 | +2 | 3 | no | yes | Sad, looking down |
| **playful** | 0 | 0 | 4 | *star* | no | Wide, star-shaped sparkle |
| **sick** | *custom* | — | — | no | no | X-pattern instead of pupils |

**Rust concept**: `Default` trait. We implement `Default` for `EyeParams` so we can write:

```rust
eye_base(&EyeParams {
    lid_rows: 3,
    pupil_y_off: 1,
    sparkle: true,
    ..Default::default()   // Fill remaining fields with defaults
})
```

One function with a parameter struct produces 8 completely different expressions. This is the power of parameterized generation.

### Mouths

| Mood | Shape | How it's built |
|------|-------|---------------|
| idle | Cat-like "w" curve | 7 manually placed pixels |
| happy | Wide smile | Outline frame + PINK fill + WHITE teeth |
| hungry | Open O | Ellipse filled dark + PINK tongue |
| tired | Yawn oval | Larger ellipse + tongue |
| sleeping | Tiny dot | 2 pixels |
| lonely | Frown | 7 pixels in inverted curve |
| playful | Big grin | Smile + tongue sticking out (separate outlined shape) |
| sick | Wavy line | 8 pixels in wave + GREEN_SICK tinge |

---

## Species 2: Pylum (Bird)

**Binary**: `src/bin/generate_pylum_sprites.rs`

**Visual identity**: Warm, golden, egg-shaped. Wings, pointed beak, tail feathers.

### Color Palette

```rust
const BODY:      Rgba<u8> = Rgba([255, 220, 140, 255]); // Warm golden body
const OUTLINE:   Rgba<u8> = Rgba([60, 45, 30, 255]);    // Warm brown outline
const BEAK_COL:  Rgba<u8> = Rgba([255, 160, 60, 255]);  // Bright orange beak
const WING_BASE: Rgba<u8> = Rgba([240, 200, 120, 255]); // Wing root
const WING_TIP:  Rgba<u8> = Rgba([200, 160, 90, 255]);  // Wing tip (darker)
```

### Wings (22x18)

Ellipses with a directional gradient — lighter at the root (connects to body), darker at the tip:

```rust
fn gen_wing(dir: &Path, side: &str) {
    let mut img = new_canvas(22, 18);
    let cx = if side == "left" { 17.0 } else { 5.0 };
    let pts = ellipse_pixels(cx, 9.0, 12.0, 7.0);

    for &(x, y) in &pts {
        let t = if side == "left" {
            ((cx - x as f32) / 12.0).clamp(0.0, 1.0)
        } else {
            ((x as f32 - cx) / 12.0).clamp(0.0, 1.0)
        };
        put(&mut img, x, y, lerp(WING_BASE, WING_TIP, t));
    }

    flood_outline(&mut img, &pts, OUTLINE);
    save(&img, &format!("wing_{side}_idle.png"), dir);
}
```

**Note**: wings are drawn separately per side (not mirrored) because the gradient direction differs.

### Beak (12x10, 3 styles)

| Style | Used for | Shape |
|-------|---------|-------|
| `closed` | idle, tired, lonely, sick, sleeping | Solid downward triangle |
| `open` | hungry | Upper/lower parts with gap |
| `open_wide` | playful | Large gap with pink interior |

Built with loop-based triangles:

```rust
// Closed beak — triangle that narrows downward
for y in 3..7 {
    let span = (4 - (y - 3)).max(1);
    for x in (cx - span)..=(cx + span) {
        pts.insert((x, y));
    }
}
```

---

## Species 3: Skael (Reptile)

**Binary**: `src/bin/generate_skael_sprites.rs`

**Visual identity**: Cool green, elongated, angular. Predator vibes — slitted eyes, crests, scaled texture.

### Color Palette

```rust
const BODY:      Rgba<u8> = Rgba([120, 180, 130, 255]); // Cool reptile green
const OUTLINE:   Rgba<u8> = Rgba([35, 50, 40, 255]);    // Dark green-tinted outline
const IRIS:      Rgba<u8> = Rgba([200, 160, 50, 255]);   // Golden reptile iris
const SCALE_COL: Rgba<u8> = Rgba([100, 165, 115, 255]); // Scale texture lines
const CREST_COL: Rgba<u8> = Rgba([160, 100, 80, 255]);  // Horn base (warm brown)
```

### Body Texture — Scale Pattern

Skael's body has horizontal lines every 4 pixels that simulate scales:

```rust
// Scale pattern — horizontal bands across the body
for y_off in (-15..=15).step_by(4) {
    let row_y = cy as i32 + y_off;
    for &(x, y) in &body {
        if y == row_y {
            put(&mut img, x, y, SCALE_COL);
        }
    }
}
```

**Rust concept**: `.step_by(4)` on a range iterator. Produces `-15, -11, -7, -3, 1, 5, 9, 13`. Clean way to create evenly-spaced lines.

### Reptile Eyes — Vertical Slit Pupil

The most distinctive feature. A golden iris fills most of the eye, with a narrow vertical slit:

```rust
struct ReptileEye {
    slit_open: i32,  // Width of the slit (0=closed, 1=narrow, 2=wide)
    lid_rows: i32,
    sparkle: bool,
    tear: bool,
}

fn reptile_eye(p: &ReptileEye) -> RgbaImage {
    // Round eye shape
    let eye_pts = ellipse_pixels(cx, cy, 4.0, 3.0);
    px_set(&mut img, &eye_pts, EYE_WHITE);

    // Golden iris (fills most of the eye)
    let iris_pts = ellipse_pixels(cx, cy, 3.0, 3.0);
    px_set(&mut img, &iris_pts, IRIS);

    // Vertical slit — only `slit_open` pixels wide
    for dy in -3..=3 {
        for dx in (-p.slit_open / 2)..=(p.slit_open / 2) {
            if iris_pts.contains(&(cx + dx, cy + dy)) {
                put(&mut img, cx + dx, cy + dy, PUPIL);
            }
        }
    }
}
```

The `slit_open` parameter creates the mood variation:
- `0` — closed (sleeping)
- `1` — narrow slit (idle, tired — calm/drowsy)
- `2` — wide slit (hungry, playful — alert/excited)

### Snout with Fangs

Three styles: `closed` (nostril dots), `open` (teeth in gap), `grin` (wide with two fangs poking down).

---

## Species 4: Nyxal (Abyssal Squid)

**Binary**: `src/bin/generate_nyxal_sprites.rs`

**Visual identity**: Deep-sea bioluminescence. Dark purple body, glowing cyan eyes with vertical slit pupils, tapered tentacles with luminous tips. Think deep ocean meets alien intelligence.

### Color Palette

```rust
const BODY:         Rgba<u8> = Rgba([95, 60, 130, 255]);    // Deep purple
const BODY_HI:      Rgba<u8> = Rgba([120, 80, 155, 255]);   // Purple highlight
const OUTLINE:      Rgba<u8> = Rgba([25, 18, 40, 255]);     // Near-black violet
const EYE_GLOW:     Rgba<u8> = Rgba([40, 180, 200, 255]);   // Bioluminescent cyan
const EYE_GLOW_HI:  Rgba<u8> = Rgba([80, 220, 240, 255]);   // Bright glow center
const TENTACLE_TIP: Rgba<u8> = Rgba([50, 170, 180, 255]);   // Luminous tips
```

### Bioluminescent Eyes — Radial Glow

Unlike the other species, Nyxal eyes use a radial gradient from bright center to dimmer edge, creating a glow effect:

```rust
for &(x, y) in &globe {
    let dist = (((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt()) / 5.0;
    let c = lerp(EYE_GLOW_HI, EYE_GLOW, dist.min(1.0));
    put(&mut img, x, y, c);
}
```

The pupil is a vertical slit (squid-like), drawn as a single-pixel-wide column. Mood variants control whether the glow is active (idle, playful) or dim (hungry, sick, tired).

### Tentacles — Tapered with Luminous Tips

Four tentacles, each generated as a column of pixels that narrows toward the tip:

```rust
for y in 0..28 {
    let t = y as f32 / 28.0;
    let half_w = (4.0 * (1.0 - t * 0.5)) as i32; // tapers from 4px to 2px wide
    // ...
    let c = if t > 0.75 {
        lerp(TENTACLE, TENTACLE_TIP, (t - 0.75) / 0.25)  // glow at tips
    } else {
        lerp(TENTACLE, BODY_SH, t * 0.3)
    };
}
```

Front tentacles are shorter and thicker (manipulation), back tentacles are longer and thinner (locomotion). Left/right pairs are generated with `flip_h()`.

### Mantle — Dome Shape

A simple ellipse dome sitting above the body, representing the squid's mantle. Uses slightly different purple tones to distinguish it from the body.

---

## Running the Generators

```bash
cargo run --bin generate_moluun_sprites   # → assets/sprites/moluun/
cargo run --bin generate_pylum_sprites    # → assets/sprites/pylum/
cargo run --bin generate_skael_sprites    # → assets/sprites/skael/
cargo run --bin generate_nyxal_sprites    # → assets/sprites/nyxal/
```

Each binary prints the generated files with their final dimensions:

```
Generating Pylum sprites → assets/sprites/pylum
  body_idle.png (384x352)
  wing_left_idle.png (176x144)
  eye_left_idle.png (96x96)
  ...
Done!
```

After running all four generators, the game automatically detects the sprites and uses them instead of procedural mesh fallbacks. Each species generates sprites for all growth stages (egg, cub, young, adult, elder).

---

## Creating a New Species: Step by Step

### 1. Define the visual identity

- **Shape language**: Round? Angular? Elongated? Flat?
- **Color palette**: 4-5 colors (base, highlight, shadow, deep shadow, accent)
- **Outline color**: Complement the palette, not pure black
- **Unique parts**: What replaces ears/mouth? (Antennae? Tentacles? Mandibles?)
- **Eye style**: Round pupils? Slits? Compound eyes?

### 2. Create the Rust binary

Copy an existing generator. Replace colors, shapes, and part functions. The shared `sprite_common.rs` provides all the primitives you need.

```
src/bin/generate_drakel_sprites.rs
```

### 3. Register in the game

In `src/genome/mod.rs` — add the species variant.
In `src/creature/rig.rs` — add a rig function with landmark positions.
In `src/creature/species.rs` — add a template with part definitions.

### 4. Generate and test

```bash
cargo run --bin generate_drakel_sprites
cargo run --bin kokoro
```

The game loads sprites automatically. No other code changes needed.

---

## Design Principles for Pixel Art Sprites

These principles apply whether you generate sprites with code or draw them by hand:

1. **Consistent outline weight** — 1px outlines everywhere. Never 2px on one part and 0px on another.

2. **Light source from above** — Highlights at the top, shadows at the bottom. Universal in 2D games.

3. **Palette cohesion** — Each species has 4-5 body colors at different lightness levels of the same hue.

4. **Complementary outline** — Outline color = very dark version of the body's hue. Moluun: dark blue-gray. Pylum: dark brown. Skael: dark green. Nyxal: dark violet.

5. **Transparent backgrounds** — Always RGBA with `[0,0,0,0]` background.

6. **Small canvas, big upscale** — Draw at 10-50px, upscale 8x with nearest-neighbor. Chunky pixel look without anti-aliasing artifacts.

7. **Expressiveness through minimal change** — Moluun's 8 eye variants differ by 3-5 parameters. Small changes create big personality shifts.

---

## Summary

| What | Binary | Sprite count |
|------|--------|-------------|
| Moluun | `cargo run --bin generate_moluun_sprites` | 29 files (body, ears, 8 eye moods, 8 mouth moods) |
| Pylum | `cargo run --bin generate_pylum_sprites` | 25 files (body, wings, 7 eye moods, 7 beak moods, tail) |
| Skael | `cargo run --bin generate_skael_sprites` | 25 files (body, crests, 7 eye moods, 7 snout moods, tail) |
| Nyxal | `cargo run --bin generate_nyxal_sprites` | 20 files (body, mantle, 7 eye moods, 4 tentacles) |
| Shared effects | Generated by Moluun binary | 5 files (zzz, hearts, rain, stars, sparkle) |

The key insight: **sprites are an interface**. The Rust binaries produce files that follow a naming convention. The game loads files that follow that same convention. Neither side knows how the other works — they agree only on filenames, transparency, and dimensions. This is modular design in action.

And because everything is Rust, you get type safety, compile-time checks, and `cargo run` for the entire pipeline.
