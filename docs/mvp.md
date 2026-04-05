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
| Egg/birth stage per species | TODO | First thing the player sees |
| Realistic time cycles | TODO | Gradual mood transitions, sleep takes time |
| Nutrition system (3+ food types) | TODO | Not just "Feed" — protein, fruit, minerals |
| Touch interaction (pet/scratch) | TODO | Core engagement — touching the creature |
| Learning & preferences | TODO | Creature refuses food it dislikes, requests activities |
| Physics + species animations | DONE | Gravity, bounce, buoyancy, idle behaviors |
| Breathing + heartbeat vitals | DONE | Visible in UI panel |
| Kokoro-sac glow | DONE | Emotional resonance visualization |
| Absence awareness (Mirror Bond) | DONE | Creature reacts when player returns |
| Species personality | DONE | Different stat impacts per species |
| Persistence (all creatures saved) | DONE | SQLite, multi-creature |
| Neural network learning | DONE | MLP learns owner patterns |
| Clean UI (collapsible menu) | DONE | Minimal, not cluttered |
| Growth stages with visuals | DONE | Baby → Elder with scale changes |
| Sound effects | TODO | Ambient + creature sounds |
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
