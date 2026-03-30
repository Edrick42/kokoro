"""
Kokoro — Tsubasa (Bird) Species Sprite Generator

Generates modular pixel art sprites for the Tsubasa species:
- Rounder, lighter body than Marumi (egg-shaped)
- Wings instead of ears
- Pointed beak instead of mouth
- Tail feathers
- Same mood variants for eyes and beak

Output: assets/sprites/tsubasa/{slot}_{mood}.png
"""

from PIL import Image, ImageDraw
import os
import math

OUT_DIR = os.path.join(os.path.dirname(__file__), '..', 'assets', 'sprites', 'tsubasa')
SCALE = 8

# Colors — warm bird palette
BODY       = (255, 220, 140, 255)
BODY_HI    = (255, 235, 170, 255)
BODY_SH    = (230, 190, 110, 255)
BODY_SH2   = (210, 170, 95, 255)
WING_BASE  = (240, 200, 120, 255)
WING_TIP   = (200, 160, 90, 255)
OUTLINE    = (60, 45, 30, 255)
PUPIL      = (30, 30, 45, 255)
WHITE      = (255, 255, 255, 255)
EYE_WHITE  = (250, 250, 255, 255)
CLEAR      = (0, 0, 0, 0)
BEAK_COL   = (255, 160, 60, 255)
BEAK_DARK  = (220, 130, 40, 255)
PINK       = (240, 160, 165, 255)
BLUSH      = (235, 170, 185, 100)
TEAR       = (140, 195, 250, 255)
GREEN_SICK = (180, 220, 140, 255)

def new_canvas(w, h):
    return Image.new('RGBA', (w, h), CLEAR)

def save(img, name):
    os.makedirs(OUT_DIR, exist_ok=True)
    big = img.resize((img.width * SCALE, img.height * SCALE), Image.NEAREST)
    big.save(os.path.join(OUT_DIR, name))
    print(f"  {name} ({big.size[0]}x{big.size[1]})")

def px(img, coords, color):
    data = img.load()
    for x, y in coords:
        if 0 <= x < img.width and 0 <= y < img.height:
            data[x, y] = color

def ellipse_pixels(cx, cy, rx, ry):
    pts = set()
    for y in range(int(cy - ry) - 1, int(cy + ry) + 2):
        for x in range(int(cx - rx) - 1, int(cx + rx) + 2):
            dx = (x - cx) / max(rx, 0.1)
            dy = (y - cy) / max(ry, 0.1)
            if dx*dx + dy*dy <= 1.0:
                pts.add((x, y))
    return pts

def flood_outline(img, shape_pixels, color):
    filled = set(shape_pixels)
    outline = set()
    for x, y in filled:
        for dx, dy in [(-1,0),(1,0),(0,-1),(0,1)]:
            nx, ny = x+dx, y+dy
            if (nx, ny) not in filled and 0 <= nx < img.width and 0 <= ny < img.height:
                outline.add((nx, ny))
    px(img, outline, color)

def _lerp(c1, c2, t):
    return tuple(int(c1[i] + (c2[i] - c1[i]) * t) for i in range(4))

# ---------------------------------------------------------------------------
# Body — egg-shaped, lighter
# ---------------------------------------------------------------------------
def gen_body():
    W, H = 48, 44
    img = new_canvas(W, H)
    cx, cy = W//2, H//2 + 2

    body = ellipse_pixels(cx, cy, 18, 16)
    # Shading
    for x, y in body:
        t = (y - (cy - 16)) / 32.0
        if t < 0.3:
            c = _lerp(BODY_HI, BODY, t / 0.3)
        elif t < 0.7:
            c = BODY
        else:
            c = _lerp(BODY, BODY_SH, (t - 0.7) / 0.3)
        px(img, [(x, y)], c)

    flood_outline(img, body, OUTLINE)
    save(img, 'body_idle.png')

# ---------------------------------------------------------------------------
# Wings
# ---------------------------------------------------------------------------
def gen_wing(side):
    W, H = 22, 18
    img = new_canvas(W, H)

    if side == 'left':
        cx = W - 5
        pts = ellipse_pixels(cx, H//2, 12, 7)
        for x, y in pts:
            t = (cx - x) / 12.0
            c = _lerp(WING_BASE, WING_TIP, min(t, 1.0))
            px(img, [(x, y)], c)
        flood_outline(img, pts, OUTLINE)
    else:
        cx = 5
        pts = ellipse_pixels(cx, H//2, 12, 7)
        for x, y in pts:
            t = (x - cx) / 12.0
            c = _lerp(WING_BASE, WING_TIP, min(max(t, 0), 1.0))
            px(img, [(x, y)], c)
        flood_outline(img, pts, OUTLINE)

    save(img, f'wing_{side}_idle.png')

# ---------------------------------------------------------------------------
# Eyes (similar to Marumi but rounder)
# ---------------------------------------------------------------------------
def _eye_base(W=12, H=12, pupil_dx=0, lid_rows=0, sparkle=True, tear=False):
    img = new_canvas(W, H)
    cx, cy = W//2, H//2

    eye_pts = ellipse_pixels(cx, cy, 4, 4)
    px(img, eye_pts, EYE_WHITE)
    flood_outline(img, eye_pts, OUTLINE)

    pupil_pts = ellipse_pixels(cx + pupil_dx, cy, 2, 2)
    px(img, pupil_pts, PUPIL)

    if sparkle:
        px(img, [(cx - 1, cy - 2)], WHITE)

    if tear:
        px(img, [(cx, cy + 4), (cx, cy + 5)], TEAR)

    for r in range(lid_rows):
        for x in range(cx - 4, cx + 5):
            if (x, cy - 4 + r) in eye_pts:
                px(img, [(x, cy - 4 + r)], OUTLINE)

    return img

def gen_eyes():
    moods = {
        'idle': dict(sparkle=True),
        'hungry': dict(pupil_dx=0, lid_rows=1),
        'tired': dict(lid_rows=3, sparkle=False),
        'lonely': dict(tear=True, sparkle=True),
        'playful': dict(sparkle=True),
        'sick': dict(lid_rows=2, sparkle=False),
        'sleeping': dict(lid_rows=5, sparkle=False),
    }
    for mood, kw in moods.items():
        left = _eye_base(**kw)
        save(left, f'eye_left_{mood}.png')
        right = left.transpose(Image.FLIP_LEFT_RIGHT)
        save(right, f'eye_right_{mood}.png')

# ---------------------------------------------------------------------------
# Beak (replaces mouth)
# ---------------------------------------------------------------------------
def gen_beak():
    moods = {
        'idle': 'closed',
        'hungry': 'open',
        'tired': 'closed',
        'lonely': 'closed',
        'playful': 'open_wide',
        'sick': 'closed',
        'sleeping': 'closed',
    }
    for mood, style in moods.items():
        W, H = 12, 10
        img = new_canvas(W, H)
        cx = W // 2

        if style == 'closed':
            # Simple triangle beak
            pts = set()
            for y in range(3, 7):
                span = max(1, 4 - (y - 3))
                for x in range(cx - span, cx + span + 1):
                    pts.add((x, y))
            px(img, pts, BEAK_COL)
            flood_outline(img, pts, OUTLINE)

        elif style == 'open':
            # Upper beak
            upper = set()
            for y in range(2, 5):
                span = max(1, 4 - (y - 2))
                for x in range(cx - span, cx + span + 1):
                    upper.add((x, y))
            px(img, upper, BEAK_COL)
            # Lower beak (gap)
            lower = set()
            for y in range(6, 8):
                span = max(1, 3 - (y - 6))
                for x in range(cx - span, cx + span + 1):
                    lower.add((x, y))
            px(img, lower, BEAK_DARK)
            flood_outline(img, upper | lower, OUTLINE)

        elif style == 'open_wide':
            upper = set()
            for y in range(1, 4):
                span = max(1, 5 - (y - 1))
                for x in range(cx - span, cx + span + 1):
                    upper.add((x, y))
            px(img, upper, BEAK_COL)
            lower = set()
            for y in range(5, 9):
                span = max(1, 4 - (y - 5))
                for x in range(cx - span, cx + span + 1):
                    lower.add((x, y))
            px(img, lower, BEAK_DARK)
            # Pink mouth interior
            interior = set()
            for x in range(cx - 2, cx + 3):
                interior.add((x, 4))
            px(img, interior, PINK)
            flood_outline(img, upper | lower | interior, OUTLINE)

        save(img, f'beak_{mood}.png')

# ---------------------------------------------------------------------------
# Tail
# ---------------------------------------------------------------------------
def gen_tail():
    W, H = 16, 18
    img = new_canvas(W, H)
    cx = W // 2

    # Three tail feathers
    for offset in [-3, 0, 3]:
        feather = set()
        for y in range(4, H - 1):
            span = max(1, 3 - abs(y - 10) // 3)
            for x in range(cx + offset - span, cx + offset + span + 1):
                feather.add((x, y))
        t_base = 0.3 if offset == 0 else 0.5
        c = _lerp(WING_BASE, WING_TIP, t_base)
        px(img, feather, c)
        flood_outline(img, feather, OUTLINE)

    save(img, 'tail_idle.png')

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
if __name__ == '__main__':
    os.makedirs(OUT_DIR, exist_ok=True)
    print(f"Generating Tsubasa sprites → {OUT_DIR}")
    gen_body()
    gen_wing('left')
    gen_wing('right')
    gen_eyes()
    gen_beak()
    gen_tail()
    print("Done!")
