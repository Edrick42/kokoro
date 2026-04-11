//! Creature repository — SQLite operations for the creatures table.

use crate::models::creature::CreatureRecord;
use super::Database;

/// Upserts a creature for the given user (one active creature per user).
pub fn upsert(db: &Database, record: &CreatureRecord) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO creatures (id, user_id, species, genome_json, mind_json, age_ticks, alive, synced_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(user_id) DO UPDATE SET
             species = excluded.species,
             genome_json = excluded.genome_json,
             mind_json = excluded.mind_json,
             age_ticks = excluded.age_ticks,
             alive = excluded.alive,
             synced_at = excluded.synced_at",
        (
            &record.id,
            &record.user_id,
            &record.species,
            &record.genome_json,
            &record.mind_json,
            record.age_ticks,
            record.alive as i32,
            &record.synced_at,
        ),
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// Finds the active creature for a user. Returns None if no creature synced.
pub fn find_by_user(db: &Database, user_id: &str) -> Option<CreatureRecord> {
    let conn = db.conn.lock().ok()?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, species, genome_json, mind_json, age_ticks, alive, synced_at
         FROM creatures WHERE user_id = ?1"
    ).ok()?;

    stmt.query_row([user_id], |row| {
        Ok(CreatureRecord {
            id:         row.get(0)?,
            user_id:    row.get(1)?,
            species:    row.get(2)?,
            genome_json: row.get(3)?,
            mind_json:  row.get(4)?,
            age_ticks:  row.get(5)?,
            alive:      row.get::<_, i32>(6)? != 0,
            synced_at:  row.get(7)?,
        })
    }).ok()
}
