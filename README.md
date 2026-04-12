# Kokoro

> *Kokoro* (心) means "heart" and "mind" in Japanese — the place where feeling and awareness are the same thing.

A virtual creature bio-simulation built entirely in Rust. Every **Kobara** is born with a unique genome that shapes its personality, appearance, and behaviour. No two creatures are alike — just like in nature.

**Game** (Bevy) + **Web API** (Axum) + **Frontend** (Leptos SSR) — full-stack Rust.

---

## The Four Species

Every creature is a Kobara. The **species** determines its body plan, adapted to one of Ethara's biomes:

| Species | Biome | Body Plan | Personality |
|---------|-------|-----------|-------------|
| **Moluun** | The Verdance (forests) | Round, soft, dense fur | Social, emotionally open |
| **Pylum** | Veridian Highlands (mesas) | Winged, aerodynamic, beaked | Curious, restless explorer |
| **Skael** | Abyssal Shallows (caves) | Scaled, crested, muscular tail | Quiet, fiercely loyal |
| **Nyxal** | Abyssal Depths (deep ocean) | Tentacled, bioluminescent | Intelligent, pattern-thinker |

---

## What Makes Kokoro Different

- **Genetics drive everything.** 7 genes (curiosity, appetite, resilience, circadian, loneliness sensitivity, learning rate, hue) shape behaviour and appearance. Same species, different genome = different creature.
- **Neural network learns YOU.** A small MLP (167 parameters) trains on your interaction history. Over time, the creature adapts to your patterns — on-device, unique to each Kobara.
- **Anatomy is real.** Skeleton, joints, muscles, skin — four layers that cascade. Break a bone, the connected joints lock up, muscles weaken, energy drains. Good care heals from inside out.
- **Death is real.** Starvation, neglect, old age. A well-loved Kobara lives longer. When it dies, you remember it.
- **Runtime pixel art.** No pre-made sprites. Each creature is drawn to a 64x64 pixel buffer every frame from primitives — circles, ellipses, rectangles. Species, growth stage, mood, and genome all affect the result.

---

## Game Flow

```
Loading → Title Screen → Auth (Login / Register / Guest) → Onboarding (6 steps)
    → Gameplay ↔ Death Screen
         │
         └→ Side Menu: Profile, Settings, Lore, Shop
```

---

## Architecture

```
kokoro/
├── src/
│   ├── main.rs              # 36 plugins registered
│   ├── genome/              # 7 genes, species, crossover
│   ├── mind/                # FSM (7 moods), neural network, nutrition, lifecycle, preferences
│   ├── creature/            # anatomy (skeleton→joints→muscles→skin), physics, touch, spawn, egg
│   ├── visuals/             # pixel_creature (64x64 runtime), breathing, effects, evolution, accessories
│   ├── audio/               # sound bank, species sounds, heartbeat, voice composer
│   ├── ui/                  # title, auth, onboarding, death, HUD, vitals, actions, side menu, text input
│   ├── web/                 # HTTP client (auth + creature sync via API)
│   ├── game/                # state machine (7 states)
│   ├── persistence/         # SQLite save/load, absence system
│   ├── config/              # 15 submodules, zero magic numbers
│   └── world/               # camera, day/night cycle, time tick
├── kokoro-shared/           # shared types (species, food, genes, biomes, shop) — used by game + API
└── kokoro-web/
    ├── api/                 # Axum REST API (auth, creature sync, shop/Stripe)
    └── ui/                  # Leptos SSR frontend (login, register, profile, species viewer)
```

### Numbers

| Metric | Value |
|--------|-------|
| .rs files | 91 |
| Lines of Rust | ~13,400 |
| Bevy plugins | 36 |
| Tests | 43 |
| Compiler warnings | 0 |
| Species | 4 |
| Growth stages | 5 (egg, cub, young, adult, elder) |

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2024) |
| Game engine | Bevy 0.16 |
| Web API | Axum 0.8 + Tokio |
| Web frontend | Leptos 0.8 (SSR + WASM hydration) |
| Database | SQLite (rusqlite) — game + API |
| Auth | JWT (jsonwebtoken) + Argon2 password hashing |
| Payments | Stripe (async-stripe) |
| Shared types | kokoro-shared crate |
| Serialization | Serde + bincode (game), Serde + JSON (API) |
| Pixel art | `image` crate — runtime 64x64 rendering |

---

## API Endpoints

| Endpoint | Method | Auth | Purpose |
|----------|--------|------|---------|
| `/health` | GET | — | Server status |
| `/api/species` | GET | — | List all species |
| `/api/species/{name}` | GET | — | Species details + gene ranges |
| `/api/biomes` | GET | — | List biomes |
| `/api/foods` | GET | — | List food types |
| `/auth/register` | POST | — | Create account |
| `/auth/login` | POST | — | Get JWT token |
| `/auth/profile` | GET | JWT | User profile |
| `/api/creature` | GET | JWT | Download creature state |
| `/api/creature/sync` | POST | JWT | Upload creature state |
| `/api/shop/balance` | GET | JWT | Crystal balance |
| `/api/shop/checkout` | POST | JWT | Stripe checkout session |
| `/api/shop/webhook` | POST | Stripe | Payment fulfillment |
| `/api/shop/purchase` | POST | JWT | Spend crystals on item |

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)

### Run the Game

```bash
git clone https://github.com/Edrick42/kokoro
cd kokoro
cargo run
```

### Run the API

```bash
cd kokoro-web
cargo run -p kokoro-api
# → http://localhost:8080
```

### Run the Web Frontend

```bash
cd kokoro-web/ui
cargo leptos watch
# → http://localhost:3000
```

### Environment Variables (API)

Copy `kokoro-web/api/.env.example` to `.env`:

```bash
STRIPE_SECRET_KEY=sk_test_your_key_here
SHOP_SUCCESS_URL=http://localhost:3000/profile?purchase=success
SHOP_CANCEL_URL=http://localhost:3000/profile?purchase=cancelled
```

---

## The Genome

7 genes on a spectrum, each f32 between 0.0 and 1.0:

| Gene | Low (0.0) | High (1.0) |
|------|-----------|------------|
| Curiosity | Calm, apathetic | Always exploring |
| Appetite | Slow metabolism | Gets hungry fast |
| Loneliness Sensitivity | Independent | Needs constant company |
| Circadian | Night owl | Day creature |
| Resilience | Fragile, moody | Bounces back quickly |
| Learning Rate | Slow learner | Adapts fast |
| Hue | 0 | 360 (body colour) |

Species constrain the ranges. A Skael can't have curiosity below 0.1 or resilience below 0.5. Within those bounds, every creature is unique.

---

## Lifecycle

```
Egg → Cub → Young → Adult → Elder → Death
```

| Stage | Duration | Visual |
|-------|----------|--------|
| Egg | ~2 min | Species-colored oval with pattern |
| Cub | ~2 weeks | Huge head, huge eyes (Kindchenschema) |
| Young | ~6 weeks | Appendages emerging, body elongating |
| Adult | ~3 months | Full features, mature proportions |
| Elder | 2-3 months | Faded colors, wisdom marks |

Good care extends lifespan up to 30%. Neglect shortens it.

---

## Roadmap

| Phase | Status |
|-------|--------|
| 1-8: Core systems, visuals, persistence, audio | Done |
| 9: Onboarding + side menu + shop scaffold | Done |
| 10: Biological depth (metabolism, abilities, hygiene) | Planned |
| 11: Web enhancement (auth, sync, Stripe) | Done |
| 12: Breeding + mobile (cargo-mobile2) | Planned |
| 13: Kobara encounters (P2P proximity interaction) | Planned |
| 14: Sensory perception (voice tone, touch quality, presence) | Planned |
| 15: Polish + launch | Planned |

---

## The Book

Kokoro is also a **commercial Rust ebook** — learn Rust by building this game from scratch.

- 57 planned chapters across 10 parts
- 23 chapters written (~8,600 lines)
- Part I (Rust foundations): complete
- Part VII (Full-stack web): nearly complete
- Covers: ownership, traits, ECS, pixel art, neural networks, web APIs, Stripe, P2P

---

## License

MIT
