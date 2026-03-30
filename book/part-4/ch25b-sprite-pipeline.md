# Chapter 25b: The Sprite Pipeline — Generating Game Art with Python

> *This chapter is a practical guide to generating all the sprites Kokoro needs. No artistic talent required — just Python and an understanding of how the modular body system works.*

## Why Generate Sprites with Code?

Game art is usually drawn by hand. But for prototyping, iteration, and procedural generation, **scripting your sprites** has huge advantages:

- **Instant iteration** — change a color, rerun, see it everywhere
- **Consistency** — every mood variant has the same proportions and style
- **Modular output** — each body part is a separate PNG, exactly what our rig system expects
- **Version control** — art is code, so it lives in git with everything else

The sprites in Kokoro are generated using Python with the **Pillow** (PIL) library. Each species has its own script that outputs all the parts the game engine expects.

## The Contract: What the Game Expects

Before writing a single pixel, you need to understand what the Bevy game engine will look for. This is defined in `src/creature/species.rs` and `src/creature/spawn.rs`.

### File naming convention

```
assets/sprites/{species_dir}/{slot}_{mood_key}.png
```

### Parts per species

| Species | species_dir | Parts |
|---------|------------|-------|
| **Marumi** | `marumi` | body, ear_left, ear_right, eye_left, eye_right, mouth |
| **Tsubasa** | `tsubasa` | body, wing_left, wing_right, eye_left, eye_right, beak, tail |
| **Uroko** | `uroko` | body, crest_left, crest_right, eye_left, eye_right, snout, tail |

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

## The Toolkit

Every script uses the same minimal toolkit. You need Python 3 and Pillow:

```bash
pip install Pillow
```

### Core Helper Functions

These four functions appear in every sprite generator:

```python
from PIL import Image
import os

SCALE = 8   # Pixel art drawn small, upscaled 8x for crisp rendering
CLEAR = (0, 0, 0, 0)  # Transparent

def new_canvas(w, h):
    """Create a transparent RGBA canvas."""
    return Image.new('RGBA', (w, h), CLEAR)

def save(img, name, out_dir):
    """Upscale with nearest-neighbor (keeps pixel art crisp) and save."""
    big = img.resize((img.width * SCALE, img.height * SCALE), Image.NEAREST)
    big.save(os.path.join(out_dir, name))

def px(img, coords, color):
    """Set specific pixels to a color. The fundamental drawing operation."""
    data = img.load()
    for x, y in coords:
        if 0 <= x < img.width and 0 <= y < img.height:
            data[x, y] = color

def ellipse_pixels(cx, cy, rx, ry):
    """Return all (x, y) positions inside an ellipse. Used for every
    round shape: bodies, eyes, wings, everything."""
    pts = set()
    for y in range(int(cy - ry) - 1, int(cy + ry) + 2):
        for x in range(int(cx - rx) - 1, int(cx + rx) + 2):
            if ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 1.0:
                pts.add((x, y))
    return pts
```

### Why sets of pixels?

Every shape is stored as a `set` of `(x, y)` coordinates. This allows:

- **Flood outline**: check which pixels are adjacent to the shape but not inside it
- **Composition**: combine shapes with `body_pts | arm_pts` (set union)
- **Overlap detection**: `if (x, y) in body_pts` for seamless connections
- **Color per pixel**: iterate over the set and assign colors based on position

```python
def flood_outline(img, shape_pixels, outline_color):
    """Add a 1px outline around any shape. The secret to that
    pixel art 'chunky' look."""
    filled = set(shape_pixels)
    outline = set()
    for x, y in filled:
        for dx, dy in [(-1,0),(1,0),(0,-1),(0,1)]:
            nx, ny = x + dx, y + dy
            if (nx, ny) not in filled:
                outline.add((nx, ny))
    px(img, outline, outline_color)
```

### Color interpolation

For shading (gradients from highlight to shadow):

```python
def _lerp(c1, c2, t):
    """Linearly interpolate between two RGBA colors.
    t=0 returns c1, t=1 returns c2, t=0.5 is halfway."""
    return tuple(int(c1[i] + (c2[i] - c1[i]) * t) for i in range(4))
```

---

## Species 1: Marumi (Round Mammal)

**Script**: `scripts/generate_sprites.py`

**Visual identity**: Round, chunky, soft. Think hamster meets Kirby. Blue-toned with heavy outlines and gradient shading.

### Color Palette

```python
# Sampled from the reference art (idle_00.png)
BODY       = (156, 232, 252, 255)   # Main body — light blue
BODY_HI    = (180, 240, 255, 255)   # Highlight — top of head
BODY_SH    = (120, 195, 225, 255)   # Shadow — sides and bottom
BODY_SH2   = (100, 170, 205, 255)   # Deep shadow — under body
EAR_INNER  = (130, 200, 230, 255)   # Inner ear — slightly darker
OUTLINE    = (40, 45, 55, 255)      # Dark charcoal (not pure black)
PUPIL      = (50, 50, 65, 255)      # Eye pupil
EYE_WHITE  = (245, 248, 255, 255)   # Slightly blue-tinted white
PINK       = (240, 160, 165, 255)   # Tongue, blush
TEAR       = (140, 195, 250, 255)   # Tear drops
GREEN_SICK = (165, 215, 145, 255)   # Sick tinge
```

**Design note**: The outline is `(40, 45, 55)`, not pure black. Pure black outlines look harsh in pixel art. A dark charcoal reads as "outline" but feels softer.

### Body (52x52 pixels)

The body is the most complex part. It combines:
- A main torso ellipse
- Two feet overlapping the bottom
- Two arms overlapping the sides
- Gradient shading from highlight (top) to deep shadow (bottom)
- Edge darkening for a 3D feel
- A specular highlight near the top center

```python
def gen_body():
    W, H = 52, 52
    img = new_canvas(W, H)

    # Main body — wider than tall (squat, round look)
    cx, cy, rx, ry = 26, 23, 19, 18
    body_pts = ellipse_pixels(cx, cy, rx, ry)

    # Appendages overlap the body — treated as one connected shape
    foot_l = ellipse_pixels(18, 40, 7, 5)
    foot_r = ellipse_pixels(34, 41, 7, 5)
    arm_l  = ellipse_pixels(8, 29, 4, 5)
    arm_r  = ellipse_pixels(44, 28, 4, 5)

    # Union of all parts = one connected "blob"
    all_parts = body_pts | foot_l | foot_r | arm_l | arm_r
```

**Key technique — vertical gradient shading**:

```python
    for x, y in all_parts:
        t = (y - top) / span     # 0 = top of body, 1 = bottom
        dx = abs(x - cx) / rx    # 0 = center, 1 = edge

        # Smooth 4-zone gradient
        if t < 0.18:
            c = _lerp(BODY_HI, BODY, t / 0.18)      # Highlight zone
        elif t < 0.45:
            c = BODY                                   # Main body color
        elif t < 0.70:
            c = _lerp(BODY, BODY_SH, (t - 0.45) / 0.25)  # Shadow transition
        else:
            c = _lerp(BODY_SH, BODY_SH2, (t - 0.70) / 0.30)  # Deep shadow

        # Edge darkening — pixels near the edge are shaded more
        if dx > 0.60:
            edge_t = (dx - 0.60) / 0.40
            c = _lerp(c, BODY_SH, edge_t * 0.6)
```

The result is a sprite that looks lit from above — bright top, dark bottom, dark edges — all from a simple position-based algorithm.

### Ears (14x14 pixels)

Simple shaded ellipses with a darker inner ear:

```python
def gen_ears():
    img = new_canvas(14, 14)
    ear = ellipse_pixels(7, 7, 5, 6)       # Outer ear
    inner = ellipse_pixels(7, 8, 3, 3)      # Inner ear (darker)

    # Shade the ear with the same vertical gradient technique
    for x, y in ear:
        t = (y - 1) / 12
        c = BODY_HI if t < 0.3 else (BODY if t < 0.6 else BODY_SH)
        img.load()[x, y] = c

    # Inner ear overlay
    for x, y in inner:
        img.load()[x, y] = EAR_INNER

    flood_outline(img, ear, OUTLINE)
    save(img, 'ear_left_idle.png')

    # Mirror horizontally for the right ear
    right = img.transpose(Image.FLIP_LEFT_RIGHT)
    save(right, 'ear_right_idle.png')
```

**Key technique — mirroring**: draw the left part, flip it for the right. Saves work and guarantees symmetry.

### Eyes (13x10 pixels, 8 mood variants)

Eyes are the most expressive part. A single builder function with parameters creates all moods:

```python
def _eye_base(w, h, lid_rows=1, pupil_y_off=0, pupil_h=4,
              sparkle=False, tear=False):
    """
    Parameters that create personality:
    - lid_rows: how many rows the upper eyelid covers (0=wide open, 4=barely open)
    - pupil_y_off: where the pupil looks (-1=up, +2=down)
    - pupil_h: pupil height (2=tiny, 4=large)
    - sparkle: bright highlight in the pupil (makes eyes look "alive")
    - tear: adds a tear drop on the side
    """
```

Each mood tunes these parameters:

| Mood | lid_rows | pupil_y_off | pupil_h | sparkle | tear | Effect |
|------|----------|-------------|---------|---------|------|--------|
| **idle** | 3 | +1 | 3 | yes | no | Half-lidded, relaxed — signature Marumi look |
| **happy** | 1 | 0 | 4 | yes | no | Wide open, bright |
| **hungry** | 0 | -1 | 4 | yes | yes | Big, pleading, looking up with tear |
| **tired** | 4 | +1 | 2 | no | no | Very droopy, barely open |
| **sleeping** | *custom* | — | — | no | no | Just a curved line (eyes closed) |
| **lonely** | 2 | +2 | 3 | no | yes | Sad, looking down, with tear |
| **playful** | 0 | 0 | 4 | *star* | no | Wide, excited, star-shaped sparkle |
| **sick** | *custom* | — | — | no | no | X-pattern instead of pupils |

**Key insight**: one function with 5 parameters produces 8 completely different expressions. This is the power of parameterized generation — instead of drawing 8 eye sprites from scratch, you tune numbers.

### Mouths (12x6 to 14x10 pixels, 8 variants)

Mouths are drawn individually because their shapes differ too much:

| Mood | Shape | Key detail |
|------|-------|-----------|
| idle | Cat-like "w" curve | Signature Kobara look |
| happy | Wide smile | White teeth visible |
| hungry | Open O | Pink tongue at bottom |
| tired | Yawn oval | Tongue visible |
| sleeping | Tiny dot | Just 2 pixels |
| lonely | Inverted curve (frown) | Simple but effective |
| playful | Big grin | Tongue sticking out the side |
| sick | Wavy line | Green-sick tinge |

**Example — the hungry mouth:**

```python
def mouth_hungry():
    """Open O shape, wanting food."""
    img = new_canvas(10, 10)
    pts = ellipse_pixels(5, 5, 3, 3)
    for x, y in pts:
        img.load()[x, y] = (70, 35, 40, 255)  # Dark mouth interior
    flood_outline(img, pts, OUTLINE)
    px(img, [(4, 6), (5, 6), (6, 6)], PINK)   # Tongue hint
    return img
```

### Shared Effects

Effects float near the creature and aren't tied to the rig:

```python
# ZZZ — three Z's of increasing size, ascending diagonally
# Hearts — small heart + large heart, red/pink
# Rain cloud — gray cloud with blue tear drops falling
# Dizzy stars — four yellow 4-pointed stars
# Sparkle — six small light-yellow sparkle shapes
```

These go in `assets/sprites/shared/effects/` and are used by the `EffectsPlugin`.

---

## Species 2: Tsubasa (Bird)

**Script**: `scripts/generate_tsubasa_sprites.py`

**Visual identity**: Warm, golden, egg-shaped. Bird-like with wings, a pointed beak, and tail feathers.

### Color Palette

```python
BODY       = (255, 220, 140, 255)   # Warm golden body
BODY_HI    = (255, 235, 170, 255)   # Lighter highlight
BODY_SH    = (230, 190, 110, 255)   # Shadow
BODY_SH2   = (210, 170, 95, 255)    # Deep shadow
WING_BASE  = (240, 200, 120, 255)   # Wing root (close to body)
WING_TIP   = (200, 160, 90, 255)    # Wing tip (darker)
OUTLINE    = (60, 45, 30, 255)      # Warm brown outline (not blue like Marumi)
BEAK_COL   = (255, 160, 60, 255)    # Bright orange beak
BEAK_DARK  = (220, 130, 40, 255)    # Inside of open beak
```

**Design note**: Each species has its own outline color that complements its palette. Marumi uses cool charcoal `(40,45,55)`, Tsubasa uses warm brown `(60,45,30)`.

### Unique Parts

**Wings (22x18)**: Ellipses with a gradient from body-color at the root to darker at the tip. Left and right are drawn separately (not mirrored) because the gradient direction differs.

```python
def gen_wing(side):
    img = new_canvas(22, 18)
    if side == 'left':
        cx = W - 5    # Wing root on the right (connects to body)
        for x, y in pts:
            t = (cx - x) / 12.0           # Distance from root
            c = _lerp(WING_BASE, WING_TIP, t)  # Darker toward tip
    else:
        cx = 5        # Wing root on the left
        for x, y in pts:
            t = (x - cx) / 12.0
            c = _lerp(WING_BASE, WING_TIP, t)
```

**Beak (12x10)**: Replaces Marumi's mouth. Three styles:

| Style | Used for | Shape |
|-------|---------|-------|
| `closed` | idle, tired, lonely, sick, sleeping | Solid downward triangle |
| `open` | hungry | Upper/lower parts with gap and teeth |
| `open_wide` | playful | Large gap with pink interior visible |

```python
# Closed beak — simple triangle
for y in range(3, 7):
    span = max(1, 4 - (y - 3))        # Gets narrower toward the bottom
    for x in range(cx - span, cx + span + 1):
        pts.add((x, y))
```

**Tail (16x18)**: Three feathered shapes (center + two offset), each tapering toward the tip. Different shading per feather for depth.

### Eyes

Same concept as Marumi but **rounder** (4x4 radius instead of rectangular). Uses a circular pupil instead of a square block. Mood variants are simpler — mainly controlled by `lid_rows` and `sparkle/tear` flags.

---

## Species 3: Uroko (Reptile)

**Script**: `scripts/generate_uroko_sprites.py`

**Visual identity**: Cool green, elongated, angular. Predator vibes — slitted eyes, crests/horns, scaled texture.

### Color Palette

```python
BODY       = (120, 180, 130, 255)   # Cool reptile green
BODY_HI    = (150, 210, 155, 255)   # Lighter green highlight
BODY_SH    = (90, 150, 105, 255)    # Shadow
BODY_SH2   = (70, 125, 85, 255)    # Deep shadow
SCALE_COL  = (100, 165, 115, 255)  # Scale texture lines
CREST_COL  = (160, 100, 80, 255)   # Horn base (warm brown)
CREST_TIP  = (190, 120, 90, 255)   # Horn tip (lighter)
OUTLINE    = (35, 50, 40, 255)     # Dark green-tinted outline
IRIS       = (200, 160, 50, 255)   # Golden reptile iris
```

### Unique Parts

**Body (42x50)**: Taller ellipse (rx=16, ry=20) with **horizontal scale lines** every 4 pixels for reptile texture:

```python
# Scale pattern — horizontal bands across the body
for y_off in range(-15, 16, 4):
    row_y = cy + y_off
    for x, y in body:
        if y == row_y:
            px(img, [(x, y)], SCALE_COL)
```

**Crests/Horns (10x20)**: Triangular shapes pointing upward, with a gradient from dark base to light tip. Not mirrored — each side is drawn with the triangle leaning toward its respective direction.

**Eyes (12x12)**: The most distinctive feature. Uses a **vertical slit pupil** over a golden iris:

```python
def _reptile_eye(slit_open=2, lid_rows=0, sparkle=False, tear=False):
    # Round eye shape
    eye_pts = ellipse_pixels(cx, cy, 4, 3)
    px(img, eye_pts, EYE_WHITE)

    # Golden iris (fills most of the eye)
    iris_pts = ellipse_pixels(cx, cy, 3, 3)
    px(img, iris_pts, IRIS)

    # Vertical slit pupil — width varies by mood
    for dy in range(-3, 4):
        for dx in range(-slit_open//2, slit_open//2 + 1):
            if (cx + dx, cy + dy) in iris_pts:
                px(img, [(cx + dx, cy + dy)], PUPIL)
```

The `slit_open` parameter controls how wide the slit is:
- `0` — closed (sleeping)
- `1` — narrow slit (idle, tired, lonely)
- `2` — wide slit (hungry, playful — alert/excited)

**Snout (16x10)**: Replaces mouth. Three styles:
- `closed` — flat shape with two nostril dots
- `open` — upper/lower jaw with white teeth in the gap
- `grin` — wide shape with a grin line and two fangs poking down

**Tail (14x24)**: Thick, tapering, with horizontal scale bands every 3 pixels (same technique as body).

---

## Running the Generators

```bash
# Generate all species
python3 scripts/generate_sprites.py           # Marumi → assets/sprites/marumi/
python3 scripts/generate_tsubasa_sprites.py   # Tsubasa → assets/sprites/tsubasa/
python3 scripts/generate_uroko_sprites.py     # Uroko → assets/sprites/uroko/
```

Each script prints the generated files and their final dimensions:

```
Generating Tsubasa sprites → assets/sprites/tsubasa
  body_idle.png (384x352)
  wing_left_idle.png (176x144)
  wing_right_idle.png (176x144)
  eye_left_idle.png (96x96)
  ...
```

After running all three, the game will automatically detect the sprites and use them instead of procedural mesh fallbacks (the check happens 2 seconds after startup in `check_sprite_fallback`).

---

## Creating a New Species: Step by Step

Want to add a fourth species? Here's the complete process:

### 1. Define the visual identity

Before writing code, decide:
- **Shape language**: Round? Angular? Elongated? Flat?
- **Color palette**: 4-5 colors (base, highlight, shadow, deep shadow, accent)
- **Outline color**: Should complement the palette, not pure black
- **Unique parts**: What replaces ears/mouth? (Antennae? Tentacles? Mandibles?)
- **Eye style**: Round pupils? Slits? Compound eyes?

### 2. Create the Python script

Copy an existing script as a template. Replace:
- `OUT_DIR` path
- Color constants
- Body proportions
- Unique part generators

### 3. Register in Rust

In `src/genome/mod.rs`:
```rust
pub enum Species {
    Marumi, Tsubasa, Uroko,
    Drakel,  // New species
}
```

In `src/creature/rig.rs` — add a rig function:
```rust
pub fn drakel_rig() -> BodyRig { /* landmark positions */ }
```

In `src/creature/species.rs` — add a template:
```rust
fn drakel_template() -> SpeciesTemplate { /* part definitions */ }
```

Register both in `SpeciesRegistry::new()`.

### 4. Generate and test

```bash
python3 scripts/generate_drakel_sprites.py
cargo run
```

The game loads sprites automatically. No other code changes needed.

---

## Design Principles for Pixel Art Sprites

These principles apply whether you generate sprites with code, draw them by hand, or use AI:

1. **Consistent outline weight** — 1px outlines everywhere. Never 2px on one part and 0px on another.

2. **Light source from above** — Highlights at the top, shadows at the bottom. This is universal in 2D games.

3. **Palette cohesion** — Each species has 4-5 body colors that are the same hue at different lightness levels. Don't introduce random colors.

4. **Complementary outline** — Outline color should be a very dark version of the body's hue, not pure black. Marumi: dark blue-gray. Tsubasa: dark brown. Uroko: dark green.

5. **Transparent backgrounds** — Always RGBA with `(0,0,0,0)` background. The game composites parts on top of each other.

6. **Small canvas, big upscale** — Draw at pixel-art resolution (10-50px), upscale 8x with nearest-neighbor. This gives the chunky pixel look without anti-aliasing artifacts.

7. **Expressiveness through minimal change** — Marumi's 8 eye variants differ by just 3-5 parameters (lid rows, pupil position, sparkle). Small changes create big personality shifts.

---

## Summary

| What | Where | Format |
|------|-------|--------|
| Marumi sprites | `scripts/generate_sprites.py` | 52x52 body, 14x14 ears, 13x10 eyes, 12x6 mouths |
| Tsubasa sprites | `scripts/generate_tsubasa_sprites.py` | 48x44 body, 22x18 wings, 12x12 eyes, 12x10 beaks |
| Uroko sprites | `scripts/generate_uroko_sprites.py` | 42x50 body, 10x20 crests, 12x12 eyes, 16x10 snouts |
| Shared effects | Generated by Marumi script | ZZZ, hearts, rain, stars, sparkle |
| Output | `assets/sprites/{species}/` | RGBA PNG, 8x upscaled |
| In-game loading | `src/creature/spawn.rs` | Auto-detected at startup, fallback to mesh if missing |

The key insight: **sprites are an interface**. The Python scripts produce files that follow a naming convention. The Rust game loads files that follow that same convention. Neither side knows how the other works — they agree only on filenames, transparency, and dimensions. This is modular design in action.
