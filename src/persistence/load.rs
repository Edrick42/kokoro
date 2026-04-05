//! Load functions — read a saved Kobara state from the database.

use rusqlite::{Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::genome::{Genome, Species};
use crate::mind::{Mind, MoodState, VitalStats};

/// Result of loading a saved creature, including absence duration.
pub struct LoadResult {
    pub genome: Genome,
    pub mind: Mind,
    /// How many seconds the player was absent since the last session ended.
    pub absence_secs: u64,
}

/// Attempts to load a previously saved Kobara.
///
/// Returns `Ok(Some(LoadResult))` if a save exists,
/// or `Ok(None)` if the database is empty (first run).
pub fn load_saved(conn: &Connection) -> Result<Option<LoadResult>> {
    let genome = match load_genome(conn)? {
        Some(g) => g,
        None    => return Ok(None),
    };

    let (mind, last_session_end) = load_mind(conn)?
        .unwrap_or_else(|| (Mind::new(), 0));

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let absence_secs = if last_session_end > 0 {
        (now - last_session_end).max(0) as u64
    } else {
        0
    };

    Ok(Some(LoadResult { genome, mind, absence_secs }))
}

fn load_genome(conn: &Connection) -> Result<Option<Genome>> {
    let mut stmt = conn.prepare(
        "SELECT species, curiosity, loneliness_sensitivity, appetite,
                circadian, resilience, learning_rate, hue
         FROM genome WHERE id = 1",
    )?;

    let result = stmt.query_row([], |row| {
        let species_str: String = row.get(0)?;
        Ok(Genome {
            species:                str_to_species(&species_str),
            curiosity:              row.get(1)?,
            loneliness_sensitivity: row.get(2)?,
            appetite:               row.get(3)?,
            circadian:              row.get(4)?,
            resilience:             row.get(5)?,
            learning_rate:          row.get(6)?,
            hue:                    row.get(7)?,
        })
    });

    match result {
        Ok(genome)                              => Ok(Some(genome)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e)                                  => Err(e),
    }
}

fn load_mind(conn: &Connection) -> Result<Option<(Mind, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT hunger, happiness, energy, health, mood, age_ticks, last_session_end
         FROM creature WHERE id = 1",
    )?;

    let result = stmt.query_row([], |row| {
        let mood_str: String = row.get(4)?;
        let last_session_end: i64 = row.get(6)?;
        Ok((
            Mind {
                stats: VitalStats {
                    hunger:    row.get(0)?,
                    happiness: row.get(1)?,
                    energy:    row.get(2)?,
                    health:    row.get(3)?,
                },
                mood:      str_to_mood(&mood_str),
                age_ticks: row.get(5)?,
                pending_hunger: 0.0,
                pending_happiness: 0.0,
                pending_energy: 0.0,
                mood_cooldown: 0,
            },
            last_session_end,
        ))
    });

    match result {
        Ok(pair)                                => Ok(Some(pair)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e)                                  => Err(e),
    }
}

/// Loads the full creature collection from the `creatures` table.
/// Returns None if the table is empty (first run or pre-migration).
pub fn load_collection(conn: &Connection) -> Result<Option<Vec<crate::creature::collection::StoredCreature>>> {
    let mut stmt = conn.prepare(
        "SELECT slot, name, species, curiosity, loneliness_sensitivity, appetite,
                circadian, resilience, learning_rate, hue,
                hunger, happiness, energy, health, mood, age_ticks
         FROM creatures ORDER BY slot ASC",
    )?;

    let creatures: Vec<crate::creature::collection::StoredCreature> = stmt.query_map([], |row| {
        let species_str: String = row.get(2)?;
        let mood_str: String = row.get(14)?;
        Ok(crate::creature::collection::StoredCreature {
            name: row.get(1)?,
            egg: crate::creature::egg::EggData { progress: 1.0, hatched: true },
            genome: Genome {
                species: str_to_species(&species_str),
                curiosity: row.get(3)?,
                loneliness_sensitivity: row.get(4)?,
                appetite: row.get(5)?,
                circadian: row.get(6)?,
                resilience: row.get(7)?,
                learning_rate: row.get(8)?,
                hue: row.get(9)?,
            },
            mind: Mind {
                stats: VitalStats {
                    hunger: row.get(10)?,
                    happiness: row.get(11)?,
                    energy: row.get(12)?,
                    health: row.get(13)?,
                },
                mood: str_to_mood(&mood_str),
                age_ticks: row.get(15)?,
                pending_hunger: 0.0,
                pending_happiness: 0.0,
                pending_energy: 0.0,
                mood_cooldown: 0,
            },
        })
    })?.filter_map(|r| r.ok()).collect();

    if creatures.is_empty() {
        Ok(None)
    } else {
        Ok(Some(creatures))
    }
}

fn str_to_species(s: &str) -> Species {
    match s {
        "Moluun"  => Species::Moluun,
        "Pylum" => Species::Pylum,
        "Skael"   => Species::Skael,
        "Nyxal"   => Species::Nyxal,
        _         => Species::Moluun,
    }
}

fn str_to_mood(s: &str) -> MoodState {
    match s {
        "Hungry"   => MoodState::Hungry,
        "Tired"    => MoodState::Tired,
        "Lonely"   => MoodState::Lonely,
        "Playful"  => MoodState::Playful,
        "Sick"     => MoodState::Sick,
        "Sleeping" => MoodState::Sleeping,
        _          => MoodState::Happy,
    }
}
