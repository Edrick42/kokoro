# 心 Kokoro

> *Kokoro* (心) means "heart" and "mind" in Japanese — the invisible force that makes each creature feel alive.

Kokoro is a virtual creature game built in Rust where every **Kobara** is genuinely unique. Inspired by Tamagotchi, but taken further: each creature carries a genetic blueprint that shapes its personality, appearance, and behaviour. No two Kobaras are alike — just like in nature.

---

## What makes Kokoro different

In most virtual pet games, creatures are skins over the same logic. In Kokoro:

- **Genetics drive personality.** Each Kobara is born with a genome — a set of genes (values between 0.0 and 1.0) that determine appetite, curiosity, emotional resilience, sleep patterns, and more. The species sets the possible ranges; the individual fills them in.
- **Behaviour emerges from rules + randomness.** A hungry, shy Kobara at night won't ask for food — it hides. A curious, high-energy one in the afternoon might bounce around unprompted. Simple rules, complex outcomes.
- **A local neural network learns your Kobara specifically.** Over time, the creature's responses adapt to *your* interaction patterns. It's not global AI — it runs on-device, belongs only to this creature, and evolves with it.

---

## Architecture

```
src/
├── genome/       # DNA of each Kobara (genes, species, body color)
├── mind/         # AI engine: FSM mood states + emergent behaviour rules
├── systems/      # Bevy ECS systems: rendering, HUD, time ticks
├── world/        # Scene setup: camera, environment
└── main.rs       # App entry point
```

### The creature's mind (AI layers)

```
┌─────────────────────────────────────────┐
│  Finite State Machine                   │  ← current mood: Happy, Hungry, Lonely...
│  mood transitions driven by vital stats │
├─────────────────────────────────────────┤
│  Emergent behaviour rules               │  ← genome × stats × randomness
│  same stats, different genes = different│    behaviour
├─────────────────────────────────────────┤
│  Local neural network  (Phase 4)        │  ← learns from owner interaction history
│  runs on-device, unique to each Kobara  │
└─────────────────────────────────────────┘
```

### The genome

Each gene is an `f32` between `0.0` and `1.0`:

| Gene | Low (0.0) | High (1.0) |
|------|-----------|------------|
| `curiosity` | Calm, apathetic | Always exploring |
| `appetite` | Slow metabolism | Gets hungry fast |
| `loneliness_sensitivity` | Independent | Needs constant company |
| `circadian` | Night owl | Day creature |
| `resilience` | Fragile, moody | Bounces back quickly |
| `learning_rate` | Slow learner | Adapts fast |
| `hue` | — | Determines body colour (0°–360°) |

---

## Tech stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2024) |
| Game engine | [Bevy](https://bevyengine.org/) 0.16 (ECS-first, mobile-ready) |
| Serialisation | `serde` + `bincode` |
| Randomness | `rand` |
| Persistence (Phase 2) | `rusqlite` — SQLite embedded |
| Neural network (Phase 4) | `candle-core` (Hugging Face, pure Rust) |
| Target platforms | Android, iOS (via `cargo-mobile2`) |

---

## Roadmap

### Phase 1 — Foundation ✅ (current)
- [x] Bevy project structure
- [x] Genome system with random generation
- [x] Finite state machine (FSM) mood engine
- [x] Emergent behaviour rules driven by genes
- [x] Procedural creature rendering (colour from genome)
- [x] HUD displaying vital stats and mood

### Phase 2 — Memory & persistence
- [ ] SQLite persistence via `rusqlite`
- [ ] Save/load creature state between sessions
- [ ] Interaction history log
- [ ] Basic evolution: stats shift slightly over generations

### Phase 3 — Mobile UI
- [ ] Touch controls (feed, play, sleep)
- [ ] Sprite-based visuals replacing meshes
- [ ] Sound cues tied to mood states
- [ ] Android + iOS build via `cargo-mobile2`

### Phase 4 — Neural mind
- [ ] Small MLP (multi-layer perceptron) in pure Rust via `candle-core`
- [ ] Trains locally on interaction history
- [ ] Creature adapts to owner's patterns over weeks
- [ ] Each Kobara's network is unique and non-transferable

---

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- For mobile: [`cargo-mobile2`](https://github.com/tauri-apps/cargo-mobile2)

### Run on desktop (development)

```bash
git clone https://github.com/Edrick42/kokoro
cd kokoro
cargo run
```

### Build for Android

```bash
cargo mobile init
cargo android build --release
```

---

## Art direction — generating creature sprites (v1.0)

The current version renders the Kobara procedurally with Bevy meshes. For v1.0 sprites, here's the recommended approach:

### Option A — AI image generation (fastest)
Use a tool like **Midjourney**, **DALL·E 3**, or **Stable Diffusion** with prompts like:

```
cute virtual pet creature, round body, big eyes, soft pastel [COLOR] fur,
chibi style, transparent background, pixel art / flat vector / hand-drawn,
tamagotchi aesthetic, 512x512
```

Generate at least 4 animation frames per mood: idle, happy, hungry, sleeping.

### Option B — Pixel art tools (full control)
- **Aseprite** (~$20, industry standard for pixel art + animation)
- **Libresprite** (free Aseprite fork)
- **Pixelorama** (free, open source)

Design at 32×32 or 64×64 px. Export each frame as PNG to `assets/sprites/kobara/`.

### Option C — Vector → raster pipeline
Draw in **Figma** or **Inkscape** → export SVG → rasterise to PNG at multiple resolutions for different screen densities.

### Sprite structure (target)
```
assets/
└── sprites/
    └── kobara/
        ├── idle_0.png
        ├── idle_1.png
        ├── happy_0.png
        ├── hungry_0.png
        └── sleeping_0.png
```

---

## Contributing

This project is an open learning exercise in Rust. Issues, suggestions, and PRs are welcome — especially from people also learning the language.

---

## License

MIT
