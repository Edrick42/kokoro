//! Foods controller — handles HTTP for food endpoints.

use axum::Json;
use crate::models::foods::FoodResponse;
use crate::services;

/// GET /api/foods — returns all available food items.
pub async fn list() -> Json<Vec<FoodResponse>> {
    Json(services::foods::list_all())
}
