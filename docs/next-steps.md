# Kokoro — Roadmap (April 2026)

> Updated after soft body physics implementation. Reflects actual project state.

## Project Status

```
Codebase:  204 files, 18K+ lines of game code
Plugins:   45 Bevy plugins
Species:   4 (Moluun fully animated, 3 with basic pixel art)
Ebook:     58 chapters written, Chapter 0 learning path added
API:       17 endpoints (auth, sync, Stripe, GDPR)
Web:       Leptos SSR frontend (login, profile, sync)
```

### What's Done (MVP features)

| Feature | Status |
|---------|--------|
| 4 species with unique behaviors | DONE |
| Egg → Cub → Young → Adult → Elder lifecycle | DONE |
| Nutrition (24 foods × 7 nutrients) | DONE |
| Touch interaction (body part sensitivity) | DONE |
| Neural network learning | DONE |
| Soft body physics (Moluun) | DONE |
| Chewing animation + expression system | DONE |
| Absence awareness (Mirror Bond) | DONE |
| Persistence (SQLite, multi-creature) | DONE |
| Autonomic nervous system | DONE |
| Anatomy (skeleton, muscles, skin) | DONE |
| Disease system (5 conditions) | DONE |
| Hygiene system | DONE |
| Dev mode (F12 panels, stage selector) | DONE |
| Web API (Axum, 17 endpoints) | DONE |
| Web frontend (Leptos SSR) | DONE |
| Freemium model (Stripe checkout) | DONE |

### What's NOT Done

| Feature | Priority | Why it matters |
|---------|----------|----------------|
| Soft body for Pylum, Skael, Nyxal | HIGH | 3 species feel stiff |
| Background environment art | HIGH | Screen feels empty |
| Sound/audio implementation | HIGH | Silent game feels dead |
| Species-specific pixel art polish | MEDIUM | Pylum/Skael/Nyxal look basic |
| Breeding (P2P) | LOW | Phase 12, future |
| Mobile build | LOW | After desktop polish |

---

## The Roadmap: 6 Sprints

### Sprint 1: "Each Species Has a Soul" (1-2 weeks)
*Soft body + pixel art for all 4 species*

**Goal:** Every species moves uniquely. Pylum flaps, Skael slithers, Nyxal pulses.

| Task | Files | Concept learned |
|------|-------|-----------------|
| Pylum soft body (wings, beak, tail feathers) | soft_body.rs | Point layout for flight anatomy |
| Skael soft body (long tail chain, low body) | soft_body.rs | Multi-segment tail physics |
| Nyxal soft body (tentacles, mantle) | soft_body.rs | Hydrostatic skeleton (boneless) |
| Species-specific impulses | impulse.rs | Different reactions per anatomy |
| Moluun Young soft body | soft_body.rs | Intermediate layouts |
| `SpeciesPhysics` trait | new trait | **Rust Phase 2: custom traits** |
| Wire all species to spawn system | spawn.rs | Match exhaustiveness |

**Point layouts (design):**

```
PYLUM (bird):             SKAEL (reptile):          NYXAL (cephalopod):
    beak                      head                       mantle_top
     |                         |                            |
    head                     body----tail_1               mantle
   / | \                    / | \      |                  / | \
wing_l body wing_r      leg_l leg_r tail_2         tent_fl  eye  tent_fr
   |   |   |                        |              tent_bl      tent_br
 tip_l tail tip_r                 tail_3                |
       |                                             tent_tip
     feet
```

**Deliverable:** `cargo run` → switch between 4 species, each has unique physics movement.

---

### Sprint 2: "The World Breathes" (1 week)
*Background art + sound foundation*

**Goal:** The screen isn't just a creature on blank canvas. Each biome has atmosphere.

| Task | Files | Notes |
|------|-------|-------|
| 4 biome backgrounds (pixel art, 320×320) | new: visuals/background.rs | Verdance (forest), Highlands (cliff), Ashveil (desert), Abyssal (deep) |
| Day/night tint on backgrounds | background.rs | Reuse existing DayCycle |
| Ambient sound per biome | audio/ | .ogg files in assets/sounds/ |
| Creature vocalizations (3 per species) | audio/ | Happy chirp, hungry whine, sleepy sigh |
| Heartbeat audio synced with BPM | audio/ | Already has BPM calc, just needs playback |

**Deliverable:** Game has atmosphere — forest sounds, creature chirps, visible biome.

---

### Sprint 3: "The Book Teaches" (2 weeks)
*Ebook chapters that match the code*

**Goal:** Every chapter the reader finishes gives them a working feature AND a Rust concept.

| Task | Chapters | Rust concept |
|------|----------|--------------|
| Finish Part I (Ch 1-10) | Foundations | Ownership, borrowing, traits, errors |
| Write Ch 11-16 (Bevy + ECS) | Game core | Systems, Resources, Events, Queries |
| Write Ch 25 (sprite pipeline rewrite) | Visuals | Runtime pixel art, change detection |
| Update Ch 27 with soft body | Animation | Physics simulation, impulses |
| Add exercises to all chapters | All | 3 levels: Practice, Build, Explore |

**Rust study integration:** As you write each chapter, apply the Phase from Chapter 0:
- Writing Ch 7 (errors)? Apply Phase 3 — replace `.expect()` with `?` in Kokoro first
- Writing Ch 9 (traits)? Apply Phase 2 — create `SpeciesPhysics` trait first
- Writing Ch 15 (mind FSM)? Add `Copy` to `MoodState` first (Phase 1 task)

**Deliverable:** Parts I-IV complete and publishable.

---

### Sprint 4: "Polish Until It Shines" (1-2 weeks)
*Bug fixes, UX, performance*

| Task | Why |
|------|-----|
| Fix all TODO comments (4 remaining) | Clean codebase |
| Creature revival after death (new egg option) | Death shouldn't be permanent frustration |
| Onboarding flow (species selection + first egg) | New player experience |
| UI animation polish (menu transitions) | Feels professional |
| Performance profiling (60fps guarantee) | Can't ship janky |
| 50+ unit tests | Confidence to refactor |

**Rust study integration:** Phase 3 — refactor persistence layer to use `KokoroError` enum instead of `.expect()`. This is real polish that also teaches error handling.

**Deliverable:** Someone could download this and enjoy it for a week.

---

### Sprint 5: "The World Sees It" (1-2 weeks)
*Distribution + early access*

| Task | Platform |
|------|----------|
| Build for macOS + Windows + Linux | cargo build --release |
| itch.io page with screenshots | Desktop distribution |
| Gumroad ebook early access (Parts I-IV) | $9.99 |
| GitHub readme with GIFs | Developer audience |
| First dev log video | YouTube/TikTok |

**Deliverable:** Real humans playing Kokoro and reading the ebook.

---

### Sprint 6: "Mobile + Breeding" (future)
*After desktop validation*

| Task | When |
|------|------|
| Mobile build (cargo-mobile2) | After 100+ desktop users |
| Breeding system (P2P via Bluetooth/LAN) | After mobile |
| Premium features (cosmetics, extra slots) | After 1K users |
| Scale API (SQLite → PostgreSQL) | After 5K DAU |
| Ebook Parts IX-X (P2P, Sensory) | After code exists |

---

## Decision Log

**Why Sprint 1 first (species physics) instead of backgrounds/sound?**
Because the core product is the creature. If all 4 species feel alive and unique,
the game has value even on a blank screen. Backgrounds are decoration.
A beautifully decorated game with stiff creatures is worthless.

**Why ebook Sprint 3 is in the middle, not at the end?**
Because writing forces understanding. Writing Ch 9 (traits) while implementing
`SpeciesPhysics` creates a feedback loop: the code teaches the writing, the
writing reveals gaps in the code. Do them together.

**Why not mobile first?**
Desktop is faster to iterate. Debug on desktop, port when stable.
Mobile adds build complexity, touch input differences, and app store review.
Ship desktop, validate the core loop, then port.
