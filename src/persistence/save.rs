//! Save functions — write the current Kobara state to the database.

use rusqlite::{Connection, Result, params};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::genome::Genome;
use crate::mind::{Mind, MoodState};

/// Persists the full creature state (genome + mind) to the database.
///
/// - Genome is inserted with `INSERT OR IGNORE` — it is written once and never changed.
/// - Creature stats use `INSERT OR REPLACE` — always reflects the latest state.
pub fn save_all(conn: &Connection, genome: &Genome, mind: &Mind) -> Result<()> {
    save_genome(conn, genome)?;
    save_mind(conn, mind)?;
    Ok(())
}

/// Writes the genome. Uses INSERT OR IGNORE so the original DNA is never overwritten.
fn save_genome(conn: &Connection, genome: &Genome) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO genome
         (id, species, curiosity, loneliness_sensitivity, appetite,
          circadian, resilience, learning_rate, hue)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            format!("{:?}", genome.species),
            genome.curiosity,
            genome.loneliness_sensitivity,
            genome.appetite,
            genome.circadian,
            genome.resilience,
            genome.learning_rate,
            genome.hue,
        ],
    )?;
    Ok(())
}

/// Writes the current vital stats and mood. Replaces any previous row.
fn save_mind(conn: &Connection, mind: &Mind) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO creature
         (id, hunger, happiness, energy, health, mood, age_ticks)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            mind.stats.hunger,
            mind.stats.happiness,
            mind.stats.energy,
            mind.stats.health,
            mood_to_str(&mind.mood),
            mind.age_ticks,
        ],
    )?;
    Ok(())
}

/// Appends a single event to the event log.
///
/// `event_type` examples: `"fed"`, `"played"`, `"mood_changed"`, `"slept"`
/// `payload` is an optional JSON string for extra context.
pub fn log_event(
    conn: &Connection,
    tick: u64,
    event_type: &str,
    payload: Option<&str>,
) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    conn.execute(
        "INSERT INTO event_log (tick, event_type, payload, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![tick, event_type, payload, now],
    )?;
    Ok(())
}

fn mood_to_str(mood: &MoodState) -> &'static str {
    match mood {
        MoodState::Happy    => "Happy",
        MoodState::Hungry   => "Hungry",
        MoodState::Tired    => "Tired",
        MoodState::Lonely   => "Lonely",
        MoodState::Playful  => "Playful",
        MoodState::Sick     => "Sick",
        MoodState::Sleeping => "Sleeping",
    }
}
