//! Router — maps URL paths to controller handlers.
//! All route definitions live here in one place.

use std::sync::Arc;
use axum::{routing::{get, post, delete}, Router};
use axum::http::{HeaderValue, header};
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::constants::{routes, auth};
use crate::controllers;
use crate::db::Database;

/// Creates the application router with all routes and middleware.
pub fn create_router(db: Arc<Database>) -> Router {
    // CORS — restrictive in production, permissive in dev
    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let origins: Vec<HeaderValue> = allowed_origins.split(',')
        .filter_map(|o| o.trim().parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

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
        // Creature endpoints (auth required via AuthUser extractor)
        .route(routes::CREATURE_GET, get(controllers::creature::get))
        .route(routes::CREATURE_SYNC, post(controllers::creature::sync))
        // Shop endpoints
        .route(routes::SHOP_BALANCE, get(controllers::shop::balance))
        .route(routes::SHOP_CHECKOUT, post(controllers::shop::checkout))
        .route(routes::SHOP_WEBHOOK, post(controllers::shop::webhook))
        .route(routes::SHOP_PURCHASE, post(controllers::shop::purchase_item))
        // Privacy endpoints (LGPD/GDPR compliance)
        .route(routes::USER_DATA_EXPORT, get(controllers::auth::data_export))
        .route(routes::USER_DELETE_ACCOUNT, delete(controllers::auth::delete_account))
        // Donation endpoint
        .route(routes::DONATE_CHECKOUT, post(controllers::shop::donate_checkout))
        // Shared state + middleware
        .with_state(db)
        .layer(cors)
        // Security headers
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
}
