pub mod creatures;
pub mod shop;
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

            CREATE TABLE IF NOT EXISTS creatures (
                id          TEXT PRIMARY KEY,
                user_id     TEXT NOT NULL UNIQUE,
                species     TEXT NOT NULL,
                genome_json TEXT NOT NULL,
                mind_json   TEXT NOT NULL,
                age_ticks   INTEGER NOT NULL DEFAULT 0,
                alive       INTEGER NOT NULL DEFAULT 1,
                synced_at   TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS wallets (
                user_id  TEXT PRIMARY KEY,
                crystals INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS purchases (
                id              TEXT PRIMARY KEY,
                user_id         TEXT NOT NULL,
                stripe_session  TEXT NOT NULL,
                crystals        INTEGER NOT NULL,
                amount_cents    INTEGER NOT NULL,
                status          TEXT NOT NULL DEFAULT 'pending',
                created_at      TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            );
        ").expect("Failed to create tables");

        Self { conn: Mutex::new(conn) }
    }
}