//! Shop controller — Stripe checkout, webhook, balance, and item purchases.

use std::sync::Arc;
use axum::{extract::State, Json};

use crate::db::Database;
use crate::error::ApiError;
use crate::middleware::auth::AuthUser;
use crate::models::shop::*;
use crate::services::shop as shop_service;

/// GET /api/shop/balance — get the user's crystal balance.
pub async fn balance(
    AuthUser(claims): AuthUser,
    State(db): State<Arc<Database>>,
) -> Result<Json<BalanceResponse>, ApiError> {
    let crystals = crate::db::shop::get_balance(&db, &claims.sub)
        .map_err(|e| ApiError::internal(e))?;

    Ok(Json(BalanceResponse { crystals }))
}

/// POST /api/shop/checkout — create a Stripe Checkout session.
pub async fn checkout(
    AuthUser(claims): AuthUser,
    State(db): State<Arc<Database>>,
    Json(body): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>, ApiError> {
    let pack = PACKS.iter()
        .find(|p| p.id == body.pack_id)
        .ok_or_else(|| ApiError::not_found(format!("Pack '{}' not found", body.pack_id)))?;

    let stripe_key = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| ApiError::internal("STRIPE_SECRET_KEY not set"))?;

    let client = stripe::Client::new(&stripe_key);

    let success_url = std::env::var("SHOP_SUCCESS_URL")
        .unwrap_or_else(|_| "http://localhost:3000/profile?purchase=success".to_string());
    let cancel_url = std::env::var("SHOP_CANCEL_URL")
        .unwrap_or_else(|_| "http://localhost:3000/profile?purchase=cancelled".to_string());

    let (checkout_url, session_id) = shop_service::create_checkout(
        &client, &db, &claims.sub, pack, &success_url, &cancel_url,
    ).await.map_err(|e| ApiError::internal(e))?;

    Ok(Json(CheckoutResponse { checkout_url, session_id }))
}

/// POST /api/shop/webhook — Stripe webhook to fulfill purchases.
///
/// This endpoint is called by Stripe, NOT by the user. No auth required.
/// In production, verify the Stripe signature header.
pub async fn webhook(
    State(db): State<Arc<Database>>,
    body: String,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Parse the Stripe event
    let event: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| ApiError::internal(format!("Invalid webhook payload: {e}")))?;

    let event_type = event["type"].as_str().unwrap_or("");

    if event_type == "checkout.session.completed" {
        let session_id = event["data"]["object"]["id"].as_str()
            .ok_or_else(|| ApiError::internal("Missing session ID in webhook"))?;

        shop_service::fulfill_purchase(&db, session_id)
            .map_err(|e| ApiError::internal(e))?;

        println!("Purchase fulfilled for session {session_id}");
    }

    Ok(Json(serde_json::json!({"received": true})))
}

/// POST /api/shop/purchase — spend crystals on an in-game item.
pub async fn purchase_item(
    AuthUser(claims): AuthUser,
    State(db): State<Arc<Database>>,
    Json(body): Json<PurchaseItemRequest>,
) -> Result<Json<PurchaseItemResponse>, ApiError> {
    let (spent, remaining) = shop_service::purchase_item(&db, &claims.sub, &body.item_id)
        .map_err(|e| ApiError::internal(e))?;

    Ok(Json(PurchaseItemResponse {
        item_id: body.item_id,
        crystals_spent: spent,
        remaining_balance: remaining,
    }))
}

/// POST /api/donate — Create a Stripe Checkout for a one-time donation ("buy me a coffee").
///
/// No auth required — anyone can donate.
pub async fn donate_checkout(
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let amount_cents = body["amount_cents"].as_i64()
        .ok_or_else(|| ApiError::internal("Missing amount_cents"))?;

    if amount_cents < 100 || amount_cents > 50000 {
        return Err(ApiError::internal("Amount must be between $1.00 and $500.00"));
    }

    let stripe_key = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| ApiError::internal("STRIPE_SECRET_KEY not set"))?;

    let client = stripe::Client::new(&stripe_key);

    let success_url = std::env::var("DONATE_SUCCESS_URL")
        .unwrap_or_else(|_| "http://localhost:3000/?donated=true".to_string());
    let cancel_url = std::env::var("DONATE_CANCEL_URL")
        .unwrap_or_else(|_| "http://localhost:3000/".to_string());

    let mut params = stripe::CreateCheckoutSession::new();
    params.mode = Some(stripe::CheckoutSessionMode::Payment);
    params.success_url = Some(&success_url);
    params.cancel_url = Some(&cancel_url);
    params.line_items = Some(vec![stripe::CreateCheckoutSessionLineItems {
        quantity: Some(1),
        price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
            currency: stripe::Currency::USD,
            unit_amount: Some(amount_cents),
            product_data: Some(stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                name: "Support Kokoro Development".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }]);

    let session = stripe::CheckoutSession::create(&client, params)
        .await
        .map_err(|e| ApiError::internal(format!("Stripe error: {e}")))?;

    let checkout_url = session.url
        .ok_or_else(|| ApiError::internal("No checkout URL"))?;

    Ok(Json(serde_json::json!({
        "checkout_url": checkout_url,
        "amount_cents": amount_cents,
    })))
}
