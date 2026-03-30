# Chapter 3: Structs and Building Data

In the previous chapter, we defined a simple `VitalStats` struct. Now let's go deeper. Structs are Rust's primary way to create custom data types — the building blocks of every system in Kokoro.

## Defining Structs

A struct groups related data together under a name:

```rust
pub struct Genome {
    pub species: Species,
    pub hue: f32,
    pub curiosity: f32,
    pub loneliness_sensitivity: f32,
    pub appetite: f32,
    pub circadian: f32,
    pub resilience: f32,
}
```

This is directly from Kokoro's `src/genome/mod.rs`. Each field has a name and a type. The `pub` keyword makes the field accessible from outside the module.

### Creating Instances

```rust
let genome = Genome {
    species: Species::Moluun,
    hue: 200.0,
    curiosity: 0.65,
    loneliness_sensitivity: 0.50,
    appetite: 0.40,
    circadian: 0.70,
    resilience: 0.55,
};
```

Every field must be specified — Rust doesn't have null, so there's no concept of "leaving a field empty."

### The Spread Operator: `..default()`

When a struct implements the `Default` trait, you can fill in only the fields you care about:

```rust
let window = Window {
    title: "Kokoro".into(),
    resolution: (400.0, 700.0).into(),
    resizable: true,
    ..default()   // Fill remaining fields with defaults
};
```

The `..default()` syntax says: "for every field I didn't mention, use the value from `Default::default()`." This is incredibly common in Bevy, where structs often have 10+ fields but you only need to customize 2-3.

## Methods with `impl`

Methods are functions associated with a struct. They're defined in an `impl` block:

```rust
impl Genome {
    /// Creates a random genome for a given species.
    pub fn random_for(species: Species) -> Self {
        let mut rng = rand::rng();
        match species {
            Species::Moluun => Genome {
                species,
                hue: rng.random_range(180.0..260.0),
                curiosity: rng.random_range(0.3..0.7),
                loneliness_sensitivity: rng.random_range(0.4..0.8),
                appetite: rng.random_range(0.3..0.6),
                circadian: rng.random_range(0.4..0.7),
                resilience: rng.random_range(0.3..0.6),
            },
            // ... other species
        }
    }
}
```

### `self`, `&self`, and `&mut self`

Methods can access the struct instance through a parameter called `self`:

```rust
impl Mind {
    /// Checks if the current mood is critical (cannot be overridden).
    pub fn is_critical(&self) -> bool {
        matches!(self.mood, MoodState::Sick | MoodState::Sleeping)
    }

    /// Feeds the creature — reduces hunger, slight happiness boost.
    pub fn feed(&mut self) {
        self.stats.hunger = (self.stats.hunger + 25.0).min(100.0);
        self.stats.happiness = (self.stats.happiness + 5.0).min(100.0);
    }
}
```

The three forms of `self`:

| Signature | Meaning | When to use |
|---|---|---|
| `&self` | Immutable borrow — can read, not modify | Most methods: getters, calculations |
| `&mut self` | Mutable borrow — can read and modify | Methods that change state |
| `self` | Takes ownership — consumes the struct | Rare: transformations, builders |

This ties directly back to ownership. When you call `mind.feed()`, the method borrows `mind` mutably for the duration of the call. Nothing else can access `mind` while `feed()` is running. This prevents data races — even in single-threaded code, it prevents subtle bugs where two parts of code modify the same data simultaneously.

### Associated Functions (No `self`)

Functions that don't take `self` are like static methods in other languages:

```rust
impl Mind {
    /// Creates a new Mind with default stats and Happy mood.
    pub fn new() -> Self {
        Mind {
            stats: VitalStats::new(),
            mood: MoodState::Happy,
            age_ticks: 0,
        }
    }
}
```

Called as `Mind::new()` (with `::`, not `.`).

## Tuple Structs and Newtype Pattern

Sometimes you want a struct with unnamed fields:

```rust
/// Identifies which body part an entity represents.
#[derive(Component, Clone)]
pub struct BodyPartSlot(pub String);
```

`BodyPartSlot` wraps a `String` but gives it a distinct type. This is the **newtype pattern** — it prevents accidentally passing a random `String` where a `BodyPartSlot` is expected.

```rust
let slot = BodyPartSlot("eye_left".to_string());
println!("{}", slot.0);  // Access the inner value with .0
```

From Kokoro's `src/creature/species.rs`:

```rust
#[derive(Component)]
struct SelectorButton(usize);        // Wraps an index

#[derive(Component)]
struct NewSpeciesButton(Species);    // Wraps a Species enum
```

Each of these is just a wrapper, but the type system now distinguishes them. You can't accidentally pass a `SelectorButton` where a `NewSpeciesButton` is expected.

## Derive Macros: Automatic Trait Implementations

Rust can automatically implement common traits for your structs using `#[derive(...)]`:

```rust
#[derive(Debug, Clone, Default, Resource)]
pub struct VitalStats {
    pub hunger: f32,
    pub happiness: f32,
    pub energy: f32,
    pub health: f32,
}
```

| Derive | What it does |
|---|---|
| `Debug` | Enables `{:?}` formatting for printing |
| `Clone` | Adds `.clone()` method for deep copying |
| `Copy` | Enables implicit copying (only for simple types) |
| `Default` | Adds `Default::default()` with zero/empty values |
| `PartialEq` | Enables `==` comparison |
| `Component` | Bevy: marks as an ECS component |
| `Resource` | Bevy: marks as a global resource |
| `Event` | Bevy: marks as an event type |

Let's see how Kokoro uses these. The `StoredCreature` struct:

```rust
#[derive(Debug, Clone)]
pub struct StoredCreature {
    pub name: String,
    pub genome: Genome,
    pub mind: Mind,
}
```

- `Debug` — so we can print creatures for debugging: `println!("{:?}", creature)`
- `Clone` — so we can duplicate creature data when saving/loading

But NOT `Copy` — because `String`, `Genome`, and `Mind` contain heap data. Deriving `Copy` would fail at compile time. Rust protects you from mistakes even in derive macros.

## Build It: The Genome Struct

Let's build the `Genome` struct step by step, as it exists in `src/genome/mod.rs`.

First, define the species:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Species {
    Moluun,
    Pylum,
    Skael,
}
```

Now the genome itself:

```rust
#[derive(Debug, Clone, Resource)]
pub struct Genome {
    pub species: Species,
    pub hue: f32,                       // 0-360, body color
    pub curiosity: f32,                 // 0-1, eye spacing
    pub loneliness_sensitivity: f32,    // 0-1, social need
    pub appetite: f32,                  // 0-1, hunger rate
    pub circadian: f32,                 // 0-1, sleep preference
    pub resilience: f32,                // 0-1, health recovery
}
```

Notice the design:
- All genes are `f32` in the range 0.0 to 1.0 (except `hue`, which is 0-360)
- Each gene has a **dual purpose**: it affects both behavior AND appearance
- The struct derives `Resource` so Bevy can store it as global state

This is a pattern you'll see throughout game development: **data that serves multiple systems**. The `curiosity` gene makes the creature more likely to enter a Playful mood (behavior) AND widens the eye spacing (appearance). One gene, two effects, zero coupling between the systems that read it.

## Deep Dive: Stack vs Heap

Understanding where data lives helps you understand ownership:

**Stack**: Fast, fixed-size. Function locals, primitives, small structs.
```rust
let hunger: f32 = 50.0;        // 4 bytes on the stack
let active: bool = true;        // 1 byte on the stack
```

**Heap**: Flexible, dynamically-sized. `String`, `Vec`, `Box`.
```rust
let name = String::from("Moluun");  // Pointer on stack → data on heap
```

A `String` is actually three values on the stack: a pointer to heap memory, a length, and a capacity. When the `String` is dropped, the heap memory is freed.

When you **move** a `String`, you're copying those three stack values (cheap!) and invalidating the original (so only one variable can free the heap memory). When you **clone** a `String`, you're copying the three stack values AND duplicating the heap data (expensive!).

This is why Rust distinguishes move from clone. It's not pedantic — it's giving you control over performance.

## Checkpoint

After this chapter, you should understand:

- Structs group named fields into custom types
- `impl` blocks add methods: `&self` (read), `&mut self` (modify), `self` (consume)
- Associated functions (no `self`) are called with `::` syntax
- Tuple structs wrap existing types with new names (newtype pattern)
- `#[derive(...)]` automatically implements common traits
- Stack data is cheap to copy; heap data requires explicit `.clone()`
- The `..default()` spread syntax fills remaining fields with defaults

In the next chapter, we'll explore enums — Rust's incredibly powerful algebraic data types. We'll build the `MoodState` enum and discover how `match` makes impossible states unrepresentable.
