# Phase 11: Web Enhancement — Full Sync + Stripe

## Context

The game (Bevy) and web stack (Axum API + Leptos SSR) are separate projects sharing `kokoro-shared`. The API has working auth (JWT+Argon2) but no creature endpoints. Leptos frontend has login/register HTML shells with no functionality. The game saves locally to SQLite but has no web connectivity. Phase 11 connects everything: functional web auth, creature sync between game and API, profile page, and Stripe for purchasing etharin crystals.

## Scope — 5 sub-phases, ordered by dependency

---

### 11A: Leptos Auth Wiring (server functions + forms)

**Goal:** Login/Register pages actually work, calling the existing API endpoints.

**Files to edit:**
- `kokoro-web/ui/src/server/mod.rs` — add server functions for login/register/profile
- `kokoro-web/ui/src/pages/login.rs` — wire form to server function, handle response
- `kokoro-web/ui/src/pages/register.rs` — wire form to server function
- `kokoro-web/ui/src/components/nav.rs` — show username when logged in, logout link
- `kokoro-web/ui/src/app.rs` — add `/profile` route + provide auth context

**Files to create:**
- `kokoro-web/ui/src/pages/profile.rs` — profile page showing user info
- `kokoro-web/ui/src/pages/mod.rs` — add profile module

**Approach:**
- Server functions (`#[server]`) call the API at `localhost:8080` using `reqwest`
- Login/register return JWT token, stored in browser cookie (httponly)
- Auth context: `RwSignal<Option<UserSession>>` provided at App level
- Profile page: fetch `/auth/profile` with stored token, display user data
- Nav dynamically shows "Login" or "Profile / Logout" based on auth state
- Add `reqwest` to UI Cargo.toml (ssr feature only)

---

### 11B: Creature Sync — API endpoints + DB schema

**Goal:** API can store and serve creature state (genome + mind stats).

**Files to edit:**
- `kokoro-web/api/src/db/mod.rs` — add creatures table
- `kokoro-web/api/src/constants/routes.rs` — add creature routes
- `kokoro-web/api/src/constants/auth.rs` — add creature route constants
- `kokoro-web/api/src/router.rs` — register creature endpoints
- `kokoro-web/api/src/models/auth.rs` — (no change, reuse auth extractor)

**Files to create:**
- `kokoro-web/api/src/db/creatures.rs` — CRUD for creature records
- `kokoro-web/api/src/models/creature.rs` — CreatureSnapshot, SyncRequest/Response
- `kokoro-web/api/src/controllers/creature.rs` — sync upload/download endpoints
- `kokoro-web/api/src/services/creature.rs` — business logic
- `kokoro-web/api/src/middleware/auth.rs` — proper Axum extractor for authenticated routes

**New DB table:**
```sql
CREATE TABLE IF NOT EXISTS creatures (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id),
    species     TEXT NOT NULL,
    genome_json TEXT NOT NULL,
    mind_json   TEXT NOT NULL,
    age_ticks   INTEGER NOT NULL DEFAULT 0,
    alive       INTEGER NOT NULL DEFAULT 1,
    synced_at   TEXT NOT NULL,
    UNIQUE(user_id)  -- one active creature per user
);
```

**New endpoints (all auth-required):**
| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/creature/sync` | POST | Upload creature state from game |
| `/api/creature` | GET | Download latest creature state |

**Auth middleware:** Axum extractor `AuthUser(Claims)` that validates Bearer token and extracts claims — replaces manual `extract_bearer_token` pattern. Apply to creature routes and profile.

---

### 11C: Game Auth — Bevy HTTP client

**Goal:** Game's auth screen actually calls the API, stores JWT, syncs creature.

**Files to edit:**
- `Cargo.toml` (game) — add `reqwest` (blocking) + `serde_json`
- `src/ui/auth_screen.rs` — wire login/register to API calls
- `src/persistence/plugin.rs` — sync on save if authenticated
- `src/ui/side_menu.rs` — profile tab shows web account info

**Files to create:**
- `src/web/mod.rs` — HTTP client module
- `src/web/auth.rs` — login/register/profile API calls
- `src/web/sync.rs` — creature state upload/download

**Approach:**
- New `WebSession` resource: `Option<{ token: String, user_id: String, email: String }>`
- Auth screen: login/register fields (text input + password), call API via `reqwest::blocking`
- On successful auth → store JWT in WebSession resource → transition to Onboarding
- Guest mode → WebSession stays None → offline play
- Autosave system: if WebSession has token, also POST to `/api/creature/sync`
- On startup: if WebSession token exists, try to download creature from API (merge or skip if local is newer based on age_ticks)

**Note:** `reqwest::blocking` in a Bevy system is acceptable because auth is a one-shot action. For sync, use `IoTaskPool` to avoid blocking the main thread.

---

### 11D: Profile Page with Creature Data (Leptos)

**Goal:** Web profile page shows the user's creature fetched from the API.

**Files to edit:**
- `kokoro-web/ui/src/pages/profile.rs` — add creature section
- `kokoro-web/ui/style/main.css` — profile page styles

**Profile page content:**
- User info (display name, email, member since)
- Creature section: species, age, mood, stats (hunger/happiness/energy)
- Genome visualization (7 gene bars, same style as game side menu)
- "Last synced" timestamp
- If no creature: "Play the game to meet your Kobara!"

---

### 11E: Stripe Integration — Etharin Crystal Shop

**Goal:** Players can buy etharin crystals via Stripe Checkout, spend them in-game shop.

**Files to edit:**
- `kokoro-web/api/Cargo.toml` — add `stripe-rust` crate
- `kokoro-web/api/src/db/mod.rs` — add `wallets` and `purchases` tables
- `kokoro-web/api/src/constants/routes.rs` — add shop routes
- `kokoro-web/api/src/router.rs` — register shop endpoints

**Files to create:**
- `kokoro-web/api/src/db/wallets.rs` — balance CRUD
- `kokoro-web/api/src/db/purchases.rs` — purchase history
- `kokoro-web/api/src/models/shop.rs` — CheckoutRequest, PurchaseRecord, WalletBalance
- `kokoro-web/api/src/controllers/shop.rs` — checkout + webhook handlers
- `kokoro-web/api/src/services/shop.rs` — Stripe session creation, balance management

**New DB tables:**
```sql
CREATE TABLE IF NOT EXISTS wallets (
    user_id  TEXT PRIMARY KEY REFERENCES users(id),
    crystals INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS purchases (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id),
    stripe_session  TEXT NOT NULL,
    crystals        INTEGER NOT NULL,
    amount_cents    INTEGER NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    created_at      TEXT NOT NULL
);
```

**New endpoints:**
| Endpoint | Method | Auth | Purpose |
|----------|--------|------|---------|
| `/api/shop/balance` | GET | Yes | Get user's crystal balance |
| `/api/shop/checkout` | POST | Yes | Create Stripe Checkout session |
| `/api/shop/webhook` | POST | No | Stripe webhook → credit crystals |
| `/api/shop/purchase` | POST | Yes | Spend crystals on item |

**Crystal packs:**
| Pack | Crystals | Price |
|------|----------|-------|
| Starter | 100 | $0.99 |
| Explorer | 500 | $3.99 |
| Resonance | 1200 | $7.99 |

**Flow:**
1. User clicks "Buy Crystals" in game or web
2. API creates Stripe Checkout session → returns URL
3. User completes payment on Stripe hosted page
4. Stripe sends webhook → API credits crystals to wallet
5. Game syncs balance on next API call
6. In-game shop "BUY" button → POST `/api/shop/purchase` → deduct crystals

**Game integration:**
- Side menu Shop tab: show crystal balance (fetched from API if authenticated)
- BUY button: if authenticated, call purchase endpoint; if guest, show "Login to buy"
- New resource: `WalletBalance(u32)` synced from API

---

## Execution Order

1. **11B first** — API creature endpoints + auth middleware (foundation)
2. **11A** — Leptos auth wiring (depends on middleware from 11B)
3. **11C** — Game auth + sync (depends on creature endpoints from 11B)
4. **11D** — Profile page with creature data (depends on 11A + 11B)
5. **11E** — Stripe integration (depends on auth middleware from 11B)

---

## Verification

1. `cargo build -p kokoro-api` — zero warnings
2. `cargo leptos build` (in kokoro-web/ui) — zero warnings
3. `cargo build` (game) — zero warnings
4. Start API → register user → login → get profile → all 200 OK
5. Start game → login screen → enter credentials → JWT stored → Onboarding plays
6. Play game → creature auto-syncs to API
7. Open web profile → see creature data matching game state
8. Click "Buy Crystals" → Stripe Checkout → webhook → balance updated
9. In-game shop → buy item → crystals deducted
