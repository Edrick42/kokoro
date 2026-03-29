//! Database connection management and schema initialisation.

use rusqlite::{Connection, Result};
use std::path::PathBuf;

/// Returns the platform-appropriate path for the save database.
///
/// - Linux/macOS: `~/.config/kokoro/save.db`
/// - Windows:     `%APPDATA%\kokoro\save.db`
pub fn db_path() -> PathBuf {
    let base = dirs_next();
    base.join("kokoro").join("save.db")
}

/// Opens (or creates) the database and runs all migrations.
/// Returns a ready-to-use connection.
pub fn open() -> Result<Connection> {
    let path = db_path();

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .expect("Failed to create config directory");
    }

    let conn = Connection::open(&path)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Runs all schema migrations in order.
/// Safe to call on an existing database — uses IF NOT EXISTS throughout.
fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        -- Vital stats and current mood state
        CREATE TABLE IF NOT EXISTS creature (
            id          INTEGER PRIMARY KEY CHECK (id = 1),
            hunger      REAL    NOT NULL DEFAULT 30.0,
            happiness   REAL    NOT NULL DEFAULT 70.0,
            energy      REAL    NOT NULL DEFAULT 80.0,
            health      REAL    NOT NULL DEFAULT 100.0,
            mood        TEXT    NOT NULL DEFAULT 'Happy',
            age_ticks   INTEGER NOT NULL DEFAULT 0
        );

        -- Genetic blueprint — written once, never overwritten
        CREATE TABLE IF NOT EXISTS genome (
            id                      INTEGER PRIMARY KEY CHECK (id = 1),
            species                 TEXT    NOT NULL DEFAULT 'Kobara',
            curiosity               REAL    NOT NULL,
            loneliness_sensitivity  REAL    NOT NULL,
            appetite                REAL    NOT NULL,
            circadian               REAL    NOT NULL,
            resilience              REAL    NOT NULL,
            learning_rate           REAL    NOT NULL,
            hue                     REAL    NOT NULL
        );

        -- Append-only event log — training data for Phase 4 neural net
        CREATE TABLE IF NOT EXISTS event_log (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            tick        INTEGER NOT NULL,
            event_type  TEXT    NOT NULL,  -- 'fed', 'played', 'mood_changed', 'slept'
            payload     TEXT,              -- JSON blob for extra context
            created_at  INTEGER NOT NULL   -- Unix timestamp
        );
    ")?;

    Ok(())
}

/// Returns the config base directory for the current platform.
fn dirs_next() -> PathBuf {
    // std::env::var is used here to avoid a dependency on the `dirs` crate.
    // For a production build, replacing this with `dirs::config_dir()` is recommended.
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| ".".into()))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config")
    }
}
