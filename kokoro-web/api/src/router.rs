//! Router — maps URL paths to controller handlers.
//! All route definitions live here in one place.

use std::sync::Arc;
use axum::{routing::{get, post}, Router};
use tower_http::cors::{Any, CorsLayer};

use crate::constants::{routes, auth};
use crate::controllers;
use crate::db::Database;

/// Creates the application router with all routes and middleware.
///
/// The `db` parameter is shared across all handlers via Axum's
/// State extractor. `Arc` provides cheap cloning for thread safety.
pub fn create_router(db: Arc<Database>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Public endpoints
        .route(routes::HEALTH, get(controllers::health::check))
        .route(routes::SPECIES_LIST, get(controllers::species::list))
        .route(routes::SPECIES_BY_NAME, get(controllers::species::get_by_name))
        .route(routes::BIOME_LIST, get(controllers::biomes::list))
        .route(routes::BIOME_BY_NAME, get(controllers::biomes::get_by_name))
        .route(routes::FOOD_LIST, get(controllers::foods::list))
        // Auth endpoints
        .route(auth::routes::REGISTER, post(controllers::auth::register))
        .route(auth::routes::LOGIN, post(controllers::auth::login))
        .route(auth::routes::PROFILE, get(controllers::auth::profile))
        // Shared state + middleware
        .with_state(db)
        .layer(cors)
}
