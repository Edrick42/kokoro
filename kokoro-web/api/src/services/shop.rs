//! Shop service — Stripe Checkout sessions and crystal management.
//!
//! No HTTP types here — pure business logic.

use stripe::{
    Client, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CreateCheckoutSessionLineItemsPriceData,
    CreateCheckoutSessionLineItemsPriceDataProductData,
    CheckoutSession, CheckoutSessionMode, Currency,
};

use crate::models::shop::{CrystalPack, PurchaseRecord};
use crate::db::{self, Database};

/// Creates a Stripe Checkout session for a crystal pack.
pub async fn create_checkout(
    stripe_client: &Client,
    db: &Database,
    user_id: &str,
    pack: &CrystalPack,
    success_url: &str,
    cancel_url: &str,
) -> Result<(String, String), String> {
    let mut params = CreateCheckoutSession::new();
    params.mode = Some(CheckoutSessionMode::Payment);
    params.success_url = Some(success_url);
    params.cancel_url = Some(cancel_url);
    params.client_reference_id = Some(user_id);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        quantity: Some(1),
        price_data: Some(CreateCheckoutSessionLineItemsPriceData {
            currency: Currency::USD,
            unit_amount: Some(pack.price_cents),
            product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                name: format!("{} — {} Etharin Crystals", pack.name, pack.crystals),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }]);

    let session = CheckoutSession::create(stripe_client, params)
        .await
        .map_err(|e| format!("Stripe error: {e}"))?;

    let session_id = session.id.to_string();
    let checkout_url = session.url
        .ok_or_else(|| "No checkout URL returned".to_string())?;

    // Record the pending purchase
    let record = PurchaseRecord {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        stripe_session: session_id.clone(),
        crystals: pack.crystals,
        amount_cents: pack.price_cents,
        status: "pending".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    db::shop::create_purchase(db, &record)?;

    Ok((checkout_url, session_id))
}

/// Fulfills a purchase after Stripe webhook confirms payment.
pub fn fulfill_purchase(db: &Database, stripe_session: &str) -> Result<(), String> {
    let (user_id, crystals) = db::shop::find_purchase_by_session(db, stripe_session)
        .ok_or_else(|| "Purchase not found or already fulfilled".to_string())?;

    db::shop::credit_crystals(db, &user_id, crystals)?;
    db::shop::update_purchase_status(db, stripe_session, "completed")?;

    Ok(())
}

/// Spends crystals on an in-game item.
pub fn purchase_item(db: &Database, user_id: &str, item_id: &str) -> Result<(i64, i64), String> {
    // Find the item price from config
    let item = kokoro_shared::shop_items()
        .iter()
        .find(|i| i.id == item_id)
        .ok_or_else(|| format!("Item '{}' not found", item_id))?;

    let price = item.price as i64;

    db::shop::debit_crystals(db, user_id, price)?;
    let remaining = db::shop::get_balance(db, user_id)?;

    Ok((price, remaining))
}
