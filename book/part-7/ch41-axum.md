# Chapter 41: Web Backend with Axum

> *"Your Kobara lives on your phone. But what if it could live on the web too?"*

In this chapter, you'll build a REST API in Rust using **Axum** — the same language you used for the game. By the end, you'll have a running server that serves creature data as JSON, ready for a frontend to consume.

---

## Why Full-Stack Rust?

Most web developers use different languages for backend and frontend — Python or Node.js on the server, JavaScript in the browser. We're going to use **Rust for everything**.

Why?

1. **One language to master** — the ownership, borrowing, and type system you learned building Kokoro applies directly to web code
2. **Shared types** — your `Genome` struct works in both the game and the API
3. **Performance** — Rust web servers handle thousands of requests per second with minimal memory
4. **Safety** — the compiler catches bugs at compile time, not at 3am in production

## The Stack

| Layer | Tool | Why |
|-------|------|-----|
| **HTTP Server** | Axum | Built by the Tokio team, ergonomic, type-safe |
| **Async Runtime** | Tokio | The standard async runtime for Rust |
| **Serialization** | Serde | You already know this from the game |
| **Database** | SQLite (rusqlite) | Same DB the game uses |
| **Frontend** (Ch43) | Leptos | Rust → WebAssembly, reactive UI |

## Setup

The project structure already exists at `kokoro-web/api/`. Let's look at what's there:

```
kokoro-web/
├── api/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── ui/          ← We'll build this in Chapter 43
```

### Cargo.toml

```toml
[package]
name = "kokoro-api"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower-http = { version = "0.6", features = ["cors"] }
```

Each dependency has a role:
- **axum**: the web framework — routes, handlers, extractors
- **tokio**: async runtime — lets your server handle many requests concurrently
- **serde + serde_json**: serialize Rust structs to JSON (you already use this in the game!)
- **tower-http**: middleware — we'll use it for CORS (Cross-Origin Resource Sharing)

### Running It

```bash
cd kokoro-web/api
cargo run
```

You should see:
```
Kokoro API running on http://localhost:3000
```

Open `http://localhost:3000/health` in your browser. You'll see:
```json
{"status":"ok","version":"0.1.0"}
```

That's your first Rust web endpoint. Let's understand how it works.

---

## Anatomy of an Axum App

```rust
use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct Health {
    status: String,
    version: String,
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("Kokoro API running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
```

### Key Concepts

**`async fn`** — Web servers need to handle many requests at once. The `async` keyword tells Rust this function can be paused and resumed, allowing other requests to be processed while one is waiting (for a database query, for example).

**`#[tokio::main]`** — Transforms your `main()` into an async function running on the Tokio runtime. Without this, you can't use `await`.

**`Router`** — Maps URL paths to handler functions. `get(health)` means "when someone makes a GET request to this path, call the `health` function."

**`Json<T>`** — Axum's way of saying "serialize this struct as JSON and return it with the correct `Content-Type: application/json` header." It uses Serde under the hood — the same `#[derive(Serialize)]` you use in the game.

**`env!("CARGO_PKG_VERSION")`** — A compile-time macro that reads the version from your `Cargo.toml`. No hardcoded strings.

---

## Adding Routes: Creature Data

Let's add an endpoint that returns Kobara species information.

### Step 1: Define the Response Types

```rust
use serde::Serialize;

#[derive(Serialize)]
struct SpeciesInfo {
    name: String,
    classification: String,
    biome: String,
    description: String,
    gene_ranges: GeneRanges,
}

#[derive(Serialize)]
struct GeneRanges {
    curiosity: (f32, f32),
    appetite: (f32, f32),
    resilience: (f32, f32),
}
```

Notice: these structs mirror the game's `Species` and `Genome` data, but they're **separate types** designed for the API response. This is intentional — you don't want to expose your internal game structures directly. This is called a **DTO** (Data Transfer Object).

### Step 2: Create the Handler

```rust
use axum::extract::Path;

async fn get_species(Path(name): Path<String>) -> Result<Json<SpeciesInfo>, StatusCode> {
    let info = match name.to_lowercase().as_str() {
        "moluun" => SpeciesInfo {
            name: "Moluun".into(),
            classification: "K. moluunaris".into(),
            biome: "The Verdance (forests)".into(),
            description: "Round, soft, forest-dwelling Kobara.".into(),
            gene_ranges: GeneRanges {
                curiosity: (0.2, 1.0),
                appetite: (0.1, 0.8),
                resilience: (0.2, 1.0),
            },
        },
        "pylum" => SpeciesInfo {
            name: "Pylum".into(),
            classification: "K. pylumensis".into(),
            biome: "Veridian Highlands".into(),
            description: "Winged, curious Kobara from the highlands.".into(),
            gene_ranges: GeneRanges {
                curiosity: (0.4, 1.0),
                appetite: (0.1, 0.5),
                resilience: (0.3, 0.9),
            },
        },
        "skael" => SpeciesInfo {
            name: "Skael".into(),
            classification: "K. skaelith".into(),
            biome: "Abyssal Shallows (caves)".into(),
            description: "Scaled, resilient Kobara from underground caves.".into(),
            gene_ranges: GeneRanges {
                curiosity: (0.1, 0.7),
                appetite: (0.3, 1.0),
                resilience: (0.5, 1.0),
            },
        },
        "nyxal" => SpeciesInfo {
            name: "Nyxal".into(),
            classification: "K. nyxalaris".into(),
            biome: "Abyssal Depths (deep ocean)".into(),
            description: "Tentacled, intelligent Kobara from the deep.".into(),
            gene_ranges: GeneRanges {
                curiosity: (0.5, 1.0),
                appetite: (0.2, 0.7),
                resilience: (0.1, 0.6),
            },
        },
        _ => return Err(StatusCode::NOT_FOUND),
    };

    Ok(Json(info))
}
```

**New concept: `Path<String>`** — This is an Axum **extractor**. It pulls the `{name}` segment from the URL path. Axum uses the type system to parse request data — you declare what you need in the function signature, and Axum provides it.

**`Result<Json<T>, StatusCode>`** — The handler can either return JSON (success) or an HTTP status code (error). If the species isn't found, we return `404 NOT FOUND`.

### Step 3: Register the Route

```rust
let app = Router::new()
    .route("/health", get(health))
    .route("/api/species/{name}", get(get_species));
```

The `{name}` in the path is a **dynamic segment** — it matches any string and passes it to the `Path` extractor.

### Step 4: Test It

```bash
# In another terminal:
curl http://localhost:3000/api/species/moluun
```

```json
{
  "name": "Moluun",
  "classification": "K. moluunaris",
  "biome": "The Verdance (forests)",
  "description": "Round, soft, forest-dwelling Kobara.",
  "gene_ranges": {
    "curiosity": [0.2, 1.0],
    "appetite": [0.1, 0.8],
    "resilience": [0.2, 1.0]
  }
}
```

```bash
curl http://localhost:3000/api/species/unknown
# Returns: 404 Not Found
```

---

## Adding a Lore Endpoint

Let's serve the lore text for each biome:

```rust
#[derive(Serialize)]
struct BiomeInfo {
    name: String,
    description: String,
    species: String,
    atmosphere: String,
}

async fn get_biome(Path(name): Path<String>) -> Result<Json<BiomeInfo>, StatusCode> {
    let info = match name.to_lowercase().as_str() {
        "verdance" => BiomeInfo {
            name: "The Verdance".into(),
            description: "Vast bioluminescent forests with spiral-formation trees.".into(),
            species: "Moluun".into(),
            atmosphere: "Air thick with luminescent spores.".into(),
        },
        // ... other biomes
        _ => return Err(StatusCode::NOT_FOUND),
    };
    Ok(Json(info))
}
```

Register it:
```rust
.route("/api/biome/{name}", get(get_biome))
```

---

## Listing All Species

```rust
#[derive(Serialize)]
struct SpeciesSummary {
    name: String,
    biome: String,
}

async fn list_species() -> Json<Vec<SpeciesSummary>> {
    Json(vec![
        SpeciesSummary { name: "Moluun".into(), biome: "The Verdance".into() },
        SpeciesSummary { name: "Pylum".into(), biome: "Veridian Highlands".into() },
        SpeciesSummary { name: "Skael".into(), biome: "Abyssal Shallows".into() },
        SpeciesSummary { name: "Nyxal".into(), biome: "Abyssal Depths".into() },
    ])
}
```

Register: `.route("/api/species", get(list_species))`

---

## Middleware: CORS

When your frontend (running in the browser) tries to call your API, the browser blocks it by default — this is a security feature called **CORS** (Cross-Origin Resource Sharing). We need to tell the browser our API is safe to call:

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

let app = Router::new()
    .route("/health", get(health))
    .route("/api/species", get(list_species))
    .route("/api/species/{name}", get(get_species))
    .layer(cors);
```

**`.layer(cors)`** wraps all routes with CORS headers. `Any` is permissive — in production, you'd restrict this to your domain.

---

## Error Handling

Right now, unknown species return a bare `404`. Let's return a proper JSON error:

```rust
use axum::response::IntoResponse;

#[derive(Serialize)]
struct ApiError {
    error: String,
    code: u16,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}
```

Now your handlers can return:
```rust
async fn get_species(Path(name): Path<String>) -> Result<Json<SpeciesInfo>, ApiError> {
    match name.to_lowercase().as_str() {
        "moluun" => { /* ... */ },
        _ => Err(ApiError {
            error: format!("Species '{}' not found", name),
            code: 404,
        }),
    }
}
```

---

## The Complete main.rs

After this chapter, your `kokoro-web/api/src/main.rs` should have:
- Health check endpoint
- List all species
- Get species details by name
- CORS middleware
- Proper error responses

---

## Rust Concepts Learned

| Concept | Game Equivalent | Web Usage |
|---------|----------------|-----------|
| `#[derive(Serialize)]` | Genome, Mind persistence | JSON API responses |
| `match` with patterns | FSM mood transitions | Route matching |
| `Result<T, E>` | Database operations | HTTP success/error |
| Traits (`IntoResponse`) | Bevy Components/Resources | Custom response types |
| `async/await` | (new!) | Non-blocking I/O |
| Generics (`Json<T>`) | `Query<&T>` in Bevy | Type-safe extractors |

---

## Exercises

### Level 1: Practice
1. Add a `/api/biome/{name}` endpoint that returns biome details for all 4 biomes
2. Change the health endpoint to include the current timestamp
3. What happens if you remove the `#[derive(Serialize)]` from `SpeciesInfo`? Try it and read the error

### Level 2: Build It
4. Add a `/api/lore/mysteries` endpoint that returns a list of the Open Mysteries from the lore
5. Create a `GET /api/species/{name}/genes` endpoint that returns the gene ranges for a species
6. Add request logging — print each request's method and path to the console using `tower_http::trace`

### Level 3: Explore
7. Research Axum's `State` extractor. How would you share a database connection across handlers? (Hint: this is what Chapter 42 covers)
8. Compare Axum with Actix-web and Rocket. What are the trade-offs?
9. Use `reqwest` to write a Rust program that calls your API and prints the species list

---

## What's Next

In Chapter 42, we'll add **authentication** — user registration, login, and JWT tokens. Your API will know *who* is asking for creature data, which means each player can have their own profile and their own Kobaras on the web.

But first: make sure your API runs, test all endpoints with `curl` or your browser, and complete at least the Level 2 exercises. The API you build here is the foundation for everything that follows.

```bash
cd kokoro-web/api && cargo run
# Test: curl http://localhost:3000/api/species
# Test: curl http://localhost:3000/api/species/nyxal
```
