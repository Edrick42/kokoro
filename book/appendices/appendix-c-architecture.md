# Appendix C: Complete Kokoro Architecture

## Project Structure

```
kokoro/
├── Cargo.toml                    # Dependencies, [features] dev = bevy_egui
├── book/                         # This ebook (47 chapters)
├── docs/
│   ├── lore.md                   # World bible: Ethara, 4 species, resonance
│   ├── mvp.md                    # MVP feature checklist
│   ├── species-evolution-design.md # Visual progression per stage
│   ├── next-steps.md             # Ordered task list
│   ├── art-direction.md          # Low-poly style guide
│   └── species-evolution-design.md # Visual progression per stage
├── kokoro-web/                   # Full-stack Rust companion website
│   ├── api/                      # Axum REST API
│   └── ui/                       # Leptos frontend (planned)
├── assets/
│   └── sprites/
│       ├── moluun/{egg,cub,young,adult,elder}/
│       ├── pylum/{egg,cub,young,adult,elder}/
│       ├── skael/{egg,cub,young,adult,elder}/
│       ├── nyxal/{egg,cub,young,adult,elder}/
│       └── shared/effects/
└── src/
    ├── main.rs                   # App entry, plugin orchestration
    ├── audio/                    # Sound system
    │   └── mod.rs                # SoundPlugin, VocalRepertoire, GameSound enum
    ├── config/                   # All tunable constants (no magic numbers)
    │   ├── mod.rs                # Re-exports
    │   ├── stats.rs              # Vital stats, FSM thresholds, decay rates
    │   ├── species.rs            # Feed/play/sleep values per species
    │   ├── physics.rs            # Gravity, bounce, friction, impulses
    │   ├── biology.rs            # Breathing, heartbeat, resonance, growth, egg
    │   ├── timing.rs             # Tick interval, autosave, neural, circadian
    │   ├── absence.rs            # Mirror Bond time brackets
    │   └── slots.rs              # Body part string constants
    ├── creature/                 # Creature lifecycle
    │   ├── mod.rs
    │   ├── collection.rs         # Multi-creature management (4 species)
    │   ├── egg.rs                # Egg stage, incubation, hatching
    │   ├── physics.rs            # Mini physics: gravity, bounce, buoyancy
    │   ├── reproduction.rs       # Breeding system (future — mobile P2P)
    │   ├── rig.rs                # Body rig landmark system (4 rigs)
    │   ├── spawn.rs              # Entity spawning + stage-aware sprite loading
    │   └── species.rs            # Species templates and part definitions
    ├── genome/                   # Genetic blueprint
    │   ├── mod.rs                # Genome struct (8 genes), random generation
    │   ├── species.rs            # Species enum (Moluun, Pylum, Skael, Nyxal)
    │   ├── crossover.rs          # Genetic crossover + mutation
    │   └── color.rs              # Genome → HSL body/tint color
    ├── mind/                     # AI and emotional state
    │   ├── mod.rs                # Mind struct, MoodState, VitalStats, FSM
    │   ├── absence.rs            # Mirror Bond (absence awareness)
    │   ├── lifecycle.rs           # Aging, death, care quality tracking
    │   ├── neural.rs             # MLP neural network (12→8→7, 167 params)
    │   ├── nutrition.rs          # NutrientState (7 nutrients), decay, deficiency
    │   ├── preferences.rs        # Food memory, creature requests, refusal
    │   ├── plugin.rs             # Bevy integration, training schedule
    │   └── training.rs           # Training pipeline, event extraction
    ├── persistence/              # SQLite save/load
    │   ├── mod.rs
    │   ├── db.rs                 # Schema creation + migrations
    │   ├── load.rs               # Load genome/mind/collection from DB
    │   ├── save.rs               # Save genome/mind/collection/neural to DB
    │   └── plugin.rs             # Startup load, periodic save, exit save
    ├── ui/                       # User interface
    │   ├── mod.rs
    │   ├── actions.rs            # Collapsible "..." menu (food, play, sleep, species)
    │   ├── hud.rs                # Stat bar display (top-left)
    │   ├── style.rs              # UI design system (colors, sizes, button animations)
    │   └── vitals.rs             # BPM + breathing rate panel (top-right)
    ├── visuals/                  # Graphics and animation
    │   ├── mod.rs
    │   ├── accessories.rs        # Milestone rewards (ribbon, scarf, crown)
    │   ├── animation.rs          # Eye blink system
    │   ├── breathing.rs          # Breathing + heartbeat (scale oscillation + BPM)
    │   ├── effects.rs            # Floating mood effect sprites
    │   ├── evolution.rs          # Growth stages (egg → elder)
    │   ├── genome_visuals.rs     # Genome → visual appearance (tint, scale)
    │   ├── mood_sync.rs          # Mood → sprite swaps
    │   ├── resonance_glow.rs     # Kokoro-sac glow (pulsing circle)
    │   └── species_behavior.rs   # Per-species idle animations
    ├── dev/                      # Dev Mode (feature-gated: --features dev)
    │   ├── mod.rs                # DevPlugin, F12 toggle, DevModeState
    │   ├── rig_gizmos.rs         # Skeleton visualization with Bevy Gizmos
    │   └── panels.rs             # egui panels: stats, genome, neural, physics, cheats
    └── world/                    # Environment
        ├── mod.rs                # Camera setup
        ├── daycycle.rs           # Day/night cycle (system clock)
        └── time_tick.rs          # Game tick system (1 tick = 1 second)
```

## Plugin Registration Order

```rust
let mut app = App::new();

app.add_plugins(DefaultPlugins)              // Bevy built-ins
    .add_plugins(PersistencePlugin)          // 1. Load saved state
    .add_systems(Startup, setup_world)       // 2. Camera
    .add_plugins(CreatureVisualsPlugin)      // 3. Spawn creature entities
    .add_plugins((DayCyclePlugin, TimeTickPlugin))  // 4. World systems
    .add_plugins((NeuralMindPlugin, NutritionPlugin))  // 5. AI + Nutrition
    .add_plugins((StatsPlugin, ActionsPlugin, VitalsPlugin))  // 6. UI
    .add_plugins((MultiCreaturePlugin, EggPlugin))   // 7. Lifecycle
    .add_plugins(PhysicsPlugin)              // 8. Gravity, collision, buoyancy
    .add_plugins((EffectsPlugin, AnimationPlugin, EvolutionPlugin, AccessoriesPlugin))
    .add_plugins((BreathingPlugin, SpeciesBehaviorPlugin, ResonanceGlowPlugin))
    .add_systems(Update, (sync_mood_sprites, apply_genome_visuals));

#[cfg(feature = "dev")]
app.add_plugins(dev::DevPlugin);             // Dev Mode (F12 toggle)

app.run();
```

## The Four Species

| Species | Habitat | Body Plan | Unique Features |
|---------|---------|-----------|-----------------|
| **Moluun** | The Verdance (forest) | Bipedal, round | Ears (twitch), rosy cheeks, dormant lateral eyes |
| **Pylum** | Veridian Highlands | Avian, winged | Wings (flutter), beak, tail feathers, tuft crest |
| **Skael** | Abyssal Shallows (caves) | Reptilian, elongated | Crests/horns, slit-pupil eyes, snout, scaled tail |
| **Nyxal** | Abyssal Depths (deep sea) | Cephalopod, tentacled | 4 tentacles (undulate), mantle, bioluminescent eyes |

## Growth Stages

| Stage | Age (ticks) | Real Time (~) | Scale | Sprite Directory |
|-------|-------------|---------------|-------|-----------------|
| Egg | incubation | ~3 days | 0.4 | `{species}/egg/` |
| Cub | 0–1,200,000 | ~2 weeks | 0.6 | `{species}/cub/` |
| Young | →3,800,000 | ~6 weeks | 0.8 | `{species}/young/` |
| Adult | →8,500,000 | ~3.3 months | 1.0 | `{species}/adult/` |
| Elder | 8,500,000+ | ~2-3 months more | 0.95 | `{species}/elder/` |
| Death | species-dependent | ~6-7 months total | — | — |

## Genome — 8 Genes

| Gene | Range | Behavior Effect | Visual Effect |
|------|-------|----------------|---------------|
| `curiosity` | 0.0–1.0 | More likely to enter Playful mood | Eye spacing |
| `loneliness_sensitivity` | 0.0–1.0 | Lonely instead of Tired when unhappy | — |
| `appetite` | 0.0–1.0 | Faster hunger growth | Body width |
| `circadian` | 0.0–1.0 | Night owl vs early bird | — |
| `resilience` | 0.0–1.0 | Faster emotional recovery | Eye height |
| `learning_rate` | 0.0–1.0 | Neural network learns faster | — |
| `hue` | 0.0–360.0 | — | Body/part tint color (HSL) |

## Config Module (`src/config/`)

All tunable game constants organized by domain — zero magic numbers in game logic:

| Submodule | What It Contains |
|-----------|-----------------|
| `stats.rs` | Initial vitals, FSM thresholds (per species), decay rates, mood cooldown |
| `species.rs` | Feed/play/sleep stat changes per species |
| `physics.rs` | Gravity, bounce, friction, impulses, velocity thresholds |
| `biology.rs` | Breathing rates, heartbeat BPM, resonance frequencies, growth stages, egg timing |
| `nutrition.rs` | FoodType enum, nutrient profiles (7 nutrients × 8 foods), species decay rates |
| `lifecycle.rs` | Lifespan per species (~6 months), aging, death conditions, care quality |
| `communication.rs` | 5 communication channels (vocal/visual/kinetic/chemical/tactile) per species |
| `nervous_system.rs` | Sensory sensitivity (pleasure/pain/warmth) per species × body part |
| `timing.rs` | Tick interval, autosave, neural training schedule, circadian bonuses |
| `absence.rs` | Mirror Bond time brackets (1min, 30min, 4h, 24h), reunion ticks |
| `slots.rs` | Body part name constants (prevents typos) |

## Neural Network Architecture

```
Input (12)              Hidden (8)           Output (7)
─────────              ──────────           ──────────
hunger/100      ─┐
happiness/100    │     ┌─ neuron 0 ─┐
energy/100       ├────▶│  neuron 1  ├──────▶ Happy
health/100       │     │  neuron 2  │        Hungry
curiosity        │     │  neuron 3  │        Tired
loneliness_s     ├────▶│  neuron 4  ├──────▶ Lonely
appetite         │     │  neuron 5  │        Playful
circadian        │     │  neuron 6  │        Sick
resilience       ├────▶│  neuron 7  │        Sleeping
learning_rate    │     └────────────┘
hue/360          │       ReLU activation     Softmax
hour/24         ─┘                           (probabilities)

Total parameters: 12×8 + 8 + 8×7 + 7 = 167
Training: every 120 ticks, 5 epochs, max 200 samples
Influence: 0% → 60% (grows logarithmically with sessions)
```

## Dev Mode (`cargo run --features dev`)

Toggle with **F12**. Panels:

| Panel | What It Shows |
|-------|--------------|
| **Stats** | Mood, hunger/happiness/energy/health bars, age, FSM vs NN comparison |
| **Genome** | All 8 gene values as progress bars, hue color swatch |
| **Neural** | Influence %, maturity, sessions, loss, live 7-mood prediction bars |
| **Physics** | Velocity, grounded state, buoyancy, breathing rate, BPM, kokoro-sac Hz |
| **Cheats** | Speed slider, skip time, max stats, force mood, hatch egg |
