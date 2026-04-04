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

- [Chapter 7: Error Handling](part-1/ch07-errors.md)
  - Panics vs recoverable errors
  - `Result<T, E>` in depth
  - The `?` operator
  - Custom error types
  - *In Practice: Database operations and `rusqlite` error handling*

- [Chapter 8: Collections and Generics](part-1/ch08-collections.md)
  - `Vec<T>`, `HashMap<K, V>`, `HashSet<T>`
  - Generics: writing code that works for many types
  - Trait bounds
  - *Build It: `CreatureCollection` and `PartSpriteHandles`*

- [Chapter 9: Traits — Rust's Superpower](part-1/ch09-traits.md)
  - Defining and implementing traits
  - Trait objects (`dyn Trait`) vs static dispatch
  - Common standard library traits: `Display`, `From`, `Into`
  - Operator overloading
  - *In Practice: Why `Genome` implements `Clone`, and how Bevy uses traits everywhere*

- [Chapter 10: Modules and Project Organization](part-1/ch10-modules.md)
  - `mod`, `pub`, and visibility
  - File-based module hierarchy
  - `use` and re-exports
  - Crates and the dependency ecosystem
  - *Build It: Restructuring from flat files to `creature/`, `visuals/`, `ui/`*

---

## Part II — Building the Game Core

- [Chapter 11: Introduction to Bevy and ECS](part-2/ch11-bevy-intro.md)
  - What is ECS? Entities, Components, Systems
  - Why ECS over OOP for games
  - Creating a Bevy `App`
  - `DefaultPlugins` and the game window
  - *Build It: The minimal Kokoro window*

- [Chapter 12: Components and Resources](part-2/ch12-components.md)
  - Components as plain data structs
  - Resources: global shared state
  - The `#[derive(Component)]` and `#[derive(Resource)]` macros
  - `Res<T>`, `ResMut<T>` — accessing resources in systems
  - *Build It: `Genome` and `Mind` as Resources*

- [Chapter 13: Systems — Where Logic Lives](part-2/ch13-systems.md)
  - System functions and their signatures
  - Queries: `Query<&T>`, `Query<(&A, &B)>`, filters
  - System ordering and schedules: `Startup`, `Update`
  - Change detection: `Changed<T>`, `Added<T>`
  - *Build It: The stat decay system and mood FSM*

- [Chapter 14: The Creature's Genome](part-2/ch14-genome.md)
  - Designing a genetic system
  - Gene ranges and species-specific defaults
  - Random generation with the `rand` crate
  - How genes influence behavior and appearance
  - *Build It: `Genome::random_for()` and species templates*

- [Chapter 15: The Creature's Mind — Finite State Machine](part-2/ch15-mind-fsm.md)
  - State machines in game design
  - Mood transitions based on vital stats
  - Critical states and priority systems
  - Time-based stat decay with genome modulation
  - *Build It: The complete `Mind` struct with `update_mood()` and `tick()`*

- [Chapter 16: The World — Time and Environment](part-2/ch16-world.md)
  - Game time vs real time
  - The tick system
  - Day/night cycles
  - How environment affects creature behavior
  - *Build It: `TimeTickPlugin` and `DayCyclePlugin`*

---

## Part III — Intelligence and Evolution

- [Chapter 17: Serialization — Saving and Loading](part-3/ch17-persistence.md)
  - The `serde` ecosystem
  - `Serialize` and `Deserialize` derives
  - Binary encoding with `bincode`
  - SQLite with `rusqlite`: schemas, queries, and transactions
  - *Build It: The complete persistence pipeline*

- [Chapter 18: Neural Networks in Pure Rust](part-3/ch18-neural.md)
  - What is a neural network? (conceptual)
  - The MLP architecture: input → hidden → output
  - Xavier initialization and why it matters
  - Forward pass: matrix multiplication, ReLU, softmax
  - *Build It: The `MLP` struct with `forward()` and `predict()`*

- [Chapter 19: Training the Mind](part-3/ch19-training.md)
  - Backpropagation and gradient descent
  - Cross-entropy loss for classification
  - Mini-batch training from event history
  - The FSM-Neural hybrid: veto system for critical states
  - *Build It: `train_step()`, `train_on_samples()`, and the mood override system*

- [Chapter 20: Genetic Crossover and Mutation](part-3/ch20-genetics.md)
  - Genetic algorithms: crossover, mutation, selection
  - Per-gene crossover with random parent selection
  - Mutation rate and bounded gene values
  - Species-specific gene ranges
  - *Build It: `Genome::crossover()` with mutation*

- [Chapter 21: Multi-Creature Management](part-3/ch21-collection.md)
  - Managing multiple game entities with a collection
  - The active-creature pattern: swap resources, respawn visuals
  - Event-driven architecture: `SelectSpeciesEvent`
  - *Build It: `CreatureCollection` and the switch system*

- [Chapter 22: Events — Decoupled Communication](part-3/ch22-events.md)
  - Bevy's event system: `EventWriter`, `EventReader`
  - Why events beat direct coupling
  - Event ordering and frame boundaries
  - *In Practice: How a button press travels through 4 systems*

---

## Part IV — Visuals, UI, and Polish

- [Chapter 23: 2D Rendering in Bevy](part-4/ch23-rendering.md)
  - Sprites and sprite sheets
  - Procedural meshes: `Circle`, `Rectangle`
  - Materials and colors: HSL, sRGB
  - The transform hierarchy: parent/child entities
  - *Build It: Procedural mesh fallback creatures*

- [Chapter 24: The Body Rig System](part-4/ch24-rig.md)
  - Proportional landmark positioning
  - Normalized coordinates → pixel offsets
  - Gene-driven offsets (curiosity → eye spacing)
  - Species-specific rigs
  - *Build It: `BodyRig`, `moluun_rig()`, `pylum_rig()`, `skael_rig()`*

- [Chapter 25: Modular Sprite Composition](part-4/ch25-sprites.md)
  - Entity hierarchy for body parts
  - Sprite loading and fallback strategy
  - Mood-reactive parts: swapping sprites on state change
  - Genome-driven visual modifiers: tint, scale, spacing
  - *Build It: `CreatureVisualsPlugin` and the spawn system*

- [Chapter 25b: The Sprite Pipeline — Generating Art with Rust](part-4/ch25b-sprite-pipeline.md)
  - Why generate art with code (and why in Rust, not Python)
  - The contract: what the game engine expects (names, sizes, transparency)
  - The toolkit: `image` crate, HashSet shapes, flood outline, gradient shading
  - Moluun: body shading, parameterized eyes with Default trait, 8 mouth variants
  - Pylum: wings with directional gradient, triangle beak, tail feathers
  - Skael: scale texture with step_by, slit pupils, horn crests, fanged snout
  - Nyxal: deep-sea palette, bioluminescent eyes, tapered tentacles
  - Creating a new species: the complete step-by-step
  - Design principles for pixel art sprites

- [Chapter 26: User Interface with Bevy UI](part-4/ch26-ui.md)
  - Bevy's UI system: `Node`, `Button`, flexbox layout
  - Interaction handling: `Interaction` component
  - Building a HUD: stat bars, icons
  - Dynamic UI: the creature selector
  - *Build It: Action buttons, stat display, species selector*

- [Chapter 27: Animation and Organic Behavior](part-4/ch27-animation.md)
  - Eye blink system
  - Species-specific idle behaviors (ear twitch, wing flutter, tentacle undulation)
  - Breathing: mood-driven rhythmic scale oscillation
  - Heartbeat: periodic pulse tied to health, irregular when sick
  - Growth stages and visual evolution
  - Accessories: milestone rewards
  - *Build It: `AnimationPlugin`, `BreathingPlugin`, `SpeciesBehaviorPlugin`*

- [Chapter 27b: Physics for Virtual Creatures](part-4/ch27b-physics.md)
  - Building a mini physics engine without external crates
  - Gravity, ground collision, and bounce
  - Buoyancy for aquatic creatures
  - Mood-triggered impulses (jump, stumble, slump)
  - Transform ownership: preventing system conflicts
  - *Build It: `PhysicsPlugin` and `PhysicsBody`*

- [Chapter 28: Plugins — Modular Architecture](part-4/ch28-plugins.md)
  - The Bevy Plugin trait
  - Designing self-contained plugins
  - Plugin ordering and dependencies
  - When to split vs when to combine
  - Feature-gated plugins: Dev Mode with `#[cfg(feature)]`
  - *Deep Dive: How Kokoro's 15+ plugins compose into one App*

- [Chapter 28b: Dev Mode — Building Debug Tools](part-4/ch28b-devmode.md)
  - Feature flags in Cargo.toml
  - Runtime toggles with `DevModeState`
  - Bevy Gizmos for rig visualization
  - egui panels for real-time data inspection
  - Conditional compilation: zero cost in release builds
  - *Build It: `DevPlugin` with rig gizmos and data panels*

---

## Part V — Advanced Rust and Beyond 🔮

> *These chapters will be written as the project grows.*

- [Chapter 29: Concurrency and Async](part-5/ch29-concurrency.md)
  - Threads and `Send`/`Sync`
  - `Mutex` and `Arc` — thread-safe shared state
  - Async/await basics
  - *In Practice: `DbConnection` wrapped in `Mutex` for thread safety*

- [Chapter 30: Performance and Optimization](part-5/ch30-performance.md)
  - Profiling Rust code
  - Zero-cost abstractions in practice
  - Memory layout and cache friendliness
  - When to use `Box`, `Rc`, `Arc`

- [Chapter 31: Testing](part-5/ch31-testing.md)
  - Unit tests with `#[test]`
  - Integration tests
  - Testing game systems
  - Property-based testing for genetic algorithms

- [Chapter 32: Cross-Platform Distribution](part-5/ch32-distribution.md)
  - Building for desktop (Windows, macOS, Linux)
  - Mobile targets (Android, iOS)
  - Web builds with WebAssembly
  - Packaging and release

- [Chapter 33: Full-Stack Rust — Building the Kokoro Website](part-5/ch33-website.md)
  - Why full-stack Rust (backend + frontend)
  - Backend: Axum or Actix-web for REST API
  - Frontend: Leptos, Dioxus, or Yew for reactive UI
  - Database: connecting SQLite creature data to the web
  - User profiles, Kobara info pages, lore browser
  - Community features: sharing creatures, leaderboards
  - *Build It: A companion website where your Kobara lives online*

- [Chapter 34: The Rust Ecosystem](part-5/ch34-ecosystem.md)
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
