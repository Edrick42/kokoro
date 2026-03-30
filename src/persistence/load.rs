//! Load functions — read a saved Kobara state from the database.

use rusqlite::{Connection, Result};

use crate::genome::{Genome, Species};
use crate::mind::{Mind, MoodState, VitalStats};

/// Attempts to load a previously saved Kobara.
///
/// Returns `Ok(Some((genome, mind)))` if a save exists,
/// or `Ok(None)` if the database is empty (first run).
pub fn load_saved(conn: &Connection) -> Result<Option<(Genome, Mind)>> {
    let genome = match load_genome(conn)? {
        Some(g) => g,
        None    => return Ok(None), // No save found — first run
    };

    let mind = load_mind(conn)?.unwrap_or_else(Mind::new);

    Ok(Some((genome, mind)))
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

fn load_mind(conn: &Connection) -> Result<Option<Mind>> {
    let mut stmt = conn.prepare(
        "SELECT hunger, happiness, energy, health, mood, age_ticks
         FROM creature WHERE id = 1",
    )?;

    let result = stmt.query_row([], |row| {
        let mood_str: String = row.get(4)?;
        Ok(Mind {
            stats: VitalStats {
                hunger:    row.get(0)?,
                happiness: row.get(1)?,
                energy:    row.get(2)?,
                health:    row.get(3)?,
            },
            mood:      str_to_mood(&mood_str),
            age_ticks: row.get(5)?,
        })
    });

    match result {
        Ok(mind)                                => Ok(Some(mind)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e)                                  => Err(e),
    }
}

fn str_to_species(s: &str) -> Species {
    match s {
        "Moluun"  => Species::Moluun,
        "Pylum" => Species::Pylum,
        "Skael"   => Species::Skael,
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
