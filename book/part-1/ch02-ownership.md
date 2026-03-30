# Chapter 2: Variables, Types, and Ownership

Ownership is the concept that makes Rust unique among programming languages. Before we get there, we need to understand variables and types — but keep in mind that everything in this chapter builds toward that central idea.

## Variables and Mutability

In Rust, variables are **immutable by default**. This might surprise you if you come from Python, JavaScript, or Java where everything is mutable.

```rust
fn main() {
    let hunger = 50.0;
    println!("Hunger: {hunger}");

    hunger = 75.0; // ERROR: cannot assign twice to immutable variable
}
```

The compiler stops you. To make a variable mutable, you add `mut`:

```rust
fn main() {
    let mut hunger = 50.0;
    println!("Hunger: {hunger}");  // 50.0

    hunger = 75.0;
    println!("Hunger: {hunger}");  // 75.0
}
```

**Why immutable by default?** Because bugs love mutation. When a value can change anywhere, it's hard to reason about what it is at any point. Rust forces you to be explicit about what can change. This seems restrictive at first, but you'll find that most variables genuinely don't need to change — and the ones that do are clearly marked with `mut`.

### Constants

For values that are truly fixed forever, use `const`:

```rust
const MAX_HUNGER: f32 = 100.0;
const STAT_DECAY_RATE: f32 = 0.15;
```

Constants must have a type annotation (`: f32`) and their value must be computable at compile time. By convention, they use `SCREAMING_SNAKE_CASE`.

In Kokoro, you'll see constants like these defining the boundaries of our creature's world.

## Primitive Types

Rust is **statically typed** — every value has a known type at compile time. Often the compiler can infer the type, but it's good to know them explicitly.

### Integers

| Type | Size | Range |
|------|------|-------|
| `i8` | 8-bit | -128 to 127 |
| `i16` | 16-bit | -32,768 to 32,767 |
| `i32` | 32-bit | ~-2 billion to ~2 billion |
| `i64` | 64-bit | very large |
| `u8` | 8-bit | 0 to 255 |
| `u16` | 16-bit | 0 to 65,535 |
| `u32` | 32-bit | 0 to ~4 billion |
| `u64` | 64-bit | very large |
| `usize` | pointer-sized | used for indexing |

The prefix tells you: `i` = signed (can be negative), `u` = unsigned (always ≥ 0). The number tells you how many bits.

```rust
let age_ticks: u64 = 0;         // unsigned 64-bit — creature age in ticks
let active_index: usize = 0;    // pointer-sized — used for Vec indexing
let offspring_count: u32 = 0;    // unsigned 32-bit — breeding counter
```

> **In Kokoro**, we use `u64` for tick counts (they can grow very large over a creature's lifetime) and `usize` for collection indices (Rust requires `usize` for array/vector indexing).

### Floating Point

| Type | Size | Precision |
|------|------|-----------|
| `f32` | 32-bit | ~7 decimal digits |
| `f64` | 64-bit | ~15 decimal digits |

```rust
let hunger: f32 = 50.0;
let curiosity: f32 = 0.72;
let hue: f32 = 180.0;
```

> **In Kokoro**, virtually all game values are `f32`. Stats range from 0.0 to 100.0, genes range from 0.0 to 1.0, and colors use hue values from 0.0 to 360.0. We don't need `f64` precision — and `f32` is faster, especially for the GPU.

### Booleans and Characters

```rust
let is_critical: bool = true;
let initial: char = 'K';   // Unicode character — 4 bytes
```

## Strings: `String` vs `&str`

Strings in Rust have two main types, and understanding the difference is your first encounter with ownership thinking:

```rust
let name: String = String::from("Marumi #1");   // Owned, heap-allocated, growable
let label: &str = "Marumi";                      // Borrowed reference to string data
```

- `String` — **owns** its data. Allocated on the heap. Can be modified. Freed when it goes out of scope.
- `&str` — a **reference** (or "slice") to string data. Doesn't own anything. Lightweight. Read-only.

Think of it like this: `String` is the book you bought and own. `&str` is someone letting you read over their shoulder. You can look at it, but you didn't buy it and you can't scribble in the margins.

```rust
// In Kokoro's creature collection:
pub struct StoredCreature {
    pub name: String,    // The creature OWNS its name
    // ...
}

// But when displaying it, we just borrow:
fn display_name(name: &str) {
    println!("Active: {name}");
}
```

This distinction between owned and borrowed data is everywhere in Rust. Let's understand why.

## The Ownership Model

This is the most important section in the entire book. Rust's ownership model is the feature that eliminates entire categories of bugs — null pointer dereferences, use-after-free, double-free, data races — all at compile time, with zero runtime cost.

### The Three Rules

1. **Each value in Rust has exactly one owner** (a variable)
2. **There can only be one owner at a time**
3. **When the owner goes out of scope, the value is dropped** (memory is freed)

Let's see this in action:

```rust
fn main() {
    let creature_name = String::from("Marumi");  // creature_name owns the String

    let selected = creature_name;  // Ownership MOVES to selected

    println!("{creature_name}");   // ERROR: creature_name no longer owns the data
}
```

This is a **move**. When you assign a `String` to another variable, ownership transfers. The original variable becomes invalid. Rust does this to ensure that exactly one variable is responsible for freeing the memory.

If you're coming from Python or JavaScript, this feels strange. In those languages, `selected = creature_name` would create a second reference to the same data. But that creates a problem: who frees the memory? If both try, you get a double-free bug. If neither does, you get a memory leak. Garbage collectors solve this at runtime (with performance cost). Rust solves it at compile time.

### Copy Types

Not everything moves. Small, simple types that live on the **stack** implement the `Copy` trait, which means they're copied instead of moved:

```rust
let hunger: f32 = 50.0;
let saved_hunger = hunger;   // COPY — both variables are valid
println!("{hunger}");         // This works fine!
```

Copy types include: all integers, floats, `bool`, `char`, and tuples of Copy types.

Non-Copy types include: `String`, `Vec<T>`, and most structs (unless you explicitly derive `Copy`).

> **In Kokoro**, our gene values (`f32`) are Copy, so passing them around is effortless. But `Genome` and `Mind` are not Copy — they contain `String` fields and complex data. When we need a second copy, we explicitly call `.clone()`:

```rust
// From collection.rs — saving current creature state
current.genome = genome.clone();
current.mind = mind.clone();
```

The `.clone()` call makes our intention explicit: "Yes, I want to duplicate this data." Rust never silently copies expensive data.

### Clone vs Copy

| | Copy | Clone |
|---|---|---|
| When | Happens automatically on assignment | Must call `.clone()` explicitly |
| Cost | Cheap (stack copy of a few bytes) | Potentially expensive (heap allocation) |
| Types | Primitives, small fixed-size data | Any type that implements Clone |
| Implicit? | Yes | No — you must ask for it |

This is a design principle in Rust: **cheap things are implicit, expensive things are explicit**. You'll see this pattern throughout the language.

## Putting It Together: Our First Data Structure

Let's define something that will grow into Kokoro's vital stats. Create a new file at `src/stats.rs`:

```rust
/// The creature's vital statistics.
/// Each stat ranges from 0.0 (depleted) to 100.0 (full).
pub struct VitalStats {
    pub hunger: f32,
    pub happiness: f32,
    pub energy: f32,
    pub health: f32,
}
```

And add a way to create default stats:

```rust
impl VitalStats {
    pub fn new() -> Self {
        VitalStats {
            hunger: 50.0,
            happiness: 50.0,
            energy: 80.0,
            health: 100.0,
        }
    }
}
```

We just used several concepts from this chapter:
- `pub` — makes the struct and its fields visible outside the module
- `f32` — floating point for smooth stat values
- `impl` — adds methods to our struct (more in Chapter 3)
- `Self` — refers to the type being implemented (`VitalStats`)

Now think about ownership. Who owns a `VitalStats`? In Kokoro, the `Mind` struct owns it:

```rust
pub struct Mind {
    pub stats: VitalStats,   // Mind OWNS the VitalStats
    pub mood: MoodState,
    pub age_ticks: u64,
}
```

When a `Mind` is dropped (freed), its `VitalStats` is automatically dropped too. No memory leaks, no manual cleanup, no garbage collector.

## Deep Dive: Why Rust Works This Way

If you've used C or C++, you know the pain of manual memory management. Every `malloc` needs a `free`. Every `new` needs a `delete`. Forget one and you leak memory. Do it twice and you crash. Use memory after freeing it and you get undefined behavior — potentially a security vulnerability.

If you've used Python, JavaScript, or Java, you've never worried about this because a garbage collector handles it. But garbage collectors have costs: memory overhead, unpredictable pauses, and no deterministic cleanup.

Rust's ownership model is a third way: the compiler figures out where to free memory based on scope rules, and inserts the cleanup code automatically. It's as safe as garbage collection and as fast as manual management. The trade-off is that you must satisfy the compiler's rules, which is a learning curve — but once you internalize them, they become second nature.

## Checkpoint

After this chapter, you should understand:

- Variables are immutable by default; use `mut` when you need mutation
- Rust has fixed-size numeric types (`f32`, `u64`, `usize`, etc.)
- `String` owns its data; `&str` borrows it
- Each value has exactly one owner
- Assignment of non-Copy types is a **move** (original becomes invalid)
- Copy types (primitives) are duplicated automatically
- `.clone()` creates an explicit deep copy of non-Copy types

In the next chapter, we'll explore structs in depth — how to define them, add methods, derive useful traits, and build the `Genome` struct that gives each creature its unique identity.
