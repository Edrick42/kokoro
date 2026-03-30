# 心 Kokoro

> *Kokoro* (心) means "heart" and "mind" in Japanese — the invisible force that makes each creature feel alive.

Kokoro is a virtual creature game built in Rust where every **Kobara** is genuinely unique. The name *Kobara* fuses 心 (kokoro, heart/mind) with 腹 (hara, the seat of the soul) — "where the spirit lives". Inspired by Tamagotchi, but taken further: each creature carries a genetic blueprint that shapes its personality, appearance, and behaviour. No two Kobaras are alike — just like in nature.

---

## What makes Kokoro different

In most virtual pet games, creatures are skins over the same logic. In Kokoro:

- **Genetics drive personality.** Each Kobara is born with a genome — a set of genes (values between 0.0 and 1.0) that determine appetite, curiosity, emotional resilience, sleep patterns, and more. The species sets the possible ranges; the individual fills them in.
- **Behaviour emerges from rules + randomness.** A hungry, shy Kobara at night won't ask for food — it hides. A curious, high-energy one in the afternoon might bounce around unprompted. Simple rules, complex outcomes.
- **A local neural network learns your Kobara specifically.** Over time, the creature's responses adapt to *your* interaction patterns. It's not global AI — it runs on-device, belongs only to this creature, and evolves with it.

---

## Species

All creatures are Kobaras. The **species** determines their physical form:

| Species | Kanji | Meaning | Type | Status |
|---------|-------|---------|------|--------|
| **Marumi** | 丸み | roundness | Mammal-like | Implemented |
| **Tsubasa** | 翼 | wing | Bird-like | Planned |
| **Uroko** | 鱗 | scale | Reptile-like | Planned |

Each species has its own **body rig** (a proportional landmark system, like facial mapping polygons) that defines face shape and body proportions. The genome nudges these landmarks to produce unique individuals within the same species.

---

## Architecture

```
src/
├── genome/           # DNA: 7 genes per Kobara (curiosity, appetite, hue, etc.)
├── mind/             # AI engine: FSM mood states + emergent behaviour rules
├── persistence/      # SQLite: save/load + interaction event log
├── systems/
│   ├── rig.rs            # Proportional landmark system (face/body mapping)
│   ├── body_parts.rs     # Species visual templates (parts, fallbacks)
│   ├── creature_spawn.rs # Assembles creature from rig + sprites
│   ├── mood_sync.rs      # Swaps eyes/mouth sprites on mood change
│   ├── genome_visuals.rs # Genome → tint color, body scale
│   ├── animation.rs      # Idle sway + eye blink
│   ├── effects.rs        # Mood effects (zzz, hearts, rain cloud, stars)
│   ├── evolution.rs      # Growth stages: Baby → Child → Adult → Elder
│   ├── stats.rs          # HUD displaying vital stats
│   ├── time_tick.rs      # Game tick (1/sec) + circadian bonus
│   └── ui/actions.rs     # Player buttons: Feed, Play, Sleep
├── world/
│   ├── mod.rs            # Camera setup
│   └── daycycle.rs       # Background color based on system clock
└── main.rs               # App entry point
```

### The creature's mind (AI layers)

```
┌─────────────────────────────────────────┐
│  Finite State Machine                   │  ← current mood: Happy, Hungry, Lonely...
│  mood transitions driven by vital stats │
├─────────────────────────────────────────┤
│  Emergent behaviour rules               │  ← genome x stats x randomness
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
| `hue` | --- | Determines body colour (0-360) |

### Body rig (landmark system)

Instead of hardcoded pixel positions, each species defines **anchor points** in a normalized `[-1, 1]` coordinate space. The genome shifts these anchors to produce unique face shapes:

```
        (-1, 1) ────────── (1, 1)
           |    ear   ear    |
           |   eye_L  eye_R  |     ← landmarks
           |     mouth       |
       (-1, -1) ────────── (1, -1)
```

- **Same species, different genome** = slightly different face proportions
- **Different species** = radically different rig (forward-facing predator eyes vs side-facing herbivore eyes)

---

## Tech stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2024) |
| Game engine | [Bevy](https://bevyengine.org/) 0.16 (ECS-first, mobile-ready) |
| Serialisation | `serde` + `bincode` |
| Randomness | `rand` |
| Persistence | `rusqlite` (SQLite embedded) |
| Neural network (Phase 4) | Manual implementation or `candle-core` |
| Target platforms | Desktop, Android, iOS |

---

## Roadmap

### Phase 1 — Foundation
- [x] Bevy project structure
- [x] Genome system with random generation
- [x] Finite state machine (FSM) mood engine
- [x] Emergent behaviour rules driven by genes
- [x] Procedural creature rendering (colour from genome)
- [x] HUD displaying vital stats and mood

### Phase 2 — Memory & persistence
- [x] SQLite persistence via `rusqlite`
- [x] Save/load creature state between sessions
- [x] Interaction history log (events table)

### Phase 3 — Visuals & interaction
- [x] Action buttons (Feed, Play, Sleep)
- [x] Modular sprite composition (body, eyes, mouth, ears)
- [x] Body rig / proportional landmark system
- [x] Mood-reactive sprite swapping (eyes + mouth)
- [x] Genome-driven visuals (tint, body scale, eye spacing)
- [x] Day/night cycle from system clock
- [x] Visual effects per mood (zzz, hearts, rain cloud, stars)
- [x] Idle animation (body sway + eye blink)
- [x] Visual evolution / growth stages
- [x] Programmatic sprite generation (Python/PIL)
- [ ] Refine sprite quality to match reference art
- [ ] Sound cues tied to mood states
- [ ] Android + iOS build via `cargo-mobile2`

### Phase 4 — Neural mind
- [ ] Small MLP (12-8-7, ~182 parameters) in pure Rust
- [ ] Trains locally on interaction history from SQLite
- [ ] Creature adapts to owner's daily patterns
- [ ] FSM keeps veto power on critical states (Sick, Sleeping)
- [ ] Each Kobara's network is unique and non-transferable

### Phase 5 — Expansion
- [ ] New species: Tsubasa (bird), Uroko (reptile) with unique rigs
- [ ] Accessories and visual marks at age milestones
- [ ] Reproduction / genetic inheritance between Kobaras
- [ ] Multi-creature interaction

---

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)

### Run on desktop

```bash
git clone https://github.com/Edrick42/kokoro
cd kokoro
cargo run
```

### Generate sprites

```bash
python3 scripts/generate_marumi_sprites.py
```

Sprites are output to `assets/sprites/marumi/`. The game falls back to procedural meshes if sprites are missing.

### Sprite structure

```
assets/sprites/
├── marumi/                  # Marumi species sprites
│   ├── body_idle.png
│   ├── ear_left_idle.png
│   ├── ear_right_idle.png
│   ├── eye_left_{mood}.png  # 8 mood variants
│   ├── eye_right_{mood}.png
│   └── mouth_{mood}.png     # 8 mood variants
└── shared/effects/          # Mood visual effects
    ├── zzz.png
    ├── hearts.png
    ├── rain_cloud.png
    ├── stars_dizzy.png
    └── sparkle.png
```

---

## Contributing

This project is an open learning exercise in Rust. Issues, suggestions, and PRs are welcome.

---

## License

MIT
