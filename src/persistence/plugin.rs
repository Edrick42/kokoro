//! Bevy plugin that wires persistence into the game lifecycle.
//!
//! ## What this plugin does
//!
//! - **On startup**: opens the database, loads a previous save if one exists,
//!   and injects the resulting `Genome` and `Mind` as Bevy resources.
//!   If no save is found, generates a brand-new random Kobara.
//!
//! - **Every 60 ticks**: auto-saves the current state to SQLite.
//!
//! - **On app exit**: performs a final save so no progress is lost.

use std::sync::Mutex;

use bevy::prelude::*;
use bevy::app::AppExit;

use crate::genome::Genome;
use crate::mind::Mind;
use super::{db, load, save};

/// Shared database connection stored as a Bevy resource.
/// Wrapped in a Mutex because rusqlite::Connection is not Sync.
#[derive(Resource)]
pub struct DbConnection(pub Mutex<rusqlite::Connection>);

/// How many ticks between auto-saves.
const AUTOSAVE_INTERVAL: u64 = 60;

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, startup_load)
           .add_systems(Update, autosave_system)
           .add_systems(Last, save_on_exit);
    }
}

/// Opens the database and restores (or creates) the Kobara before any other system runs.
fn startup_load(mut commands: Commands) {
    let conn = db::open().expect("Failed to open save database");

    let (genome, mind) = match load::load_saved(&conn) {
        Ok(Some((g, m))) => {
            info!("Save found — Kobara restored (age: {} ticks)", m.age_ticks);
            (g, m)
        }
        Ok(None) => {
            info!("No save found — generating a new Kobara");
            (Genome::random(), Mind::new())
        }
        Err(e) => {
            error!("Failed to load save: {e} — starting fresh");
            (Genome::random(), Mind::new())
        }
    };

    commands.insert_resource(genome);
    commands.insert_resource(mind);
    commands.insert_resource(DbConnection(Mutex::new(conn)));
}

/// Auto-saves every AUTOSAVE_INTERVAL ticks.
fn autosave_system(
    db:     Res<DbConnection>,
    genome: Res<Genome>,
    mind:   Res<Mind>,
) {
    if mind.age_ticks % AUTOSAVE_INTERVAL == 0 && mind.age_ticks > 0 {
        let conn = db.0.lock().expect("DB lock poisoned");
        if let Err(e) = save::save_all(&conn, &genome, &mind) {
            error!("Auto-save failed: {e}");
        } else {
            debug!("Auto-saved at tick {}", mind.age_ticks);
        }
    }
}

/// Final save when the player closes the app.
fn save_on_exit(
    mut exit_events: EventReader<AppExit>,
    db:     Res<DbConnection>,
    genome: Res<Genome>,
    mind:   Res<Mind>,
) {
    for _ in exit_events.read() {
        let conn = db.0.lock().expect("DB lock poisoned");
        if let Err(e) = save::save_all(&conn, &genome, &mind) {
            error!("Exit save failed: {e}");
        } else {
            info!("Saved on exit at tick {}", mind.age_ticks);
        }
    }
}
