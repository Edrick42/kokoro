pub mod users;

use std::sync::Mutex;
use rusqlite::Connection;

/// Shared database connection wrapped for thread safety.
pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// Opens the database and runs migrations.
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(path)
            .expect("Failed to open user database");

        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS users (
                id           TEXT PRIMARY KEY,
                email        TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                password_hash TEXT NOT NULL,
                created_at   TEXT NOT NULL
            );
        ").expect("Failed to create users table");

        Self { conn: Mutex::new(conn) }
    }
}