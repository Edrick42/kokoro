//! Kokoro API — companion web server for the Kokoro game.
//!
//! Layered architecture:
//!   Router → Controllers → Services → kokoro-shared
//!
//! All data comes from kokoro-shared (single source of truth).

mod constants;
mod controllers;
mod error;
mod models;
mod router;
mod services;

use constants::SERVER_ADDR;

#[tokio::main]
async fn main() {
    let app = router::create_router();

    let listener = tokio::net::TcpListener::bind(SERVER_ADDR)
        .await
        .expect("Failed to bind to server address");

    println!("Kokoro API running on http://{SERVER_ADDR}");
    axum::serve(listener, app).await.unwrap();
}
