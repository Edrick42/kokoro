//! Shop repository — wallets and purchases.

use crate::models::shop::PurchaseRecord;
use super::Database;

/// Gets the user's crystal balance, creating a wallet if needed.
pub fn get_balance(db: &Database, user_id: &str) -> Result<i64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Ensure wallet exists
    conn.execute(
        "INSERT OR IGNORE INTO wallets (user_id, crystals) VALUES (?1, 0)",
        [user_id],
    ).map_err(|e| e.to_string())?;

    let balance: i64 = conn.query_row(
        "SELECT crystals FROM wallets WHERE user_id = ?1",
        [user_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    Ok(balance)
}

/// Adds crystals to a user's wallet.
pub fn credit_crystals(db: &Database, user_id: &str, amount: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO wallets (user_id, crystals) VALUES (?1, ?2)
         ON CONFLICT(user_id) DO UPDATE SET crystals = crystals + ?2",
        rusqlite::params![user_id, amount],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// Deducts crystals from a user's wallet. Returns error if insufficient funds.
pub fn debit_crystals(db: &Database, user_id: &str, amount: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let current: i64 = conn.query_row(
        "SELECT crystals FROM wallets WHERE user_id = ?1",
        [user_id],
        |row| row.get(0),
    ).unwrap_or(0);

    if current < amount {
        return Err(format!("Insufficient crystals: have {current}, need {amount}"));
    }

    conn.execute(
        "UPDATE wallets SET crystals = crystals - ?1 WHERE user_id = ?2",
        rusqlite::params![amount, user_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// Records a purchase.
pub fn create_purchase(db: &Database, record: &PurchaseRecord) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO purchases (id, user_id, stripe_session, crystals, amount_cents, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            record.id,
            record.user_id,
            record.stripe_session,
            record.crystals,
            record.amount_cents,
            record.status,
            record.created_at,
        ],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// Updates a purchase status (e.g., pending → completed).
pub fn update_purchase_status(db: &Database, stripe_session: &str, status: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE purchases SET status = ?1 WHERE stripe_session = ?2",
        [status, stripe_session],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// Finds the user_id and crystals for a purchase by Stripe session ID.
pub fn find_purchase_by_session(db: &Database, stripe_session: &str) -> Option<(String, i64)> {
    let conn = db.conn.lock().ok()?;
    conn.query_row(
        "SELECT user_id, crystals FROM purchases WHERE stripe_session = ?1 AND status = 'pending'",
        [stripe_session],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
    ).ok()
}
