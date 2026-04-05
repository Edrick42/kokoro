# Table of Contents

> Note: This book is a living document. Chapters marked with 🔮 cover features
> that are planned but not yet implemented in the project. They will be written
> as the project evolves.

---

## Part I — Foundations: Rust from Zero

- [Chapter 1: Welcome to Rust](part-1/ch01-welcome.md)
  - Why Rust? Safety, speed, and joy
  - Installing Rust and Cargo
  - Your first program: Hello, Kokoro!
  - Cargo: the build system and package manager

- [Chapter 2: Variables, Types, and Ownership](part-1/ch02-ownership.md)
  - Let bindings and mutability
  - Primitive types: integers, floats, booleans, characters
  - Strings: `String` vs `&str`
  - **The Ownership Model** — Rust's defining feature
  - Move semantics and Copy types
  - *In Practice: Why Genome fields are `f32` and how ownership shapes our data*

- [Chapter 3: Structs and Building Data](part-1/ch03-structs.md)
  - Defining structs
  - Methods with `impl`
  - Tuple structs and unit structs
  - The `Default` trait
  - Derive macros: `Debug`, `Clone`, `Default`
  - *Build It: Creating the `VitalStats` struct*

- [Chapter 4: Enums and Pattern Matching](part-1/ch04-enums.md)
  - Enums as algebraic data types
  - `match` expressions — exhaustive and powerful
  - `Option<T>` and `Result<T, E>`
  - `if let` and `while let`
  - *Build It: The `MoodState` enum and `Species` enum*

- [Chapter 5: Functions, Closures, and Control Flow](part-1/ch05-functions.md)
  - Function signatures and return types
  - Closures: anonymous functions that capture their environment
  - Iterators and iterator adaptors (`map`, `filter`, `fold`)
  - Loops: `loop`, `while`, `for`
  - *In Practice: Stat decay functions and mood transitions*

- [Chapter 6: References and Borrowing](part-1/ch06-borrowing.md)
  - Immutable references (`&T`)
  - Mutable references (`&mut T`)
  - The borrowing rules: one `&mut` or many `&`
  - Lifetimes — what they are and when you need them
  - *Deep Dive: Why the crossover function needed standalone `fn` instead of closures*

- [Chapter 7: Error Handling](part-1/ch07-errors.md) 🔮
  - Panics vs recoverable errors
  - `Result<T, E>` in depth
  - The `?` operator
  - Custom error types
  - *In Practice: Database operations and `rusqlite` error handling*

- [Chapter 8: Collections and Generics](part-1/ch08-collections.md) 🔮
  - `Vec<T>`, `HashMap<K, V>`, `HashSet<T>`
  - Generics: writing code that works for many types
  - Trait bounds
  - *Build It: `CreatureCollection` and `PartSpriteHandles`*

- [Chapter 9: Traits — Rust's Superpower](part-1/ch09-traits.md) 🔮
  - Defining and implementing traits
  - Trait objects (`dyn Trait`) vs static dispatch
  - Common standard library traits: `Display`, `From`, `Into`
  - Operator overloading
  - *In Practice: Why `Genome` implements `Clone`, and how Bevy uses traits everywhere*

- [Chapter 10: Modules and Project Organization](part-1/ch10-modules.md) 🔮
  - `mod`, `pub`, and visibility
  - File-based module hierarchy
  - `use` and re-exports
  - Crates and the dependency ecosystem
  - *Build It: Restructuring from flat files to `creature/`, `visuals/`, `ui/`*

---

## Part II — Building the Game Core

- [Chapter 11: Introduction to Bevy and ECS](part-2/ch11-bevy-intro.md) 🔮
  - What is ECS? Entities, Components, Systems
  - Why ECS over OOP for games
  - Creating a Bevy `App`
  - `DefaultPlugins` and the game window
  - *Build It: The minimal Kokoro window*

- [Chapter 12: Components and Resources](part-2/ch12-components.md) 🔮
  - Components as plain data structs
  - Resources: global shared state
  - The `#[derive(Component)]` and `#[derive(Resource)]` macros
  - `Res<T>`, `ResMut<T>` — accessing resources in systems
  - *Build It: `Genome` and `Mind` as Resources*

- [Chapter 13: Systems — Where Logic Lives](part-2/ch13-systems.md) 🔮
  - System functions and their signatures
  - Queries: `Query<&T>`, `Query<(&A, &B)>`, filters
  - System ordering and schedules: `Startup`, `Update`
  - Change detection: `Changed<T>`, `Added<T>`
  - *Build It: The stat decay system and mood FSM*

- [Chapter 14: The Creature's Genome](part-2/ch14-genome.md) 🔮
  - Designing a genetic system
  - Gene ranges and species-specific defaults
  - Random generation with the `rand` crate
  - How genes influence behavior and appearance
  - *Build It: `Genome::random_for()` and species templates*

- [Chapter 15: The Creature's Mind — Finite State Machine](part-2/ch15-mind-fsm.md) 🔮
  - State machines in game design
  - Mood transitions based on vital stats
  - Critical states and priority systems
  - Realistic time cycles: gradual transitions, no instant mood changes
  - *Build It: The complete `Mind` struct with `update_mood()` and `tick()`*

- [Chapter 16: The World — Time and Environment](part-2/ch16-world.md) 🔮
  - Game time vs real time
  - The tick system
  - Day/night cycles
  - How environment affects creature behavior
  - *Build It: `TimeTickPlugin` and `DayCyclePlugin`*

---

## Part III — Intelligence and Evolution

- [Chapter 17: Serialization — Saving and Loading](part-3/ch17-persistence.md) 🔮
  - The `serde` ecosystem
  - `Serialize` and `Deserialize` derives
  - Binary encoding with `bincode`
  - SQLite with `rusqlite`: schemas, queries, and transactions
  - Database migrations: evolving schemas safely
  - *Build It: The complete persistence pipeline*

- [Chapter 18: Neural Networks in Pure Rust](part-3/ch18-neural.md) 🔮
  - What is a neural network? (conceptual)
  - The MLP architecture: input → hidden → output
  - Xavier initialization and why it matters
  - Forward pass: matrix multiplication, ReLU, softmax
  - *Build It: The `MLP` struct with `forward()` and `predict()`*

- [Chapter 19: Training the Mind](part-3/ch19-training.md) 🔮
  - Backpropagation and gradient descent
  - Cross-entropy loss for classification
  - Mini-batch training from event history
  - The FSM-Neural hybrid: veto system for critical states
  - Learning preferences: creature develops tastes and refuses actions
  - *Build It: `train_step()`, `train_on_samples()`, and the mood override system*

- [Chapter 20: Genetic Crossover and Mutation](part-3/ch20-genetics.md) 🔮
  - Genetic algorithms: crossover, mutation, selection
  - Per-gene crossover with random parent selection
  - Mutation rate and bounded gene values
  - Species-specific gene ranges
  - *Build It: `Genome::crossover()` with mutation*

- [Chapter 21: Multi-Creature Management](part-3/ch21-collection.md) 🔮
  - Managing multiple game entities with a collection
  - The active-creature pattern: swap resources, respawn visuals
  - Event-driven architecture: `SelectSpeciesEvent`
  - Multi-creature persistence: saving and loading all creatures
  - *Build It: `CreatureCollection` and the switch system*

- [Chapter 22: Events — Decoupled Communication](part-3/ch22-events.md) 🔮
  - Bevy's event system: `EventWriter`, `EventReader`
  - Why events beat direct coupling
  - Event ordering and frame boundaries
  - *In Practice: How a button press travels through 4 systems*

---

## Part IV — Visuals, UI, and Polish

- [Chapter 23: 2D Rendering in Bevy](part-4/ch23-rendering.md) 🔮
  - Sprites and sprite sheets
  - Procedural meshes: `Circle`, `Rectangle`
  - Materials and colors: HSL, sRGB
  - The transform hierarchy: parent/child entities
  - *Build It: Procedural mesh fallback creatures*

- [Chapter 24: The Body Rig System](part-4/ch24-rig.md) 🔮
  - Proportional landmark positioning
  - Normalized coordinates → pixel offsets
  - Gene-driven offsets (curiosity → eye spacing)
  - Species-specific rigs
  - *Build It: `BodyRig`, `moluun_rig()`, `pylum_rig()`, `skael_rig()`, `nyxal_rig()`*

- [Chapter 25: Modular Sprite Composition](part-4/ch25-sprites.md) 🔮
  - Entity hierarchy for body parts
  - Sprite loading and fallback strategy
  - Mood-reactive parts: swapping sprites on state change
  - Genome-driven visual modifiers: tint, scale, spacing
  - *Build It: `CreatureVisualsPlugin` and the spawn system*

- [Chapter 25b: The Sprite Pipeline — Generating Art with Rust](part-4/ch25b-sprite-pipeline.md)
  - Why generate art with code (and why in Rust, not Python)
  - The contract: what the game engine expects (names, sizes, transparency)
  - The toolkit: `image` crate, HashSet shapes, flood outline, gradient shading
  - Moluun, Pylum, Skael, Nyxal: species-specific pixel art generation
  - Creating a new species: the complete step-by-step
  - Design principles for pixel art sprites

- [Chapter 25c: Low-Poly Procedural Art](part-4/ch25c-lowpoly-pipeline.md) 🔮
  - Evolving from pixel ellipses to faceted polygon geometry
  - Triangle fill, facet shading, polygon outline algorithms
  - Alien body plans: bipedal, quadruped, serpentine, insectoid, cephalopod
  - Gene-driven vertex positions (each individual looks unique)
  - Compound eyes as faceted gems, chromatophore color shifting
  - Design principles for low-poly indie style

- [Chapter 26: User Interface with Bevy UI](part-4/ch26-ui.md) 🔮
  - Bevy's UI system: `Node`, `Button`, flexbox layout
  - Collapsible menus and modal panels
  - Building a HUD: stat bars, vitals panel (BPM, breathing)
  - Touch interaction: detecting where the player touches the creature
  - *Build It: Action menu, stat display, vitals panel*

- [Chapter 27: Animation and Organic Behavior](part-4/ch27-animation.md) 🔮
  - Eye blink system
  - Species-specific idle behaviors (ear twitch, wing flutter, tentacle undulation)
  - Breathing: mood-driven rhythmic scale oscillation
  - Heartbeat: BPM tracking tied to health, irregular when sick
  - Kokoro-sac resonance glow (lore-accurate frequencies)
  - Growth stages and visual evolution
  - *Build It: `AnimationPlugin`, `BreathingPlugin`, `SpeciesBehaviorPlugin`, `ResonanceGlowPlugin`*

- [Chapter 27b: Physics for Virtual Creatures](part-4/ch27b-physics.md) 🔮
  - Building a mini physics engine without external crates
  - Gravity, ground collision, and bounce
  - Buoyancy for aquatic creatures
  - Mood-triggered impulses (jump, stumble, slump)
  - *Build It: `PhysicsPlugin` and `PhysicsBody`*

- [Chapter 28: Plugins — Modular Architecture](part-4/ch28-plugins.md) 🔮
  - The Bevy Plugin trait
  - Designing self-contained plugins
  - Plugin ordering and dependencies
  - Feature-gated plugins: Dev Mode with `#[cfg(feature)]`
  - *Deep Dive: How Kokoro's 15+ plugins compose into one App*

- [Chapter 28b: Dev Mode — Building Debug Tools](part-4/ch28b-devmode.md) 🔮
  - Feature flags in Cargo.toml
  - Runtime toggles with `DevModeState`
  - Bevy Gizmos for rig visualization
  - egui panels for real-time data inspection
  - Time manipulation, state overrides, cheat panels for testing
  - *Build It: `DevPlugin` with rig gizmos, data panels, and dev cheats*

---

## Part V — Biological Simulation 🔮

> *The heart of what makes Kokoro unique: real biology, not fantasy.*

- [Chapter 29: Taxonomy — Classifying Life](part-5/ch29-taxonomy.md) 🔮
  - Biological classification: kingdom, phylum, class, order
  - Mapping Kobara species to real-world analogs (canine, amphibian, cephalopod, avian)
  - How taxonomy drives game mechanics
  - *Build It: `Taxonomy` enum and species classification system*

- [Chapter 30: The Egg Stage — Before Birth](part-5/ch30-egg-stage.md) 🔮
  - Species-specific pre-birth: cells, eggs, roe
  - Incubation mechanics: time, temperature, care
  - Genome expression during development
  - The moment of hatching
  - *Build It: `EggState` component and incubation system*

- [Chapter 31: Metabolism and Nutrition](part-5/ch31-metabolism.md) 🔮
  - Metabolic rate: energy expenditure, fat storage, body temperature
  - Nutrient types: protein, fiber, minerals, vitamins
  - Food items with nutritional profiles
  - Species dietary requirements (herbivore, carnivore, filter-feeder, omnivore)
  - Overeating, malnutrition, and their effects
  - *Build It: `Metabolism` component and `NutritionSystem`*

- [Chapter 32: Skeleton, Muscles, and Movement](part-5/ch32-body-systems.md) 🔮
  - Skeletal system: bones, joints, articulation
  - Muscular system: strength, fatigue, recovery
  - Nervous system: reflexes, coordination, learning
  - How biology drives stats: strength, agility, flexibility
  - Walking, swimming, flying animations from body structure
  - *Build It: `BodySystems` component tree and movement system*

- [Chapter 33: Natural Abilities](part-5/ch33-abilities.md) 🔮
  - Abilities grounded in real biology (not fantasy)
  - Electric sense (navigation, prey detection) — Nyxal
  - Echolocation (cave navigation) — Skael
  - Scent marking and pheromones — Moluun
  - Thermal vision, UV sight — Pylum
  - Venom, camouflage, regeneration — evolved traits with purpose
  - *Build It: `Ability` trait and species-specific implementations*

- [Chapter 34: Communication Between Creatures](part-5/ch34-communication.md) 🔮
  - Communication channels: sound, movement, expression, scent, color
  - Visual representation of invisible senses (scent clouds, sound waves)
  - Species-specific communication styles
  - Emotional broadcasting through the kokoro-sac
  - *Build It: `CommunicationSystem` with visual feedback*

- [Chapter 35: Hygiene and Self-Care](part-5/ch35-hygiene.md) 🔮
  - Species-specific cleanliness: grooming, bathing, molting, ink-cleaning
  - Hygiene stat and its effects on health and mood
  - Social grooming between creatures
  - *Build It: `HygieneSystem` and grooming behaviors*

- [Chapter 36: Learning and Preferences](part-5/ch36-learning.md) 🔮
  - Creatures develop individual tastes over time
  - Food preferences: likes, dislikes, allergies
  - Activity preferences: resist sleep, request play, refuse food
  - Memory of player interactions
  - Neural network personality emergence
  - *Build It: `PreferenceSystem` with persistent memory*

- [Chapter 37: Touch and Physical Interaction](part-5/ch37-touch.md) 🔮
  - Click/touch detection on creature body parts
  - Petting, scratching, tickling — different responses per species
  - Comfort zones and sensitive areas
  - Building trust through physical interaction
  - *Build It: `TouchInteraction` system with body-part hit detection*

---

## Part VI — Testing and Quality 🔮

- [Chapter 38: Unit Testing in Rust](part-6/ch38-unit-tests.md) 🔮
  - `#[test]` and `#[cfg(test)]`
  - Assertions: `assert!`, `assert_eq!`, `assert_ne!`
  - Testing private functions
  - Test organization: inline vs separate files
  - *Build It: Tests for Genome, Mind FSM, Physics, Metabolism*

- [Chapter 39: Integration Testing](part-6/ch39-integration-tests.md) 🔮
  - The `tests/` directory
  - Testing Bevy systems in isolation
  - Database testing with temporary DBs
  - Property-based testing for genetic algorithms
  - *Build It: Full test suite for persistence, neural network, creature lifecycle*

- [Chapter 40: Performance and Profiling](part-6/ch40-performance.md) 🔮
  - Profiling Rust code
  - Zero-cost abstractions in practice
  - Memory layout and cache friendliness
  - Bevy system ordering for performance

---

## Part VII — Full-Stack Rust Web 🔮

> *Build a companion website for Kokoro — entirely in Rust.*

- [Chapter 41: Web Backend with Axum](part-7/ch41-axum.md) 🔮
  - Async Rust: `async/await`, Tokio runtime
  - Axum fundamentals: routes, handlers, extractors
  - JSON APIs with `serde`
  - Middleware: CORS, logging, error handling
  - *Build It: `kokoro-api` — health check, creature endpoints*

- [Chapter 42: Authentication and User Profiles](part-7/ch42-auth.md) 🔮
  - User registration and login
  - Password hashing with `argon2`
  - JWT tokens or session-based auth
  - User profile storage
  - *Build It: Auth system with login, register, profile*

- [Chapter 43: Frontend with Leptos](part-7/ch43-leptos.md) 🔮
  - Rust in the browser: WebAssembly
  - Leptos signals and reactive UI
  - Components, props, and events
  - Server-side rendering (SSR)
  - *Build It: Kobara viewer, lore browser, profile page*

- [Chapter 44: Game ↔ Web Integration](part-7/ch44-integration.md) 🔮
  - Exporting creature data from the game
  - REST API for creature sync
  - Real-time updates with WebSockets
  - Community features: sharing, leaderboards
  - *Build It: Sync pipeline from Bevy game to web dashboard*

- [Chapter 45: Deployment](part-7/ch45-deployment.md) 🔮
  - Building for desktop (Windows, macOS, Linux)
  - Mobile targets (Android, iOS) with cargo-mobile2
  - Web deployment: Docker, fly.io, Railway
  - CI/CD with GitHub Actions

---

## Part VIII — Concurrency and Advanced Rust 🔮

- [Chapter 46: Concurrency in Rust](part-8/ch46-concurrency.md) 🔮
  - Threads and `Send`/`Sync`
  - `Mutex` and `Arc` — thread-safe shared state
  - Channels: `mpsc`, `crossbeam`
  - *In Practice: `DbConnection` wrapped in `Mutex` for thread safety*

- [Chapter 47: The Rust Ecosystem](part-8/ch47-ecosystem.md) 🔮
  - Crates.io and documentation
  - Useful crates for game development
  - The Rust community
  - Where to go from here

---

## Appendices

- [Appendix A: Rust Cheat Sheet](appendices/appendix-a-cheatsheet.md)
- [Appendix B: Bevy Cheat Sheet](appendices/appendix-b-bevy.md)
- [Appendix C: Complete Kokoro Architecture](appendices/appendix-c-architecture.md)
- [Appendix D: Glossary](appendices/appendix-d-glossary.md)
- [Appendix E: Using NotebookLM to Study This Book](appendices/appendix-e-notebooklm.md) 🔮
