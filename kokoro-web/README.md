# Kokoro Web

Full-stack Rust companion website for the Kokoro game.

## Structure

```
kokoro-web/
├── api/          # Backend — Axum REST API
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── ui/           # Frontend — Leptos (Rust → WebAssembly)
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## Getting Started

Follow the book (Chapter 33+) to build this step by step.

### Prerequisites

```bash
cargo install cargo-leptos
rustup target add wasm32-unknown-unknown
```

### Running

```bash
# Backend
cd api && cargo run

# Frontend
cd ui && cargo leptos watch
```
