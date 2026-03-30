"""
Kokoro — Uroko (Reptile) Species Sprite Generator

Generates modular pixel art sprites for the Uroko species:
- Elongated, angular body
- Crests/horns instead of ears
- Slitted predator eyes
- Wide snout instead of mouth
- Thick scaled tail

Output: assets/sprites/uroko/{slot}_{mood}.png
"""

from PIL import Image, ImageDraw
import os
import math

OUT_DIR = os.path.join(os.path.dirname(__file__), '..', 'assets', 'sprites', 'uroko')
SCALE = 8

# Colors — cool reptile palette
BODY       = (120, 180, 130, 255)
BODY_HI    = (150, 210, 155, 255)
BODY_SH    = (90, 150, 105, 255)
BODY_SH2   = (70, 125, 85, 255)
SCALE_COL  = (100, 165, 115, 255)
CREST_COL  = (160, 100, 80, 255)
CREST_TIP  = (190, 120, 90, 255)
OUTLINE    = (35, 50, 40, 255)
PUPIL      = (30, 30, 30, 255)
IRIS       = (200, 160, 50, 255)
EYE_WHITE  = (230, 240, 200, 255)
WHITE      = (255, 255, 255, 255)
CLEAR      = (0, 0, 0, 0)
SNOUT_COL  = (80, 100, 80, 255)
PINK       = (200, 130, 130, 255)
TEAR       = (140, 195, 250, 255)

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
# Body — elongated, slightly angular
# ---------------------------------------------------------------------------
def gen_body():
    W, H = 42, 50
    img = new_canvas(W, H)
    cx, cy = W//2, H//2

    # Main body (taller ellipse)
    body = ellipse_pixels(cx, cy, 16, 20)

    for x, y in body:
        t = (y - (cy - 20)) / 40.0
        if t < 0.25:
            c = _lerp(BODY_HI, BODY, t / 0.25)
        elif t < 0.6:
            c = BODY
        else:
            c = _lerp(BODY, BODY_SH, (t - 0.6) / 0.4)
        px(img, [(x, y)], c)

    # Scale pattern (horizontal lines for texture)
    for y_off in range(-15, 16, 4):
        row_y = cy + y_off
        for x, y in body:
            if y == row_y:
                px(img, [(x, y)], SCALE_COL)

    flood_outline(img, body, OUTLINE)
    save(img, 'body_idle.png')

# ---------------------------------------------------------------------------
# Crests (horns)
# ---------------------------------------------------------------------------
def gen_crest(side):
    W, H = 10, 20
    img = new_canvas(W, H)

    pts = set()
    if side == 'left':
        # Triangular horn pointing up-left
        for y in range(2, H - 2):
            span = max(1, (H - 2 - y) // 3)
            base_x = W - 4
            for x in range(base_x - span, base_x + 2):
                pts.add((x, y))
    else:
        for y in range(2, H - 2):
            span = max(1, (H - 2 - y) // 3)
            base_x = 4
            for x in range(base_x - 1, base_x + span + 1):
                pts.add((x, y))

    for x, y in pts:
        t = (H - y) / H
        c = _lerp(CREST_COL, CREST_TIP, min(t, 1.0))
        px(img, [(x, y)], c)

    flood_outline(img, pts, OUTLINE)
    save(img, f'crest_{side}_idle.png')

# ---------------------------------------------------------------------------
# Eyes — slitted reptile pupils
# ---------------------------------------------------------------------------
def _reptile_eye(W=12, H=12, slit_open=2, lid_rows=0, sparkle=False, tear=False):
    img = new_canvas(W, H)
    cx, cy = W//2, H//2

    # Eye shape (slightly angular)
    eye_pts = ellipse_pixels(cx, cy, 4, 3)
    px(img, eye_pts, EYE_WHITE)

    # Iris
    iris_pts = ellipse_pixels(cx, cy, 3, 3)
    px(img, iris_pts, IRIS)

    # Vertical slit pupil
    for dy in range(-3, 4):
        for dx in range(-slit_open//2, slit_open//2 + 1):
            if (cx + dx, cy + dy) in iris_pts:
                px(img, [(cx + dx, cy + dy)], PUPIL)

    if sparkle:
        px(img, [(cx - 1, cy - 1)], WHITE)

    if tear:
        px(img, [(cx, cy + 3), (cx, cy + 4)], TEAR)

    # Lid
    for r in range(lid_rows):
        for x in range(cx - 4, cx + 5):
            if (x, cy - 3 + r) in eye_pts:
                px(img, [(x, cy - 3 + r)], OUTLINE)

    flood_outline(img, eye_pts, OUTLINE)
    return img

def gen_eyes():
    moods = {
        'idle': dict(slit_open=1, sparkle=True),
        'hungry': dict(slit_open=2),
        'tired': dict(slit_open=1, lid_rows=2),
        'lonely': dict(slit_open=1, tear=True),
        'playful': dict(slit_open=2, sparkle=True),
        'sick': dict(slit_open=1, lid_rows=1),
        'sleeping': dict(slit_open=0, lid_rows=4),
    }
    for mood, kw in moods.items():
        left = _reptile_eye(**kw)
        save(left, f'eye_left_{mood}.png')
        right = left.transpose(Image.FLIP_LEFT_RIGHT)
        save(right, f'eye_right_{mood}.png')

# ---------------------------------------------------------------------------
# Snout (replaces mouth)
# ---------------------------------------------------------------------------
def gen_snout():
    moods = {
        'idle': 'closed',
        'hungry': 'open',
        'tired': 'closed',
        'lonely': 'closed',
        'playful': 'grin',
        'sick': 'closed',
        'sleeping': 'closed',
    }
    for mood, style in moods.items():
        W, H = 16, 10
        img = new_canvas(W, H)
        cx = W // 2

        if style == 'closed':
            pts = set()
            for y in range(3, 7):
                span = 5 - abs(y - 5)
                for x in range(cx - span, cx + span + 1):
                    pts.add((x, y))
            px(img, pts, SNOUT_COL)
            # Nostrils
            px(img, [(cx - 2, 4), (cx + 2, 4)], OUTLINE)
            flood_outline(img, pts, OUTLINE)

        elif style == 'open':
            upper = set()
            for y in range(2, 5):
                span = 5 - abs(y - 3)
                for x in range(cx - span, cx + span + 1):
                    upper.add((x, y))
            px(img, upper, SNOUT_COL)
            lower = set()
            for y in range(6, 9):
                span = 4 - abs(y - 7)
                for x in range(cx - span, cx + span + 1):
                    lower.add((x, y))
            px(img, lower, SNOUT_COL)
            # Teeth
            px(img, [(cx - 2, 5), (cx, 5), (cx + 2, 5)], WHITE)
            flood_outline(img, upper | lower, OUTLINE)

        elif style == 'grin':
            pts = set()
            for y in range(3, 7):
                span = 6 - abs(y - 5)
                for x in range(cx - span, cx + span + 1):
                    pts.add((x, y))
            px(img, pts, SNOUT_COL)
            # Grin line
            for x in range(cx - 4, cx + 5):
                px(img, [(x, 5)], OUTLINE)
            # Fang
            px(img, [(cx - 3, 6), (cx + 3, 6)], WHITE)
            flood_outline(img, pts, OUTLINE)

        save(img, f'snout_{mood}.png')

# ---------------------------------------------------------------------------
# Tail — thick, scaled
# ---------------------------------------------------------------------------
def gen_tail():
    W, H = 14, 24
    img = new_canvas(W, H)
    cx = W // 2

    pts = set()
    for y in range(2, H - 1):
        # Tapers toward the tip
        width = max(1, int(5 * (1.0 - (y - 2) / (H - 3) * 0.6)))
        for x in range(cx - width, cx + width + 1):
            pts.add((x, y))

    for x, y in pts:
        t = y / H
        c = _lerp(BODY, BODY_SH2, t)
        px(img, [(x, y)], c)

    # Scale bands
    for y_off in range(4, H - 2, 3):
        for x, y in pts:
            if y == y_off:
                px(img, [(x, y)], SCALE_COL)

    flood_outline(img, pts, OUTLINE)
    save(img, 'tail_idle.png')

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
if __name__ == '__main__':
    os.makedirs(OUT_DIR, exist_ok=True)
    print(f"Generating Uroko sprites → {OUT_DIR}")
    gen_body()
    gen_crest('left')
    gen_crest('right')
    gen_eyes()
    gen_snout()
    gen_tail()
    print("Done!")
