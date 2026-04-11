//! Kokoro API — companion web server for the Kokoro game.
//!
//! Layered architecture:
//!   Router → Controllers → Services → kokoro-shared
//!
//! All game data comes from kokoro-shared (single source of truth).
//! User data is stored in a local SQLite database.

mod constants;
mod controllers;
mod db;
mod error;
mod middleware;
mod models;
mod router;
mod services;

use std::sync::Arc;
use constants::SERVER_ADDR;

#[tokio::main]
async fn main() {
    // Load .env file (if present — not required)
    let _ = dotenvy::dotenv();

    // Initialize the user database (creates table if needed)
    let db = Arc::new(db::Database::new("kokoro-users.db"));

    let app = router::create_router(db);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDR)
        .await
        .expect("Failed to bind to server address");

    println!("Kokoro API running on http://{SERVER_ADDR}");
    axum::serve(listener, app).await.unwrap();
}
