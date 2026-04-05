// Kokoro API — companion web server for the Kokoro game.
//
// Endpoints:
//   GET /health              — server status
//   GET /api/species         — list all species
//   GET /api/species/{name}  — species details
//   GET /api/biome/{name}    — biome details

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct Health {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct SpeciesSummary {
    name: String,
    biome: String,
}

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

#[derive(Serialize)]
struct BiomeInfo {
    name: String,
    description: String,
    species: String,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
    code: u16,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status =
            StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

async fn list_species() -> Json<Vec<SpeciesSummary>> {
    Json(vec![
        SpeciesSummary { name: "Moluun".into(), biome: "The Verdance".into() },
        SpeciesSummary { name: "Pylum".into(),  biome: "Veridian Highlands".into() },
        SpeciesSummary { name: "Skael".into(),  biome: "Abyssal Shallows".into() },
        SpeciesSummary { name: "Nyxal".into(),  biome: "Abyssal Depths".into() },
    ])
}

async fn get_species(Path(name): Path<String>) -> Result<Json<SpeciesInfo>, ApiError> {
    let info = match name.to_lowercase().as_str() {
        "moluun" => SpeciesInfo {
            name: "Moluun".into(),
            classification: "K. moluunaris".into(),
            biome: "The Verdance (forests)".into(),
            description: "Round, soft, forest-dwelling Kobara. Most social manifestation.".into(),
            gene_ranges: GeneRanges { curiosity: (0.2, 1.0), appetite: (0.1, 0.8), resilience: (0.2, 1.0) },
        },
        "pylum" => SpeciesInfo {
            name: "Pylum".into(),
            classification: "K. pylumensis".into(),
            biome: "Veridian Highlands".into(),
            description: "Winged, curious Kobara. Reflects the seeking spirit.".into(),
            gene_ranges: GeneRanges { curiosity: (0.4, 1.0), appetite: (0.1, 0.5), resilience: (0.3, 0.9) },
        },
        "skael" => SpeciesInfo {
            name: "Skael".into(),
            classification: "K. skaelith".into(),
            biome: "Abyssal Shallows (caves)".into(),
            description: "Scaled, resilient Kobara. Quiet strength.".into(),
            gene_ranges: GeneRanges { curiosity: (0.1, 0.7), appetite: (0.3, 1.0), resilience: (0.5, 1.0) },
        },
        "nyxal" => SpeciesInfo {
            name: "Nyxal".into(),
            classification: "K. nyxalaris".into(),
            biome: "Abyssal Depths (deep ocean)".into(),
            description: "Tentacled, intelligent Kobara. Mirrors your thinking.".into(),
            gene_ranges: GeneRanges { curiosity: (0.5, 1.0), appetite: (0.2, 0.7), resilience: (0.1, 0.6) },
        },
        _ => return Err(ApiError {
            error: format!("Species '{}' not found", name),
            code: 404,
        }),
    };
    Ok(Json(info))
}

async fn get_biome(Path(name): Path<String>) -> Result<Json<BiomeInfo>, ApiError> {
    let info = match name.to_lowercase().as_str() {
        "verdance" => BiomeInfo {
            name: "The Verdance".into(),
            description: "Vast bioluminescent forests with spiral-formation trees and luminescent spores.".into(),
            species: "Moluun".into(),
        },
        "highlands" => BiomeInfo {
            name: "Veridian Highlands".into(),
            description: "Towering mesa formations with thermal updrafts and floating mineral deposits.".into(),
            species: "Pylum".into(),
        },
        "shallows" => BiomeInfo {
            name: "Abyssal Shallows".into(),
            description: "Crystalline cave network with underground rivers and bioluminescent walls.".into(),
            species: "Skael".into(),
        },
        "depths" => BiomeInfo {
            name: "Abyssal Depths".into(),
            description: "Vast subterranean ocean. Lightless, pressurized, etharin-saturated.".into(),
            species: "Nyxal".into(),
        },
        _ => return Err(ApiError {
            error: format!("Biome '{}' not found", name),
            code: 404,
        }),
    };
    Ok(Json(info))
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/species", get(list_species))
        .route("/api/species/{name}", get(get_species))
        .route("/api/biome/{name}", get(get_biome))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("Kokoro API running on http://localhost:3000");
    println!("  GET /health");
    println!("  GET /api/species");
    println!("  GET /api/species/{{name}}");
    println!("  GET /api/biome/{{name}}");
    axum::serve(listener, app).await.unwrap();
}
