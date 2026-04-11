//! Shop models — checkout, purchases, wallet.

use serde::{Deserialize, Serialize};

/// Crystal pack options available for purchase.
#[derive(Clone)]
pub struct CrystalPack {
    pub id: &'static str,
    pub name: &'static str,
    pub crystals: i64,
    pub price_cents: i64,
}

/// Available crystal packs.
pub const PACKS: &[CrystalPack] = &[
    CrystalPack { id: "starter",   name: "Starter Pack",   crystals: 100,  price_cents: 99 },
    CrystalPack { id: "explorer",  name: "Explorer Pack",  crystals: 500,  price_cents: 399 },
    CrystalPack { id: "resonance", name: "Resonance Pack", crystals: 1200, price_cents: 799 },
];

/// Request to create a Stripe Checkout session.
#[derive(Deserialize)]
pub struct CheckoutRequest {
    pub pack_id: String,
}

/// Response with the Stripe Checkout URL.
#[derive(Serialize)]
pub struct CheckoutResponse {
    pub checkout_url: String,
    pub session_id: String,
}

/// Request to spend crystals on an in-game item.
#[derive(Deserialize)]
pub struct PurchaseItemRequest {
    pub item_id: String,
}

/// Response after spending crystals.
#[derive(Serialize)]
pub struct PurchaseItemResponse {
    pub item_id: String,
    pub crystals_spent: i64,
    pub remaining_balance: i64,
}

/// Wallet balance response.
#[derive(Serialize)]
pub struct BalanceResponse {
    pub crystals: i64,
}

/// Internal purchase record.
pub struct PurchaseRecord {
    pub id: String,
    pub user_id: String,
    pub stripe_session: String,
    pub crystals: i64,
    pub amount_cents: i64,
    pub status: String,
    pub created_at: String,
}
