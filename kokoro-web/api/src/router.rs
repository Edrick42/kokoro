//! Router — maps URL paths to controller handlers.
//! All route definitions live here in one place.

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};

use crate::constants::routes;
use crate::controllers;

/// Creates the application router with all routes and middleware.
pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route(routes::HEALTH, get(controllers::health::check))
        .route(routes::SPECIES_LIST, get(controllers::species::list))
        .route(routes::SPECIES_BY_NAME, get(controllers::species::get_by_name))
        .route(routes::BIOME_LIST, get(controllers::biomes::list))
        .route(routes::BIOME_BY_NAME, get(controllers::biomes::get_by_name))
        .route(routes::FOOD_LIST, get(controllers::foods::list))
        .layer(cors)
}
