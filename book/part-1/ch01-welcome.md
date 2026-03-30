# Chapter 1: Welcome to Rust

## Why Rust?

Imagine a language that runs as fast as C, but won't let you accidentally corrupt memory. A language where the compiler catches bugs that would take hours to debug in other languages. A language that feels modern — with closures, pattern matching, and a package manager — while giving you the control to write operating systems, game engines, or embedded firmware.

That language is Rust.

Rust was created at Mozilla Research in 2010 and has been the "most loved programming language" in developer surveys for years running. It's used by companies like Amazon, Microsoft, Google, Discord, and Cloudflare — not because it's trendy, but because it solves a fundamental problem: **how do you write fast software that doesn't crash?**

The answer is Rust's **ownership system** — a set of rules enforced at compile time that guarantee memory safety without a garbage collector. We'll explore this system deeply in Chapter 2, and by the end of this book you'll understand it so intuitively that you'll miss it in other languages.

### Why Rust for Games?

Games are one of the most demanding types of software. They need to:

- Process physics, AI, and rendering **60 times per second**
- Manage thousands of entities without memory leaks
- Handle complex state transitions without crashing
- Run on multiple platforms (desktop, mobile, web)

Traditional game languages make tradeoffs. C++ gives you speed but memory bugs are constant. C# (Unity) and GDScript (Godot) are safer but add garbage collection pauses. Rust gives you **both**: C++-level performance with compile-time safety guarantees.

We'll build our game using **Bevy**, a modern Rust game engine built on the Entity Component System (ECS) pattern. Bevy is open-source, fast, and designed to feel natural to Rust developers.

## What We're Building

**Kokoro** (心 — Japanese for "heart/spirit") is a virtual creature game. Think Tamagotchi meets Pokemon, with a twist: each creature has a **genome** that shapes its personality and appearance, and a **neural network** that learns from how you interact with it.

Here's what the finished project includes:

- **Three species** — Moluun (round, friendly), Pylum (winged, curious), Skael (scaled, resilient)
- **Genetic system** — Each creature has a unique genome that affects behavior, appearance, and stats
- **Emotional AI** — A finite state machine combined with a neural network that learns your play patterns
- **Modular visuals** — Creatures assembled from body parts, with mood-reactive sprites and genome-driven variation
- **Persistence** — SQLite database saves everything: stats, genome, neural weights, event history
- **Visual evolution** — Creatures grow, gain accessories, and change appearance over their lifetime

And you'll build every piece of it, learning Rust along the way.

## Installing Rust

Rust's official installer is called `rustup`. It manages your Rust toolchain (compiler, standard library, tools) and makes updating painless.

### On macOS or Linux

Open a terminal and run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts (the defaults are fine). Then restart your terminal or run:

```bash
source $HOME/.cargo/env
```

### On Windows

Download and run the installer from [rustup.rs](https://rustup.rs). You'll also need the Visual Studio C++ Build Tools — the installer will guide you.

### Verify the Installation

```bash
rustc --version
cargo --version
```

You should see something like:

```
rustc 1.83.0 (2024-11-28)
cargo 1.83.0 (2024-11-28)
```

The exact version doesn't matter as long as it's reasonably recent (1.75+).

## Your First Program: Hello, Kokoro!

Let's create the project. Open a terminal and run:

```bash
cargo new kokoro
cd kokoro
```

`cargo new` created a directory with this structure:

```
kokoro/
├── Cargo.toml    # Project manifest — name, version, dependencies
└── src/
    └── main.rs   # Entry point — where execution begins
```

Open `src/main.rs`. Cargo generated a starter program:

```rust
fn main() {
    println!("Hello, world!");
}
```

Let's break this down:

- `fn main()` — declares the `main` function, the program's entry point
- `println!` — a **macro** (the `!` tells you it's a macro, not a regular function) that prints text to the terminal
- Every statement ends with `;`
- Curly braces `{}` define code blocks

Let's make it ours. Change the file to:

```rust
fn main() {
    println!("Welcome to Kokoro!");
    println!("A creature is waiting to be born...");
}
```

Now run it:

```bash
cargo run
```

You'll see Cargo compile the project and print:

```
   Compiling kokoro v0.1.0 (/path/to/kokoro)
    Finished dev [unoptimized + debuginfo] target(s)
     Running `target/debug/kokoro`
Welcome to Kokoro!
A creature is waiting to be born...
```

Congratulations — you just wrote and ran a Rust program.

## Understanding Cargo

Cargo is Rust's build system and package manager. Coming from other languages:

| What you know | Cargo equivalent |
|---|---|
| `npm` / `package.json` (JavaScript) | `cargo` / `Cargo.toml` |
| `pip` / `requirements.txt` (Python) | `cargo` / `Cargo.toml` |
| `maven` / `pom.xml` (Java) | `cargo` / `Cargo.toml` |

The key commands you'll use throughout this book:

```bash
cargo run        # Compile and run the project
cargo build      # Compile without running
cargo check      # Check for errors without producing a binary (fastest)
cargo test       # Run tests
cargo add bevy   # Add a dependency
```

### Cargo.toml — The Project Manifest

Open `Cargo.toml`:

```toml
[package]
name = "kokoro"
version = "0.1.0"
edition = "2021"

[dependencies]
```

- `name` — the project name
- `version` — [semantic versioning](https://semver.org)
- `edition` — which Rust edition to use (2021 is current)
- `[dependencies]` — where we'll add external libraries (called "crates")

By the end of this book, our `[dependencies]` section will look like this:

```toml
[dependencies]
bevy = "0.16"
serde = "1.0"
bincode = { version = "2.0", features = ["serde"] }
rand = "0.9"
rusqlite = { version = "0.31", features = ["bundled"] }
```

Each dependency is a crate published on [crates.io](https://crates.io), Rust's package registry. We'll add them one at a time as we need them.

## Adding Bevy — Our Game Engine

Let's add Bevy right away so we can see a window. Run:

```bash
cargo add bevy
```

Cargo will download Bevy and all its dependencies. The first compile will take a few minutes — that's normal. Subsequent compiles will be fast.

Now replace `src/main.rs` with:

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kokoro".into(),
                resolution: (400.0, 700.0).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .run();
}
```

Run it:

```bash
cargo run
```

A window appears! It's empty and gray, but it's a real game window powered by Bevy. Let's understand what we wrote:

- `use bevy::prelude::*` — imports everything commonly needed from Bevy
- `App::new()` — creates a new Bevy application
- `.add_plugins(DefaultPlugins...)` — adds Bevy's built-in systems (rendering, input, audio, etc.)
- `WindowPlugin { ... }` — configures the game window (title, size)
- `..default()` — fills in remaining fields with default values (we'll learn about this syntax in Chapter 3)
- `.into()` — converts one type to another (we'll cover the `Into` trait in Chapter 9)
- `.run()` — starts the game loop

Don't worry if some syntax feels unfamiliar. Every piece will be explained in the coming chapters. The important thing is: **you have a running game window in under 15 lines of Rust**.

## What's Next

In Chapter 2, we'll learn about variables, types, and Rust's ownership model — the foundation everything else builds on. We'll start defining the data structures that will become our creature's genome.

## Checkpoint

After this chapter, your project should have:

```
kokoro/
├── Cargo.toml          # With bevy dependency
├── Cargo.lock          # Auto-generated dependency versions
├── src/
│   └── main.rs         # Bevy app with a 400x700 window
└── target/             # Build artifacts (auto-generated)
```

Running `cargo run` should open a window titled "Kokoro".
