# Chapter 6: References and Borrowing

> *"The borrowing rules are Rust's most valuable constraint. They make concurrent bugs impossible and force you to design cleaner APIs."*

## The Problem Borrowing Solves

In Chapter 2, we learned that each value has exactly one owner. But what if two functions need to read the same data? We don't want to clone it every time — that's wasteful. We need a way to **share access without transferring ownership**.

That's what references do.

## Immutable References: `&T`

An immutable reference lets you read data without owning it:

```rust
fn display_stats(stats: &VitalStats) {
    println!("Hunger: {:.1}", stats.hunger);
    println!("Health: {:.1}", stats.health);
}

fn main() {
    let stats = VitalStats::new();
    display_stats(&stats);   // Borrow stats
    display_stats(&stats);   // Can borrow again — we never gave up ownership
}
```

The `&` creates a reference. The function **borrows** the data temporarily. When the function returns, the borrow ends and the owner retains full control.

**You can have as many immutable references as you want simultaneously:**

```rust
let genome = Genome::random_for(Species::Marumi);
let r1 = &genome;
let r2 = &genome;
let r3 = &genome;
// All three references are valid at the same time
println!("{:?} {:?} {:?}", r1.hue, r2.curiosity, r3.species);
```

This is safe because no one can modify the data while it's being read.

## Mutable References: `&mut T`

A mutable reference lets you modify data:

```rust
fn heal(stats: &mut VitalStats, amount: f32) {
    stats.health = (stats.health + amount).min(100.0);
}

fn main() {
    let mut stats = VitalStats::new();
    heal(&mut stats, 25.0);
}
```

**The critical rule: you can have exactly one mutable reference, OR any number of immutable references, but not both at the same time.**

```rust
let mut stats = VitalStats::new();
let r1 = &stats;           // Immutable borrow
let r2 = &mut stats;       // ERROR: cannot borrow as mutable while immutable borrow exists
println!("{}", r1.hunger);  // r1 is still in use here
```

This rule prevents **data races** at compile time. If something is being read, nothing can modify it. If something is being modified, nothing else can access it.

## In Practice: The Crossover Borrow Problem

When building Kokoro's genetic crossover, we hit a real borrow issue. Here's the original code that failed:

```rust
fn crossover(parent_a: &Genome, parent_b: &Genome, species: Species) -> Genome {
    let mut rng = rand::rng();

    // These closures both capture `&mut rng`
    let pick = |a: f32, b: f32| -> f32 {
        if rng.random_bool(0.5) { a } else { b }  // Borrows rng mutably
    };
    let mutate = |val: f32| -> f32 {
        if rng.random_bool(0.15) {                 // Borrows rng mutably AGAIN
            (val + rng.random_range(-0.1..0.1)).clamp(0.0, 1.0)
        } else { val }
    };

    // ERROR: both closures borrow `rng` mutably at the same time
    Genome {
        curiosity: mutate(pick(parent_a.curiosity, parent_b.curiosity)),
        // ...
    }
}
```

The compiler caught this: `pick` and `mutate` both need `&mut rng`, but they're called in the same expression. Rust won't allow two mutable borrows simultaneously.

**The fix:** convert closures to standalone functions that receive `rng` explicitly:

```rust
fn pick(rng: &mut impl Rng, a: f32, b: f32) -> f32 {
    if rng.random_bool(0.5) { a } else { b }
}

fn mutate(rng: &mut impl Rng, val: f32) -> f32 {
    if rng.random_bool(0.15) {
        (val + rng.random_range(-0.1..0.1)).clamp(0.0, 1.0)
    } else { val }
}

pub fn crossover(parent_a: &Genome, parent_b: &Genome, species: Species) -> Genome {
    let mut rng = rand::rng();

    // Now each function borrows rng sequentially, not simultaneously
    let curiosity = pick(&mut rng, parent_a.curiosity, parent_b.curiosity);
    let curiosity = mutate(&mut rng, curiosity);
    // ...
}
```

By using intermediate `let` bindings, each mutable borrow starts and ends before the next one begins. The compiler is satisfied, and we've eliminated a potential bug in the process.

## Lifetimes — The Short Version

Lifetimes ensure that references don't outlive the data they point to:

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

The `'a` annotation says: "the returned reference lives at least as long as both input references."

Most of the time, Rust infers lifetimes automatically (called **lifetime elision**). In Kokoro, we rarely write lifetime annotations because our data flows through Bevy's resource system, which manages lifetimes for us.

We'll revisit lifetimes in more detail in Part V when we discuss advanced patterns.

## Checkpoint

After this chapter, you should understand:

- `&T` borrows data immutably — many simultaneous readers allowed
- `&mut T` borrows data mutably — exclusive access required
- You can't have `&T` and `&mut T` to the same data at the same time
- These rules prevent data races at compile time
- When borrow conflicts arise, restructure code to make borrows sequential
- Lifetimes ensure references don't outlive their data

---

*Chapters 7-10 continue with error handling, collections/generics, traits, and module organization. Each follows the same pattern: concept → Kokoro example → build it → deep dive.*
