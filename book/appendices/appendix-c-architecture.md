# Appendix C: Complete Kokoro Architecture

## Project Structure

```
kokoro/
├── Cargo.toml                    # Dependencies and project metadata
├── book/                         # This ebook
├── assets/
│   └── sprites/
│       ├── kobara/               # Moluun species sprites
│       ├── pylum/              # Pylum species sprites
│       └── skael/                # Skael species sprites
├── scripts/                      # Sprite generation scripts (Python)
└── src/
    ├── main.rs                   # App entry point, plugin registration
    ├── creature/                 # Creature lifecycle and genetics
    │   ├── mod.rs
    │   ├── collection.rs         # Multi-creature management
    │   ├── reproduction.rs       # Breeding system (future)
    │   ├── rig.rs                # Body rig landmark system
    │   ├── spawn.rs              # Visual entity spawning + respawn
    │   └── species.rs            # Species templates and part definitions
    ├── genome/                   # Genetic blueprint
    │   └── mod.rs                # Genome struct, Species enum, crossover
    ├── mind/                     # AI and emotional state
    │   ├── mod.rs                # Mind struct, MoodState, VitalStats
    │   ├── neural.rs             # MLP neural network (12→8→7)
    │   ├── plugin.rs             # Bevy integration, training schedule
    │   └── training.rs           # Training pipeline, event extraction
    ├── persistence/              # SQLite save/load
    │   ├── mod.rs
    │   ├── db.rs                 # Schema creation
    │   ├── load.rs               # Load genome/mind from DB
    │   ├── save.rs               # Save genome/mind to DB
    │   └── plugin.rs             # Startup load, periodic save
    ├── ui/                       # User interface
    │   ├── mod.rs
    │   ├── actions.rs            # Feed/Play/Sleep + species buttons
    │   ├── creature_selector.rs  # Top-right species selector
    │   └── hud.rs                # Stat bar display
    ├── visuals/                  # Graphics and animation
    │   ├── mod.rs
    │   ├── accessories.rs        # Milestone rewards (ribbon, scarf, crown)
    │   ├── animation.rs          # Idle bobbing, reactions
    │   ├── effects.rs            # Particle-like visual feedback
    │   ├── evolution.rs          # Growth stages (baby → elder)
    │   ├── genome_visuals.rs     # Genome → visual appearance
    │   └── mood_sync.rs          # Mood → sprite swaps
    └── world/                    # Environment
        ├── mod.rs                # Camera setup
        ├── daycycle.rs           # Day/night cycle
        └── time_tick.rs          # Game tick system
```

## Data Flow

```
┌─────────────┐     ┌─────────────┐     ┌──────────────┐
│  SQLite DB  │────▶│ Persistence │────▶│ Genome, Mind │
│ (on disk)   │     │   Plugin    │     │ (Resources)  │
└─────────────┘     └─────────────┘     └──────┬───────┘
                                               │
                    ┌──────────────────────────┼──────────────────┐
                    │                          │                  │
                    ▼                          ▼                  ▼
            ┌──────────────┐          ┌──────────────┐    ┌────────────┐
            │  Mind Systems │          │   Visuals    │    │    UI      │
            │  - FSM mood   │          │  - Spawn     │    │  - HUD     │
            │  - Neural net │          │  - Rig       │    │  - Buttons │
            │  - Stat decay │          │  - Mood sync │    │  - Selector│
            └──────────────┘          └──────────────┘    └────────────┘
```

## Plugin Registration Order

```rust
App::new()
    .add_plugins(DefaultPlugins)           // Bevy built-ins
    .add_plugins(PersistencePlugin)        // 1. Load saved state
    .add_systems(Startup, setup_world)     // 2. Camera
    .add_plugins(CreatureVisualsPlugin)    // 3. Spawn creature entities
    .add_plugins((DayCyclePlugin, TimeTickPlugin))  // 4. World systems
    .add_plugins(NeuralMindPlugin)         // 5. AI learning
    .add_plugins((StatsPlugin, ActionsPlugin, CreatureSelectorPlugin))  // 6. UI
    .add_plugins(MultiCreaturePlugin)      // 7. Collection management
    .add_plugins((EffectsPlugin, AnimationPlugin, EvolutionPlugin, AccessoriesPlugin))
    .add_systems(Update, (sync_mood_sprites, apply_genome_visuals))
    .run();
```

## The Three Species

| Species | Personality | Visual Style | Unique Parts |
|---------|------------|-------------|-------------|
| **Moluun** | Friendly, social | Round, soft, cool colors | Ears, simple eyes, small mouth |
| **Pylum** | Curious, adventurous | Winged, warm colors | Wings, beak, tail feathers |
| **Skael** | Resilient, calm | Scaled, elongated, green | Crests/horns, slitted eyes, snout, thick tail |

## Neural Network Architecture

```
Input (12)              Hidden (8)           Output (7)
─────────              ──────────           ──────────
hunger         ─┐
happiness       │     ┌─ neuron 0 ─┐
energy          ├────▶│  neuron 1  ├──────▶ Happy
health          │     │  neuron 2  │        Hungry
curiosity       │     │  neuron 3  │        Tired
loneliness_s    ├────▶│  neuron 4  ├──────▶ Lonely
appetite        │     │  neuron 5  │        Playful
circadian       │     │  neuron 6  │        Sick
resilience      ├────▶│  neuron 7  │        Sleeping
hue (norm.)     │     └────────────┘
species (norm.) │       ReLU activation     Softmax
time_of_day    ─┘                           (probabilities)

Total parameters: 12×8 + 8 + 8×7 + 7 = 167
```

## Gene → Behavior/Visual Mapping

| Gene | Behavior Effect | Visual Effect |
|------|----------------|---------------|
| `hue` | — | Body/ear tint color (HSL) |
| `curiosity` | More likely to enter Playful mood | Wider eye spacing |
| `loneliness_sensitivity` | Faster happiness decay | — |
| `appetite` | Faster hunger growth | Body width (rounder ↔ leaner) |
| `circadian` | Sleep timing preference | — |
| `resilience` | Faster health recovery | — |
