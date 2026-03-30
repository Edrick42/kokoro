"""
Kokoro — Modular Pixel Art Sprite Generator

Generates body part sprites that closely match the art style of idle_00.png:
round, chunky character with shading, thick outlines, and personality.

Each part is drawn at pixel-art resolution and upscaled 8x with nearest-neighbor.

Output: assets/sprites/marumi/{slot}_{mood}.png

Usage:
    python3 scripts/generate_sprites.py
"""

from PIL import Image, ImageDraw
import os
import math

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

OUT_DIR = os.path.join(os.path.dirname(__file__), '..', 'assets', 'sprites', 'marumi')
EFFECTS_DIR = os.path.join(os.path.dirname(__file__), '..', 'assets', 'sprites', 'shared', 'effects')
SCALE = 8

# Colors sampled from / matching idle_00.png
BODY       = (156, 232, 252, 255)   # Main body light blue
BODY_HI    = (180, 240, 255, 255)   # Highlight (top/center of body)
BODY_SH    = (120, 195, 225, 255)   # Shadow (bottom/edges)
BODY_SH2   = (100, 170, 205, 255)   # Deeper shadow (under-body, creases)
EAR_INNER  = (130, 200, 230, 255)   # Inner ear darker tint
OUTLINE    = (40, 45, 55, 255)      # Dark outline (not pure black — more natural)
PUPIL      = (50, 50, 65, 255)      # Eye pupils — dark charcoal
WHITE      = (255, 255, 255, 255)
EYE_WHITE  = (245, 248, 255, 255)   # Slightly blue-tinted white
CLEAR      = (0, 0, 0, 0)
PINK       = (240, 160, 165, 255)
BLUSH      = (235, 170, 185, 100)   # Semi-transparent blush
TEAR       = (140, 195, 250, 255)
GREEN_SICK = (165, 215, 145, 255)
GRAY_CLOUD = (185, 185, 195, 255)
YELLOW     = (255, 255, 110, 255)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def new_canvas(w, h):
    return Image.new('RGBA', (w, h), CLEAR)

def save(img, name, out_dir=None):
    d = out_dir or OUT_DIR
    w, h = img.size
    big = img.resize((w * SCALE, h * SCALE), Image.NEAREST)
    big.save(os.path.join(d, name))
    print(f"  {name} ({big.size[0]}x{big.size[1]})")

def px(img, coords, color):
    """Set pixels at given coordinates."""
    data = img.load()
    for x, y in coords:
        if 0 <= x < img.width and 0 <= y < img.height:
            data[x, y] = color

def flood_outline(img, shape_pixels, outline_color):
    """Add a 1px outline around a set of filled pixels."""
    filled = set(shape_pixels)
    outline = set()
    for x, y in filled:
        for dx, dy in [(-1,0),(1,0),(0,-1),(0,1)]:
            nx, ny = x+dx, y+dy
            if (nx, ny) not in filled and 0 <= nx < img.width and 0 <= ny < img.height:
                outline.add((nx, ny))
    px(img, outline, outline_color)

def ellipse_pixels(cx, cy, rx, ry):
    """Return set of (x,y) inside an ellipse."""
    pts = set()
    for y in range(int(cy - ry) - 1, int(cy + ry) + 2):
        for x in range(int(cx - rx) - 1, int(cx + rx) + 2):
            if ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 1.0:
                pts.add((x, y))
    return pts

def draw_ellipse_shaded(img, cx, cy, rx, ry, base, hi, sh, sh2, outline_col):
    """Draw a shaded ellipse with highlight on top, shadow on bottom."""
    pts = ellipse_pixels(cx, cy, rx, ry)
    # Color each pixel based on vertical position within ellipse
    top = cy - ry
    bot = cy + ry
    span = bot - top if bot != top else 1
    for x, y in pts:
        t = (y - top) / span  # 0 at top, 1 at bottom
        # Horizontal distance from center for edge darkening
        dx = abs(x - cx) / rx if rx > 0 else 0
        edge = dx ** 2  # 0 at center, 1 at edge
        if t < 0.25:
            c = hi
        elif t < 0.55:
            c = base
        elif t < 0.8:
            c = sh
        else:
            c = sh2
        # Darken edges slightly
        if edge > 0.7:
            c = sh if t < 0.5 else sh2
        img.load()[x, y] = c
    # Outline
    flood_outline(img, pts, outline_col)
    return pts

# ---------------------------------------------------------------------------
# BODY — round, chunky blob matching idle_00.png proportions
# ---------------------------------------------------------------------------

def _lerp_color(c1, c2, t):
    """Linearly interpolate between two RGBA colors."""
    t = max(0.0, min(1.0, t))
    return tuple(int(a + (b - a) * t) for a, b in zip(c1, c2))

def _appendage_shaded(img, cx, cy, rx, ry, all_body, W, H):
    """Draw a shaded appendage (arm/foot) that blends into the body."""
    pts = ellipse_pixels(cx, cy, rx, ry)
    for x, y in pts:
        t = (y - (cy - ry)) / (2 * ry) if ry > 0 else 0.5
        c = _lerp_color(BODY, BODY_SH, t)
        img.load()[x, y] = c
    # Only outline pixels that are NOT touching the main body
    outline = set()
    for x, y in pts:
        for dx, dy in [(-1,0),(1,0),(0,-1),(0,1)]:
            nx, ny = x+dx, y+dy
            if (nx, ny) not in pts and (nx, ny) not in all_body and 0 <= nx < W and 0 <= ny < H:
                outline.add((nx, ny))
    px(img, outline, OUTLINE)
    return pts

def gen_body():
    W, H = 52, 52
    img = new_canvas(W, H)
    d = img.load()

    # Main body — big, squat, round (wider than tall like the original)
    cx, cy, rx, ry = 26, 23, 19, 18
    body_pts = ellipse_pixels(cx, cy, rx, ry)

    # Feet connect directly to body — overlap slightly
    foot_l_pts = ellipse_pixels(18, 40, 7, 5)
    foot_r_pts = ellipse_pixels(34, 41, 7, 5)
    # Arms
    arm_l_pts = ellipse_pixels(8, 29, 4, 5)
    arm_r_pts = ellipse_pixels(44, 28, 4, 5)

    # Combine all parts as "body mass" for connected shading
    all_parts = body_pts | foot_l_pts | foot_r_pts | arm_l_pts | arm_r_pts

    # Smooth gradient shading across the body
    top = cy - ry
    bot = max(y for _, y in all_parts)
    span = bot - top
    for x, y in all_parts:
        t = (y - top) / span  # 0=top, 1=bottom
        dx_frac = abs(x - cx) / rx  # 0=center, 1=edge
        dx_frac = min(dx_frac, 1.0)
        # Smooth vertical gradient
        if t < 0.18:
            c = _lerp_color(BODY_HI, BODY, t / 0.18)
        elif t < 0.45:
            c = BODY
        elif t < 0.70:
            c = _lerp_color(BODY, BODY_SH, (t - 0.45) / 0.25)
        else:
            c = _lerp_color(BODY_SH, BODY_SH2, (t - 0.70) / 0.30)
        # Darken edges smoothly
        if dx_frac > 0.60:
            edge_t = (dx_frac - 0.60) / 0.40
            c = _lerp_color(c, BODY_SH, edge_t * 0.6)
        d[x, y] = c

    # Gentle specular highlight — subtle bright area upper-center
    hi_pts = ellipse_pixels(cx - 3, cy - 9, 5, 3)
    for x, y in hi_pts:
        if (x, y) in body_pts:
            # Blend 60% highlight, 40% existing
            existing = d[x, y]
            d[x, y] = _lerp_color(existing, BODY_HI, 0.6)

    # Shadow in the "crotch" area between feet
    crotch = ellipse_pixels(26, 40, 5, 2)
    for x, y in crotch:
        if (x, y) in all_parts:
            d[x, y] = BODY_SH2

    # Outline everything as one connected shape
    flood_outline(img, all_parts, OUTLINE)

    save(img, 'body_idle.png')


# ---------------------------------------------------------------------------
# EARS — rounded stubby bear ears (matching idle_00.png)
# ---------------------------------------------------------------------------

def gen_ears():
    W, H = 14, 14
    img = new_canvas(W, H)

    # Rounded ear shape — ellipse, slightly taller than wide
    ear_pts = ellipse_pixels(7, 7, 5, 6)
    for x, y in ear_pts:
        t = (y - 1) / 12
        if t < 0.3:
            c = BODY_HI
        elif t < 0.6:
            c = BODY
        else:
            c = BODY_SH
        img.load()[x, y] = c

    # Inner ear — smaller, darker ellipse
    inner = ellipse_pixels(7, 8, 3, 3)
    for x, y in inner:
        img.load()[x, y] = EAR_INNER

    flood_outline(img, ear_pts, OUTLINE)
    save(img, 'ear_left_idle.png')

    # Mirror for right ear
    right = img.transpose(Image.FLIP_LEFT_RIGHT)
    big = right.resize((W * SCALE, H * SCALE), Image.NEAREST)
    big.save(os.path.join(OUT_DIR, 'ear_right_idle.png'))
    print(f"  ear_right_idle.png ({big.size[0]}x{big.size[1]})")


# ---------------------------------------------------------------------------
# EYES — each mood variant, matching the droopy/expressive style
# ---------------------------------------------------------------------------

def _eye_base(w, h, lid_rows=1, pupil_y_off=0, pupil_h=4, sparkle=False, tear=False):
    """
    Generic eye builder.
    - lid_rows: how many rows the heavy upper eyelid covers (0=wide open, 3=very droopy)
    - pupil_y_off: vertical offset of the pupil from center (-=up, +=down)
    - pupil_h: height of the pupil block
    - sparkle: add a white highlight dot in the pupil
    - tear: add a tear drop on the right side
    """
    img = new_canvas(w, h)
    d = img.load()

    # Eye white area — rounded rectangle
    ew_l, ew_t, ew_r, ew_b = 2, 2, w - 3, h - 3
    eye_pts = set()
    for y in range(ew_t, ew_b + 1):
        for x in range(ew_l, ew_r + 1):
            # Round corners
            corner_r = 2
            in_corner = False
            for cx, cy in [(ew_l + corner_r, ew_t + corner_r),
                           (ew_r - corner_r, ew_t + corner_r),
                           (ew_l + corner_r, ew_b - corner_r),
                           (ew_r - corner_r, ew_b - corner_r)]:
                if ((x < ew_l + corner_r or x > ew_r - corner_r) and
                    (y < ew_t + corner_r or y > ew_b - corner_r)):
                    if (x - cx) ** 2 + (y - cy) ** 2 > corner_r ** 2 + 1:
                        in_corner = True
            if not in_corner:
                d[x, y] = EYE_WHITE
                eye_pts.add((x, y))

    # Upper eyelid (heavy, droopy — signature look of idle_00.png)
    for row in range(lid_rows):
        for x in range(ew_l, ew_r + 1):
            if (x, ew_t + row) in eye_pts:
                d[x, ew_t + row] = OUTLINE

    # Pupil — centered horizontally, offset vertically
    pw, ph = 3, pupil_h
    pcx = (ew_l + ew_r) // 2
    pcy = (ew_t + ew_b) // 2 + pupil_y_off
    for py in range(pcy - ph // 2, pcy + (ph + 1) // 2):
        for ppx in range(pcx - pw // 2, pcx + (pw + 1) // 2):
            if (ppx, py) in eye_pts:
                d[ppx, py] = PUPIL

    # Sparkle highlight in pupil
    if sparkle:
        sx, sy = pcx - 1, pcy - ph // 2
        if (sx, sy) in eye_pts:
            d[sx, sy] = WHITE

    # Outline
    flood_outline(img, eye_pts, OUTLINE)

    # Tear drop
    if tear:
        tx = ew_r + 1
        for ty in range(ew_b - 1, ew_b + 3):
            if 0 <= tx < w and 0 <= ty < h:
                d[tx, ty] = TEAR
        if 0 <= tx < w and 0 <= ew_b + 3 < h:
            d[tx, ew_b + 3] = TEAR

    return img

def eye_idle():
    """Half-lidded, relaxed — the signature look."""
    return _eye_base(13, 10, lid_rows=3, pupil_y_off=1, pupil_h=3, sparkle=True)

def eye_happy():
    """Wide open, bright, sparkly."""
    return _eye_base(13, 12, lid_rows=1, pupil_y_off=0, pupil_h=4, sparkle=True)

def eye_hungry():
    """Big, pleading, looking up with tear."""
    return _eye_base(13, 12, lid_rows=0, pupil_y_off=-1, pupil_h=4, sparkle=True, tear=True)

def eye_tired():
    """Very droopy, barely open."""
    return _eye_base(13, 8, lid_rows=4, pupil_y_off=1, pupil_h=2)

def eye_sleeping():
    """Closed — just a curved line."""
    img = new_canvas(13, 6)
    # Gentle downward curve
    px(img, [(3, 2), (4, 3), (5, 3), (6, 4), (7, 3), (8, 3), (9, 2)], OUTLINE)
    # Eyelash tick marks
    px(img, [(4, 4), (8, 4)], OUTLINE)
    return img

def eye_lonely():
    """Sad, looking down, with tear."""
    return _eye_base(13, 12, lid_rows=2, pupil_y_off=2, pupil_h=3, tear=True)

def eye_playful():
    """Wide, excited — star sparkle in pupil."""
    img = _eye_base(13, 12, lid_rows=0, pupil_y_off=0, pupil_h=4, sparkle=False)
    d = img.load()
    # Replace pupil with a star/cross pattern
    cx, cy = 6, 6
    star = [(cx, cy-2), (cx, cy-1), (cx, cy), (cx, cy+1), (cx, cy+2),
            (cx-2, cy), (cx-1, cy), (cx+1, cy), (cx+2, cy),
            (cx-1, cy-1), (cx+1, cy-1), (cx-1, cy+1), (cx+1, cy+1)]
    for x, y in star:
        if 0 <= x < img.width and 0 <= y < img.height:
            d[x, y] = PUPIL
    d[cx, cy] = WHITE  # sparkle center
    return img

def eye_sick():
    """Spiral/dizzy — X eyes."""
    img = new_canvas(13, 10)
    d = img.load()
    # Eye shape
    eye_pts = set()
    for y in range(2, 8):
        for x in range(2, 11):
            d[x, y] = EYE_WHITE
            eye_pts.add((x, y))
    flood_outline(img, eye_pts, OUTLINE)
    # X pattern
    x_pts = [(4,3),(5,4),(6,5),(7,6),(8,3),(7,4),(5,6),(4,7),(8,7)]
    px(img, x_pts, PUPIL)
    return img

def gen_eyes():
    variants = {
        'idle': eye_idle(),
        'happy': eye_happy(),
        'hungry': eye_hungry(),
        'tired': eye_tired(),
        'sleeping': eye_sleeping(),
        'lonely': eye_lonely(),
        'playful': eye_playful(),
        'sick': eye_sick(),
    }
    for mood, img in variants.items():
        save(img, f'eye_left_{mood}.png')
        right = img.transpose(Image.FLIP_LEFT_RIGHT)
        big = right.resize((right.width * SCALE, right.height * SCALE), Image.NEAREST)
        big.save(os.path.join(OUT_DIR, f'eye_right_{mood}.png'))
        print(f"  eye_right_{mood}.png ({big.size[0]}x{big.size[1]})")


# ---------------------------------------------------------------------------
# MOUTH — mood variants, small and subtle like the original
# ---------------------------------------------------------------------------

def mouth_idle():
    """Small, subtle "w" shape — the signature Kobara mouth."""
    img = new_canvas(12, 6)
    # Cat-like "w" mouth
    px(img, [(3, 1), (4, 2), (5, 2), (6, 1), (7, 2), (8, 2), (9, 1)], OUTLINE)
    return img

def mouth_happy():
    """Wide open smile with visible teeth."""
    img = new_canvas(14, 8)
    # Smile outline
    px(img, [
        (3, 1), (4, 1), (9, 1), (10, 1),  # top edges
        (2, 2), (11, 2),
        (2, 3), (11, 3),
        (2, 4), (11, 4),
        (3, 5), (4, 5), (5, 5), (6, 5), (7, 5), (8, 5), (9, 5), (10, 5),
    ], OUTLINE)
    # Inside — pink
    for y in range(2, 5):
        for x in range(3, 11):
            img.load()[x, y] = PINK
    # Top row teeth
    px(img, [(5, 1), (6, 1), (7, 1), (8, 1)], WHITE)
    px(img, [(5, 2), (6, 2), (7, 2), (8, 2)], WHITE)
    return img

def mouth_hungry():
    """Open O shape, wanting food."""
    img = new_canvas(10, 10)
    pts = ellipse_pixels(5, 5, 3, 3)
    for x, y in pts:
        img.load()[x, y] = (70, 35, 40, 255)  # dark inside
    flood_outline(img, pts, OUTLINE)
    # Tongue hint at bottom
    px(img, [(4, 6), (5, 6), (6, 6)], PINK)
    return img

def mouth_tired():
    """Wide yawn — open oval."""
    img = new_canvas(12, 10)
    pts = ellipse_pixels(6, 5, 4, 3)
    for x, y in pts:
        img.load()[x, y] = (70, 35, 40, 255)
    flood_outline(img, pts, OUTLINE)
    # Tongue
    px(img, [(5, 6), (6, 6), (7, 6), (5, 7), (6, 7)], PINK)
    return img

def mouth_sleeping():
    """Tiny peaceful dot/line."""
    img = new_canvas(8, 4)
    px(img, [(3, 1), (4, 1)], OUTLINE)
    return img

def mouth_lonely():
    """Small frown — inverted curve."""
    img = new_canvas(12, 6)
    px(img, [(3, 4), (4, 3), (5, 2), (6, 2), (7, 2), (8, 3), (9, 4)], OUTLINE)
    return img

def mouth_playful():
    """Big grin with tongue sticking out."""
    img = new_canvas(14, 10)
    # Grin
    px(img, [
        (3, 1), (4, 1), (9, 1), (10, 1),
        (2, 2), (11, 2),
        (2, 3), (11, 3),
        (2, 4), (11, 4),
        (3, 5), (4, 5), (5, 5), (6, 5), (7, 5), (8, 5), (9, 5), (10, 5),
    ], OUTLINE)
    # Inside
    for y in range(2, 5):
        for x in range(3, 11):
            img.load()[x, y] = PINK
    # Teeth
    px(img, [(5, 1), (6, 1), (7, 1), (8, 1), (5, 2), (6, 2), (7, 2), (8, 2)], WHITE)
    # Tongue out the side
    tongue = [(10, 5), (11, 5), (10, 6), (11, 6), (12, 6), (11, 7), (12, 7)]
    px(img, tongue, PINK)
    flood_outline(img, tongue, OUTLINE)
    return img

def mouth_sick():
    """Wavy queasy line with green tinge."""
    img = new_canvas(12, 6)
    px(img, [(2, 3), (3, 2), (4, 3), (5, 4), (6, 3), (7, 2), (8, 3), (9, 4)], OUTLINE)
    px(img, [(3, 4), (7, 4)], GREEN_SICK)
    return img

def gen_mouths():
    variants = {
        'idle': mouth_idle(),
        'happy': mouth_happy(),
        'hungry': mouth_hungry(),
        'tired': mouth_tired(),
        'sleeping': mouth_sleeping(),
        'lonely': mouth_lonely(),
        'playful': mouth_playful(),
        'sick': mouth_sick(),
    }
    for mood, img in variants.items():
        save(img, f'mouth_{mood}.png')


# ---------------------------------------------------------------------------
# SHARED EFFECTS
# ---------------------------------------------------------------------------

def gen_effects():
    os.makedirs(EFFECTS_DIR, exist_ok=True)

    # ZZZ sleep indicator
    img = new_canvas(20, 20)
    px(img, [
        # Small z
        (3, 14), (4, 14), (5, 14), (5, 15), (4, 16), (3, 16), (3, 17), (4, 17), (5, 17),
        # Medium Z
        (8, 9), (9, 9), (10, 9), (11, 9), (11, 10), (10, 11), (9, 12),
        (8, 13), (9, 13), (10, 13), (11, 13),
        # Large Z
        (13, 2), (14, 2), (15, 2), (16, 2), (17, 2),
        (16, 3), (15, 4), (14, 5), (13, 6),
        (13, 7), (14, 7), (15, 7), (16, 7), (17, 7),
    ], WHITE)
    save(img, 'zzz.png', EFFECTS_DIR)

    # Hearts
    img = new_canvas(20, 20)
    def draw_heart(img, ox, oy, size, color):
        """Draw a pixel heart at offset."""
        if size == 'sm':
            pts = [(0,0),(1,0),(3,0),(4,0),
                   (-1,1),(0,1),(1,1),(2,1),(3,1),(4,1),(5,1),
                   (0,2),(1,2),(2,2),(3,2),(4,2),
                   (1,3),(2,3),(3,3),(2,4)]
        else:
            pts = [(1,0),(2,0),(4,0),(5,0),
                   (0,1),(1,1),(2,1),(3,1),(4,1),(5,1),(6,1),
                   (0,2),(1,2),(2,2),(3,2),(4,2),(5,2),(6,2),
                   (1,3),(2,3),(3,3),(4,3),(5,3),
                   (2,4),(3,4),(4,4),(3,5)]
        px(img, [(x+ox, y+oy) for x,y in pts], color)

    draw_heart(img, 3, 12, 'sm', (255, 100, 120, 255))
    draw_heart(img, 10, 3, 'lg', (255, 75, 100, 255))
    save(img, 'hearts.png', EFFECTS_DIR)

    # Rain cloud
    img = new_canvas(24, 18)
    # Cloud body
    cloud = set()
    for cx, cy, r in [(11, 3, 4), (8, 4, 3), (14, 4, 3), (11, 5, 5)]:
        cloud |= ellipse_pixels(cx, cy, r, r * 0.6)
    for x, y in cloud:
        img.load()[x, y] = GRAY_CLOUD
    flood_outline(img, cloud, OUTLINE)
    # Rain drops
    for rx, ry in [(8, 9), (11, 10), (14, 9), (9, 13), (13, 12)]:
        px(img, [(rx, ry), (rx, ry+1), (rx, ry+2)], TEAR)
    save(img, 'rain_cloud.png', EFFECTS_DIR)

    # Dizzy stars
    img = new_canvas(24, 24)
    for sx, sy in [(6, 5), (17, 7), (5, 15), (18, 17)]:
        star = [(sx, sy-2), (sx, sy-1), (sx, sy), (sx, sy+1), (sx, sy+2),
                (sx-2, sy), (sx-1, sy), (sx+1, sy), (sx+2, sy)]
        px(img, star, YELLOW)
        px(img, [(sx, sy)], WHITE)
    save(img, 'stars_dizzy.png', EFFECTS_DIR)

    # Sparkle
    img = new_canvas(24, 24)
    for sx, sy in [(5, 4), (18, 6), (4, 18), (20, 17), (12, 2), (12, 21)]:
        sparkle = [(sx, sy-2), (sx, sy-1), (sx, sy), (sx, sy+1), (sx, sy+2),
                   (sx-2, sy), (sx-1, sy), (sx+1, sy), (sx+2, sy)]
        px(img, sparkle, (255, 255, 200, 255))
        px(img, [(sx, sy)], WHITE)
    save(img, 'sparkle.png', EFFECTS_DIR)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == '__main__':
    os.makedirs(OUT_DIR, exist_ok=True)
    print("Generating Kobara modular sprites...\n")

    print("[Body]")
    gen_body()

    print("\n[Ears]")
    gen_ears()

    print("\n[Eyes — all moods]")
    gen_eyes()

    print("\n[Mouths — all moods]")
    gen_mouths()

    print("\n[Shared effects]")
    gen_effects()

    print("\nDone! All sprites saved to assets/sprites/")
