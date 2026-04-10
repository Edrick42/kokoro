//! User repository — SQLite operations for the users table.

use crate::models::auth::UserRecord;
use super::Database;

/// Inserts a new user. Returns Err if the email already exists.
pub fn create_user(db: &Database, user: &UserRecord) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO users (id, email, display_name, password_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (&user.id, &user.email, &user.display_name, &user.password_hash, &user.created_at),
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// Finds a user by email. Returns None if not found.
pub fn find_by_email(db: &Database, email: &str) -> Option<UserRecord> {
    let conn = db.conn.lock().ok()?;
    let mut stmt = conn.prepare(
        "SELECT id, email, display_name, password_hash, created_at
         FROM users WHERE email = ?1"
    ).ok()?;

    stmt.query_row([email], |row| {
        Ok(UserRecord {
            id:            row.get(0)?,
            email:         row.get(1)?,
            display_name:  row.get(2)?,
            password_hash: row.get(3)?,
            created_at:    row.get(4)?,
        })
    }).ok()
}

/// Finds a user by ID. Returns None if not found.
pub fn find_by_id(db: &Database, id: &str) -> Option<UserRecord> {
    let conn = db.conn.lock().ok()?;
    let mut stmt = conn.prepare(
        "SELECT id, email, display_name, password_hash, created_at
         FROM users WHERE id = ?1"
    ).ok()?;

    stmt.query_row([id], |row| {
        Ok(UserRecord {
            id:            row.get(0)?,
            email:         row.get(1)?,
            display_name:  row.get(2)?,
            password_hash: row.get(3)?,
            created_at:    row.get(4)?,
        })
    }).ok()
}
