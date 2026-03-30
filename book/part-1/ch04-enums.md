# Chapter 4: Enums and Pattern Matching

If structs are Rust's answer to "how do I group data together?", enums are the answer to "how do I represent a value that can be one of several things?" Combined with `match`, they become one of Rust's most powerful features.

## Defining Enums

An enum defines a type that can be one of several **variants**:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum MoodState {
    Happy,
    Hungry,
    Tired,
    Lonely,
    Playful,
    Sick,
    Sleeping,
}
```

This is from Kokoro's `src/mind/mod.rs`. A creature's mood is always exactly one of these seven states — never none, never two at once, never something unlisted. The compiler enforces this.

Compare this to how you might model mood in other languages:

```python
# Python — mood is just a string. Typos are silent bugs.
mood = "hapy"  # Oops — no error, but nothing will match it

# JavaScript — mood could be anything
let mood = null;     // valid but dangerous
let mood = 42;       // valid but nonsensical
```

In Rust, `MoodState` can only hold a valid variant. There's no null, no typos, no invalid states.

## Enums with Data

Unlike enums in C or Java, Rust enums can carry data:

```rust
#[derive(Event)]
pub struct SelectSpeciesEvent {
    pub species: Species,
}

// But we could also model it as an enum:
enum CreatureEvent {
    Feed,
    Play,
    Sleep,
    SelectSpecies(Species),     // Carries a Species value
    SwitchTo { index: usize },  // Carries a named field
}
```

Each variant can hold different types and amounts of data. This is called an **algebraic data type** (or "tagged union" in other languages).

## Pattern Matching with `match`

`match` is how you work with enums. It's like `switch` in other languages, but much more powerful:

```rust
impl MoodState {
    /// Returns the sprite asset key for this mood.
    pub fn mood_key(&self) -> &str {
        match self {
            MoodState::Happy => "idle",
            MoodState::Hungry => "hungry",
            MoodState::Tired => "tired",
            MoodState::Lonely => "lonely",
            MoodState::Playful => "playful",
            MoodState::Sick => "sick",
            MoodState::Sleeping => "sleeping",
        }
    }
}
```

### Exhaustiveness

The most important property of `match` is that it's **exhaustive** — you must handle every variant. If you add a new mood state later, the compiler will tell you every `match` that needs updating:

```rust
// If we add: MoodState::Excited
// The compiler immediately errors everywhere:
// "non-exhaustive patterns: `Excited` not covered"
```

This is huge for maintenance. In Python or JavaScript, adding a new state means searching the entire codebase for string comparisons. In Rust, the compiler finds them all for you.

### Matching with Data

When variants carry data, `match` can destructure them:

```rust
fn describe_action(event: &CreatureEvent) {
    match event {
        CreatureEvent::Feed => println!("Feeding the creature"),
        CreatureEvent::Play => println!("Playing with the creature"),
        CreatureEvent::Sleep => println!("Putting creature to sleep"),
        CreatureEvent::SelectSpecies(species) => {
            println!("Selecting species: {:?}", species);
        }
        CreatureEvent::SwitchTo { index } => {
            println!("Switching to creature #{}", index);
        }
    }
}
```

The variable names in the pattern (`species`, `index`) bind to the data inside the variant.

### Match Guards and Wildcards

```rust
fn mood_priority(mood: &MoodState, health: f32) -> &str {
    match mood {
        MoodState::Sick if health < 20.0 => "CRITICAL",  // Guard condition
        MoodState::Sick => "concerning",
        MoodState::Sleeping => "resting",
        _ => "normal",                                     // Wildcard: matches anything
    }
}
```

- `if health < 20.0` is a **match guard** — extra condition on a pattern
- `_` is the **wildcard** — matches anything not already covered

## `Option<T>` — Rust's Null Replacement

Rust has no null. Instead, it has `Option<T>`:

```rust
enum Option<T> {
    Some(T),    // A value exists
    None,       // No value
}
```

This is used everywhere in Kokoro:

```rust
// From body_parts.rs — a part may or may not have a custom fallback color
pub struct BodyPartDef {
    pub slot: String,
    pub fallback_shape: FallbackShape,
    pub fallback_color: Option<Color>,   // None = use body_color
    // ...
}
```

Working with `Option`:

```rust
// Get position from rig, or fall back to origin
let (offset, z_depth) = anchor_map
    .get(&part_def.slot)                    // Returns Option<&ResolvedAnchor>
    .map(|a| (a.position, a.z_depth))       // Transform the inner value
    .unwrap_or((Vec2::ZERO, 0.0));          // Provide a default if None
```

Common `Option` methods:

| Method | What it does |
|---|---|
| `.unwrap()` | Get the value or **panic** (crash). Use only when you're certain. |
| `.unwrap_or(default)` | Get the value or use a default |
| `.map(f)` | Transform `Some(x)` to `Some(f(x))`, leave `None` as `None` |
| `.and_then(f)` | Like `map`, but `f` returns an `Option` (flat map) |
| `.is_some()` / `.is_none()` | Check without extracting |

The beauty of `Option` is that the compiler **forces** you to handle the `None` case. You can't accidentally dereference a null pointer because null doesn't exist.

## `Result<T, E>` — Handling Errors

`Result` is like `Option` but for operations that can fail:

```rust
enum Result<T, E> {
    Ok(T),     // Success, with a value
    Err(E),    // Failure, with an error
}
```

From Kokoro's persistence system:

```rust
pub fn log_event(
    conn: &rusqlite::Connection,
    tick: u64,
    event_type: &str,
    payload: Option<&str>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO event_log (tick, event_type, payload) VALUES (?1, ?2, ?3)",
        (tick, event_type, payload),
    )?;   // The ? operator — we'll cover this in Chapter 7
    Ok(())
}
```

We'll explore `Result` and error handling deeply in Chapter 7.

## `if let` — When You Only Care About One Variant

Sometimes you only want to handle one case:

```rust
// Full match — verbose for this case
match collection.creatures.get(btn.0) {
    Some(creature) => {
        select_events.write(SelectSpeciesEvent {
            species: creature.genome.species.clone(),
        });
    }
    None => {}   // Do nothing
}

// if let — much cleaner
if let Some(creature) = collection.creatures.get(btn.0) {
    select_events.write(SelectSpeciesEvent {
        species: creature.genome.species.clone(),
    });
}
```

`if let` is syntactic sugar for a `match` with one interesting arm and a wildcard. Use it when you only care about one variant.

## The `matches!` Macro

For simple pattern checks that return a boolean:

```rust
// From mind/mod.rs
pub fn is_critical(&self) -> bool {
    matches!(self.mood, MoodState::Sick | MoodState::Sleeping)
}
```

`matches!` returns `true` if the value matches the pattern. The `|` means "or" — so this returns `true` for Sick **or** Sleeping. Much cleaner than:

```rust
pub fn is_critical(&self) -> bool {
    match self.mood {
        MoodState::Sick | MoodState::Sleeping => true,
        _ => false,
    }
}
```

## Build It: The Complete MoodState

Let's build the mood system as it exists in Kokoro. This combines everything from this chapter:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum MoodState {
    Happy,
    Hungry,
    Tired,
    Lonely,
    Playful,
    Sick,
    Sleeping,
}

impl MoodState {
    /// Returns the sprite asset key for this mood.
    pub fn mood_key(&self) -> &str {
        match self {
            MoodState::Happy => "idle",
            MoodState::Hungry => "hungry",
            MoodState::Tired => "tired",
            MoodState::Lonely => "lonely",
            MoodState::Playful => "playful",
            MoodState::Sick => "sick",
            MoodState::Sleeping => "sleeping",
        }
    }

    /// Returns true if this mood is critical (FSM has absolute authority).
    pub fn is_critical(&self) -> bool {
        matches!(self, MoodState::Sick | MoodState::Sleeping)
    }
}
```

Notice how `match` and `matches!` make the state machine logic crystal clear. Every mood has a sprite key. Exactly two moods are critical. If we add a new mood variant, the compiler tells us everywhere that needs updating.

This is the power of Rust enums: **making invalid states unrepresentable and ensuring all valid states are handled**.

## Deep Dive: Enums vs Inheritance

In object-oriented languages, you might model moods with inheritance:

```java
abstract class MoodState { }
class Happy extends MoodState { }
class Hungry extends MoodState { }
// ...
```

The problem? Anyone can add a subclass. Your `switch` statements over mood types are never exhaustive — a new subclass can silently pass through without being handled.

Rust enums are **closed** — all variants are defined in one place. This means `match` can guarantee exhaustiveness. It's a different design philosophy: instead of "anyone can extend this," it's "all possibilities are known and handled."

For game development, where state machines are everywhere, this is a significant advantage.

## Checkpoint

After this chapter, you should understand:

- Enums define types with a fixed set of variants
- Variants can carry data (unlike C-style enums)
- `match` is exhaustive — every variant must be handled
- `Option<T>` replaces null: `Some(value)` or `None`
- `Result<T, E>` handles errors: `Ok(value)` or `Err(error)`
- `if let` handles a single variant concisely
- `matches!` checks patterns and returns `bool`
- Enums make invalid states unrepresentable

In Chapter 5, we'll explore functions, closures, and the iterator system that makes Rust code both expressive and efficient.
