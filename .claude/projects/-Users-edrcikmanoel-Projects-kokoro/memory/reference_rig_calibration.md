---
name: Rig calibration reference
description: Measured landmark positions from idle_00.png used to calibrate the Marumi body rig
type: reference
---

## Marumi rig calibration (measured from idle_00.png)

Values extracted by pixel analysis of idle_00.png downsampled to 128x128.
Body center and proportions measured from blue body pixels; eyes/mouth from
interior dark pixel clusters.

### Raw measurements (128x128 space)

- Body center: (63.7, 62.6), size: 89x78
- Left eye: (44.5, 48.8)
- Right eye: (78.7, 48.0)
- Mouth: (61.0, 61.6)

### Normalized coordinates [-1, 1]

| Part | X | Y | Notes |
|------|---|---|-------|
| Eyes | ±0.385 | 0.364 | ~38% from center, ~36% above center |
| Mouth | 0.0 | 0.026 | nearly centered vertically |
| Ears | ±0.398 | 0.884 | far above body, slightly wider than eyes |

### Rig values applied (before → after)

| Part | Old value | New value (measured) | Pixel offset (new) |
|------|-----------|---------------------|-------------------|
| base_size | 140×160 | **416×365** | matches rendered sprite scale |
| Eyes X | ±0.45 (±31px) | **±0.38 (±79px)** | 2.5x more separated |
| Eyes Y | 0.15 (+12px) | **0.36 (+66px)** | much higher |
| Mouth Y | -0.12 (-10px) | **0.03 (+5px)** | nearly centered |
| Ears X | ±0.35 (±24px) | **±0.40 (±83px)** | wider |
| Ears Y | 0.55 (+44px) | **0.88 (+161px)** | much higher |

### Key insight

The original `base_size` of 140×160 was designed for procedural mesh fallbacks
(body radius ~55px). After switching to 8x upscaled sprites (52px × 8 = 416px),
the base_size needed to match the rendered scale. All offsets were too small
because they were being computed against a 140px box but rendered against a 416px sprite.
