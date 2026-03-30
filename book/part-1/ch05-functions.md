# Chapter 5: Functions, Closures, and Control Flow

Functions are the workhorses of any program. In this chapter, we'll cover how Rust functions work, introduce closures (anonymous functions), and explore the iterator pattern that makes Rust code remarkably expressive.

## Function Signatures

Every function in Rust declares its parameter types and return type explicitly:

```rust
/// Clamps a value between 0.0 and a maximum.
fn clamp_stat(value: f32, max: f32) -> f32 {
    value.max(0.0).min(max)
}
```

- Parameters: `value: f32, max: f32` — name and type for each
- Return type: `-> f32` after the parameter list
- The last expression without `;` is the return value (no `return` keyword needed)

If a function returns nothing, the return type is omitted (or written as `-> ()`):

```rust
fn feed(&mut self) {
    self.stats.hunger = (self.stats.hunger + 25.0).min(100.0);
    self.stats.happiness = (self.stats.happiness + 5.0).min(100.0);
}
```

### Expression vs Statement

This is a subtle but important Rust distinction:

```rust
// Expression — evaluates to a value (no semicolon)
let result = if health > 50.0 { "healthy" } else { "weak" };

// Statement — performs an action (has semicolon)
let x = 5;     // Binding statement
println!("x"); // Expression statement (value discarded)
```

In Rust, almost everything is an expression. `if/else`, `match`, and even blocks `{}` evaluate to a value:

```rust
let mood_label = match mood {
    MoodState::Happy => "happy",
    MoodState::Hungry => "hungry",
    _ => "neutral",
};
```

This eliminates a whole class of "forgot to assign in one branch" bugs.

## Closures: Anonymous Functions

A closure is an anonymous function that can capture variables from its surrounding scope:

```rust
// Regular function
fn add_one(x: i32) -> i32 {
    x + 1
}

// Closure — same thing, shorter
let add_one = |x: i32| -> i32 { x + 1 };

// Closure — type inference, single expression
let add_one = |x| x + 1;
```

Closures shine when combined with iterators. Here's a real example from Kokoro's `collection.rs`:

```rust
// Find the creature of a given species
let target_index = collection.creatures.iter()
    .position(|c| c.genome.species == event.species);
```

The `|c|` is a closure parameter. It receives each creature and returns `true` when the species matches. The `.position()` method uses this closure to find the index.

### Closure Capture

Closures can capture variables from their environment:

```rust
let species = Species::Marumi;

// This closure captures `species` by reference
let is_marumi = |creature: &StoredCreature| {
    creature.genome.species == species   // Uses `species` from outer scope
};

let count = collection.creatures.iter()
    .filter(|c| is_marumi(c))
    .count();
```

Rust closures capture by the least expensive method needed:
- **By reference** (`&T`) — if the closure only reads the value
- **By mutable reference** (`&mut T`) — if the closure modifies the value
- **By value** (move) — if the closure needs to own the value

You can force move semantics with the `move` keyword:

```rust
let name = String::from("Marumi");
let greeting = move || {
    println!("Hello, {name}!");  // `name` is moved INTO the closure
};
// `name` is no longer valid here
```

## Iterators

Iterators are Rust's primary way to process sequences. They're lazy (nothing happens until consumed) and zero-cost (the compiler optimizes them to plain loops).

### The Iterator Chain Pattern

Kokoro uses iterator chains throughout:

```rust
// Count how many of a species exist in the collection
let species_count = collection.creatures.iter()
    .filter(|c| c.genome.species == event.species)
    .count();
```

Let's break down what happens:

1. `.iter()` — creates an iterator over `&StoredCreature` references
2. `.filter(|c| ...)` — keeps only elements where the closure returns `true`
3. `.count()` — consumes the iterator and counts the remaining items

No intermediate allocations. No temporary vectors. The compiler fuses these into a single optimized loop.

### Common Iterator Methods

| Method | What it does | Example |
|---|---|---|
| `.map(f)` | Transform each element | `.map(|c| c.name.clone())` |
| `.filter(f)` | Keep elements where `f` returns true | `.filter(|c| c.health > 0.0)` |
| `.find(f)` | First element matching `f` | `.find(|c| c.species == Marumi)` |
| `.position(f)` | Index of first element matching `f` | `.position(|c| c.age > 100)` |
| `.enumerate()` | Adds index to each element | `.enumerate().map(|(i, c)| ...)` |
| `.count()` | Number of elements | `.filter(...).count()` |
| `.collect()` | Gather into a collection | `.map(...).collect::<Vec<_>>()` |
| `.for_each(f)` | Side effect on each element | `.for_each(|c| println!("{}", c))` |
| `.any(f)` | True if any element matches | `.any(|c| c.mood == Sick)` |
| `.all(f)` | True if all elements match | `.all(|c| c.health > 50.0)` |
| `.fold(init, f)` | Accumulate a result | `.fold(0.0, |sum, c| sum + c.health)` |

### Real Example: Building a HashMap from an Iterator

From `spawn.rs` — building a lookup table from resolved rig positions:

```rust
let anchor_map: HashMap<String, &ResolvedAnchor> = resolved
    .iter()
    .map(|a| (a.slot.clone(), a))
    .collect();
```

`.collect()` is magical — it can build different collection types based on the type annotation. Here, `HashMap<String, &ResolvedAnchor>` tells `.collect()` to build a hash map from the `(key, value)` tuples.

## Control Flow

### `for` Loops

The most common loop, iterates over any iterator:

```rust
// Iterate over children of a creature entity
for child in children.iter() {
    let Ok((slot, mut transform, sprite)) = part_q.get_mut(child) else {
        continue;   // Skip if this child doesn't have the right components
    };

    if slot.0 == "body" {
        transform.scale.x = body_scale_x;
    }
}
```

### `loop` — Infinite Loop

```rust
loop {
    // Game engines run in an infinite loop
    // (Bevy handles this for us with .run())
    process_input();
    update_game_state();
    render_frame();
}
```

### `while` — Conditional Loop

```rust
let mut attempts = 0;
while attempts < 5 {
    if try_connect() {
        break;
    }
    attempts += 1;
}
```

### `let else` — Early Return Pattern

A Rust pattern you'll see in Bevy systems:

```rust
fn update_creature(
    root_q: Query<&Children, With<CreatureRoot>>,
) {
    let Ok(children) = root_q.single() else { return };
    // Only reached if exactly one CreatureRoot exists
    // `children` is now available
}
```

`let else` tries to match a pattern. If it fails, the `else` block must diverge (`return`, `break`, `continue`, or `panic!`).

## In Practice: Stat Decay Functions

Kokoro's mind system uses functions and closures for stat management. Here's how the tick system works:

```rust
pub fn tick(&mut self, genome: &Genome) {
    self.age_ticks += 1;

    // Hunger increases based on appetite gene
    let hunger_rate = 0.10 + genome.appetite * 0.10;
    self.stats.hunger = (self.stats.hunger - hunger_rate).max(0.0);

    // Happiness decays, modified by loneliness sensitivity
    let social_decay = 0.05 + genome.loneliness_sensitivity * 0.05;
    self.stats.happiness = (self.stats.happiness - social_decay).max(0.0);

    // Energy decays, modified by circadian gene
    let energy_decay = 0.08 + (1.0 - genome.circadian) * 0.04;
    self.stats.energy = (self.stats.energy - energy_decay).max(0.0);

    // Health regenerates based on resilience
    let regen = genome.resilience * 0.05;
    self.stats.health = (self.stats.health + regen).min(100.0);

    self.update_mood();
}
```

Notice the pattern: each stat uses a **base rate** plus a **gene modifier**. The genome makes each creature feel different — a high-appetite creature gets hungry faster, a resilient creature heals more quickly.

This is a great example of how simple functions create emergent complexity in games.

## Checkpoint

After this chapter, you should understand:

- Functions declare parameter types and return types explicitly
- The last expression (without `;`) is the return value
- Closures capture variables from their environment with `|params| body`
- Iterators process sequences lazily and at zero cost
- Iterator chains (`.filter().map().collect()`) replace explicit loops
- `for`, `loop`, `while` for different loop patterns
- `let else` for early returns on pattern failure

In Chapter 6, we'll dive into references and borrowing — the rules that govern how data is shared and the foundation of Rust's safety guarantees.
