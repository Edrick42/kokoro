# Kokoro — MVP Definition (Minimum Viable Product)

> The first version that someone would download, bond with a creature, and tell a friend about.

## Core Loop

```
HATCH → CARE → BOND → GROW → SHARE
```

1. **Hatch**: Egg appears, player incubates it, creature is born
2. **Care**: Feed (with different foods), play, sleep, groom, pet
3. **Bond**: Creature learns your patterns, develops preferences, reacts to absence
4. **Grow**: Baby → Juvenile → Adult → Elder with visible changes
5. **Share**: Web profile where your Kobara lives online (optional)

## MVP Feature Set

### Must Have (Release Blocker)

| Feature | Status | Notes |
|---------|--------|-------|
| 4 species with unique behaviors | DONE | Moluun, Pylum, Skael, Nyxal |
| Egg/birth stage per species | DONE | Incubation ~3 days, tap to warm |
| Realistic time cycles | DONE | Gradual transitions, mood cooldown, ~6 month lifespan |
| Nutrition system (7 nutrients × 8 foods) | DONE | Real biology: protein, carbs, fat, water, minerals, vitamins, fiber |
| Touch interaction (nervous system) | DONE | Sensitivity per body part (pleasure/pain/warmth), raycasting |
| Learning & preferences | DONE | Food memory, refusal after 10x, creature requests |
| Communication system (5 channels) | DONE | Vocal, visual, kinetic, chemical, tactile per species |
| Complete lifecycle (death) | DONE | Aging, starvation, dehydration, neglect, care quality |
| Physics + species animations | DONE | Gravity, bounce, buoyancy, idle behaviors |
| Breathing + heartbeat vitals | DONE | Visible in UI panel |
| Kokoro-sac glow | DONE | Emotional resonance visualization |
| Absence awareness (Mirror Bond) | DONE | Creature reacts when player returns |
| Species personality | DONE | Different stat impacts per species |
| Persistence (all creatures saved) | DONE | SQLite, multi-creature |
| Neural network learning | DONE | MLP learns owner patterns |
| UI design system + animations | DONE | Animated buttons, hierarchical menu, food icons |
| Growth stages with visuals | DONE | Egg → Cub → Young → Adult → Elder |
| Config organized (0 magic numbers) | DONE | src/config/ with 9 submodules |
| 30 unit tests | DONE | Genome, mind, nutrition, preferences, touch, lifecycle |
| Sound infrastructure | DONE | VocalRepertoire, species learning limits, placeholder |
| Background/environment art | TODO | Biome behind each species |

### Nice to Have (Post-MVP)

| Feature | Priority |
|---------|----------|
| Metabolism system (fat, energy burn) | High |
| Biological systems (skeleton, muscles) | High |
| Natural abilities (electric sense, echolocation) | Medium |
| Communication system (scent, sound, color) | Medium |
| Hygiene system | Medium |
| Taxonomy classification | Low (organizational) |
| Web profile (Axum + Leptos) | High |
| Auth/login | High (for web) |
| Breeding (mobile P2P) | Future |

## MVP Target Platforms

1. **Desktop** (macOS/Windows/Linux) — primary development
2. **Mobile** (iOS/Android via cargo-mobile2) — target for user acquisition
3. **Web** (companion site) — profiles, lore, community

## MVP Timeline Estimate

| Phase | Duration | What |
|-------|----------|------|
| Egg + time cycles + nutrition | 2-3 weeks | Core lifecycle feels real |
| Touch + learning + preferences | 2-3 weeks | Interaction depth |
| Sound + environment art | 1-2 weeks | Polish |
| Testing + bug fixes | 1 week | Stability |
| **Total to MVP** | **6-9 weeks** | |

## Success Metrics

- Player opens the app daily (retention)
- Player names their creature (emotional investment)
- Player tells someone about it (word of mouth)
- Player feels guilty when they neglect it (Mirror Bond works)
- Player is surprised by creature behavior (neural network + preferences)
