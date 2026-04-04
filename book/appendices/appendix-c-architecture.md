# Appendix C: Complete Kokoro Architecture

## Project Structure

```
kokoro/
├── Cargo.toml                    # Dependencies, [features] dev = bevy_egui
├── book/                         # This ebook
├── docs/
│   └── lore.md                   # World bible: Ethara, Kobaras, resonance
├── assets/
│   └── sprites/
│       ├── moluun/               # Forest mammal sprites (28 files)
│       ├── pylum/                # Highland bird sprites (25 files)
│       ├── skael/                # Cave reptile sprites (25 files)
│       ├── nyxal/                # Abyssal squid sprites (20 files)
│       └── shared/               # Effects, icons, UI elements
└── src/
    ├── main.rs                   # App entry, plugin registration
    ├── bin/                      # Sprite generators (one per species)
    │   ├── sprite_common/        # Shared pixel-art toolkit
    │   ├── generate_moluun_sprites.rs
    │   ├── generate_pylum_sprites.rs
    │   ├── generate_skael_sprites.rs
    │   └── generate_nyxal_sprites.rs
    ├── creature/                 # Creature lifecycle
    │   ├── mod.rs
    │   ├── collection.rs         # Multi-creature management (4 species)
    │   ├── physics.rs            # Mini physics: gravity, bounce, buoyancy
    │   ├── reproduction.rs       # Breeding system (not yet wired)
    │   ├── rig.rs                # Body rig landmark system (4 rigs)
    │   ├── spawn.rs              # Entity spawning + component wiring
    │   └── species.rs            # Species templates and part definitions
    ├── genome/                   # Genetic blueprint
    │   ├── mod.rs                # Genome struct, random generation
    │   ├── species.rs            # Species enum (Moluun, Pylum, Skael, Nyxal)
    │   ├── crossover.rs          # Genetic crossover + mutation
    │   └── color.rs              # Genome → HSL body/tint color
    ├── mind/                     # AI and emotional state
    │   ├── mod.rs                # Mind struct, MoodState, VitalStats, FSM
    │   ├── neural.rs             # MLP neural network (12→8→7, 167 params)
    │   ├── plugin.rs             # Bevy integration, training schedule
    │   └── training.rs           # Training pipeline, event extraction
    ├── persistence/              # SQLite save/load
    │   ├── mod.rs
    │   ├── db.rs                 # Schema creation
    │   ├── load.rs               # Load genome/mind from DB
    │   ├── save.rs               # Save genome/mind/neural weights to DB
    │   └── plugin.rs             # Startup load, periodic save, exit save
    ├── ui/                       # User interface
    │   ├── mod.rs
    │   ├── actions.rs            # Feed/Play/Sleep + species buttons
    │   ├── creature_selector.rs  # Species selector events
    │   └── hud.rs                # Stat bar display
    ├── visuals/                  # Graphics and animation
    │   ├── mod.rs
    │   ├── accessories.rs        # Milestone rewards (ribbon, scarf, crown)
    │   ├── animation.rs          # Eye blink system
    │   ├── breathing.rs          # Breathing + heartbeat (scale oscillation)
    │   ├── effects.rs            # Floating mood effect sprites
    │   ├── evolution.rs          # Growth stages (baby → elder)
    │   ├── genome_visuals.rs     # Genome → visual appearance (tint, scale)
    │   ├── mood_sync.rs          # Mood → sprite swaps
    │   └── species_behavior.rs   # Per-species idle animations
    ├── dev/                      # Dev Mode (feature-gated: --features dev)
    │   ├── mod.rs                # DevPlugin, F12 toggle, DevModeState
    │   ├── rig_gizmos.rs         # Skeleton visualization with Bevy Gizmos
    │   └── panels.rs             # egui panels: stats, genome, neural, physics
    └── world/                    # Environment
        ├── mod.rs                # Camera setup
        ├── daycycle.rs           # Day/night cycle (system clock)
        └── time_tick.rs          # Game tick system (1 tick = 1 second)
```

## Data Flow

```
┌─────────────┐     ┌─────────────┐     ┌──────────────┐
│  SQLite DB  │────▶│ Persistence │────▶│ Genome, Mind │
│ (on disk)   │     │   Plugin    │     │ (Resources)  │
└─────────────┘     └─────────────┘     └──────┬───────┘
                                               │
          ┌────────────────┬───────────────────┼──────────────────┐
          │                │                   │                  │
          ▼                ▼                   ▼                  ▼
   ┌────────────┐  ┌──────────────┐   ┌──────────────┐    ┌────────────┐
   │  Physics   │  │  Mind Systems│   │   Visuals    │    │    UI      │
   │  - Gravity │  │  - FSM mood  │   │  - Spawn     │    │  - HUD     │
   │  - Bounce  │  │  - Neural net│   │  - Rig       │    │  - Buttons │
   │  - Buoyancy│  │  - Stat decay│   │  - Mood sync │    │  - Selector│
   └────────────┘  └──────────────┘   │  - Breathing │    └────────────┘
                                      │  - Behaviors │
                                      │  - Effects   │
                                      └──────────────┘
```

## Plugin Registration Order

```rust
let mut app = App::new();

app.add_plugins(DefaultPlugins)              // Bevy built-ins
    .add_plugins(PersistencePlugin)          // 1. Load saved state
    .add_systems(Startup, setup_world)       // 2. Camera
    .add_plugins(CreatureVisualsPlugin)      // 3. Spawn creature entities
    .add_plugins((DayCyclePlugin, TimeTickPlugin))  // 4. World systems
    .add_plugins(NeuralMindPlugin)           // 5. AI learning
    .add_plugins((StatsPlugin, ActionsPlugin, CreatureSelectorPlugin))  // 6. UI
    .add_plugins(MultiCreaturePlugin)        // 7. Collection management
    .add_plugins(PhysicsPlugin)              // 8. Gravity, collision, buoyancy
    .add_plugins((EffectsPlugin, AnimationPlugin, EvolutionPlugin, AccessoriesPlugin))
    .add_plugins((BreathingPlugin, SpeciesBehaviorPlugin))  // 9. Organic behavior
    .add_systems(Update, (sync_mood_sprites, apply_genome_visuals));

#[cfg(feature = "dev")]
app.add_plugins(dev::DevPlugin);             // 10. Dev Mode (F12 toggle)

app.run();
```

## The Four Species

| Species | Habitat | Personality | Visual Style | Unique Parts |
|---------|---------|------------|-------------|-------------|
| **Moluun** | The Verdance (forest) | Friendly, social, emotional mirror | Round, soft, cool colors | Ears (twitch), simple eyes, mouth |
| **Pylum** | Veridian Highlands | Curious, restless seeker | Winged, warm colors | Wings (flutter), beak, tail (sway) |
| **Skael** | Abyssal Shallows (cave) | Resilient, quiet protector | Scaled, elongated | Crests, slitted eyes, snout, tail (sway) |
| **Nyxal** | Abyssal Depths (deep sea) | Intelligent, adaptable | Tentacled, bioluminescent | 4 tentacles (undulate), mantle, glow eyes |

## Physics System

| Creature Type | Gravity | Ground | Bounce | Special |
|---------------|---------|--------|--------|---------|
| Land (Moluun, Pylum, Skael) | 400 px/s² | Y = -230 | 0.3 | Playful → jump, Sick → stumble |
| Aquatic (Nyxal) | 0 | — | — | Buoyancy spring (strength 120) |

## Organic Behavior Systems

| System | What It Does | Driven By |
|--------|-------------|-----------|
| **Breathing** | Rhythmic body scale oscillation (0.12–0.40 Hz) | Mood state |
| **Heartbeat** | Periodic scale pulse (50–80 BPM) | Health + mood, irregular when Sick |
| **Species Behavior** | Per-species idle animations (ear twitch, wing flutter, tentacle undulation) | Species + elapsed time |
| **Blink** | Periodic eye close (3-6s interval) | Timer-based |

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

## Gene → Behavior/Visual Mapping

| Gene | Behavior Effect | Visual Effect | Rig Effect |
|------|----------------|---------------|------------|
| `hue` | — | Body/part tint color (HSL) | — |
| `curiosity` | More likely to enter Playful mood | — | Wider eye spacing |
| `loneliness_sensitivity` | Lonely instead of Tired when unhappy | — | — |
| `appetite` | Faster hunger growth | Body width (rounder ↔ leaner) | Mouth/snout/tentacle position |
| `circadian` | Night owl vs early bird happiness bonus | — | — |
| `resilience` | Faster emotional recovery, less mood noise | — | Eye height, crest height |
| `learning_rate` | Neural network learns faster (lr = gene * 0.01 + 0.005) | — | — |

## Dev Mode (`cargo run --features dev`)

Toggle with **F12**. Shows:
- **Rig Gizmos**: anchor dots (color-coded), connection lines, bounding box, gene offset arrows
- **Stats Panel**: mood, hunger/happiness/energy/health bars, age, FSM vs NN comparison
- **Genome Panel**: all gene values as progress bars, hue color swatch
- **Neural Panel**: influence %, maturity, sessions, loss, live 7-mood prediction bars
- **Physics Panel**: velocity, grounded state, buoyancy, breathing rate, BPM

Compiles out completely when feature is off — zero impact on release builds.

## Growth Stages

| Stage | Age (ticks) | Scale | Description |
|-------|-------------|-------|-------------|
| Hatchling | 0–500 | 0.6 | Small, fragile, high curiosity |
| Juvenile | 500–2,000 | 0.8 | Rapid growth, personality solidifying |
| Adult | 2,000–10,000 | 1.0 | Full size, stable temperament |
| Elder | 10,000+ | 0.95 | Deeper resonance, slower metabolism |
