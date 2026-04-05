// Kokoro API — Chapter 33 of the book will guide you through building this.
//
// Start here: `cargo run` should show "Kokoro API running on :3000"
// Then follow the book to add routes one by one.

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
