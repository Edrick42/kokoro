//! Multi-creature system — manages a collection of Kobaras.
//!
//! All three species (Moluun, Pylum, Skael) are created at startup.
//! Only ONE creature is active (visible + interactable) at a time.
//! The species buttons switch to the existing creature of that species.
//!
//! ## Architecture
//!
//! - `CreatureCollection` (Resource): stores all creature data (genome + mind)
//! - The active creature's data is synced to/from the global `Genome` and `Mind`
//!   resources, so all existing systems (HUD, effects, animation) work unchanged.
//! - Switching creatures: save current state → swap globals → respawn visuals.

use bevy::prelude::*;

use crate::game::state::AppState;
use crate::creature::anatomy::AnatomyState;
use crate::genome::{Genome, Species};
use crate::mind::Mind;
use crate::creature::egg::EggData;
use crate::persistence::plugin::DbConnection;
use crate::persistence::load;

pub struct MultiCreaturePlugin;

impl Plugin for MultiCreaturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CreatureCollection::default())
           .add_event::<SelectSpeciesEvent>()
           .add_systems(Startup, init_collection)
           .add_systems(Update, handle_select_species.run_if(in_state(AppState::Gameplay)));
    }
}

/// A stored creature — genome + mind state + egg data + anatomy.
#[derive(Debug, Clone)]
pub struct StoredCreature {
    pub name: String,
    pub genome: Genome,
    pub mind: Mind,
    pub egg: EggData,
    pub anatomy: AnatomyState,
}

/// Holds all creatures the player has. Index 0 = Moluun, 1 = Pylum, 2 = Skael.
#[derive(Resource, Default)]
pub struct CreatureCollection {
    pub creatures: Vec<StoredCreature>,
    /// Index of the currently active creature.
    pub active_index: usize,
    /// Set to true when the active creature changes and visuals need respawning.
    pub visuals_dirty: bool,
}

/// Event to switch to a species. If it already exists, just switch to it.
#[derive(Event)]
pub struct SelectSpeciesEvent {
    pub species: Species,
}

/// On startup, load the creature collection from the database.
/// If no saved collection exists (first run or pre-migration), create fresh creatures.
fn init_collection(
    genome: Res<Genome>,
    mind: Res<Mind>,
    anatomy: Res<AnatomyState>,
    db: Res<DbConnection>,
    mut collection: ResMut<CreatureCollection>,
) {
    // Try to load from database
    let conn = db.0.lock().expect("DB lock poisoned");
    if let Ok(Some(saved)) = load::load_collection(&conn) {
        if saved.len() >= 2 {
            info!("Loaded {} creatures from database", saved.len());
            collection.creatures = saved;
            collection.active_index = 0;

            // Sync active creature to global resources
            // (already handled by persistence plugin for slot 0)
            return;
        }
    }
    drop(conn);

    info!("No saved collection — creating fresh creatures");

    // Moluun — use the persisted genome/mind/anatomy (already hatched)
    collection.creatures.push(StoredCreature {
        name: "Moluun".to_string(),
        genome: genome.clone(),
        mind: mind.clone(),
        egg: EggData { progress: 1.0, hatched: true },
        anatomy: anatomy.clone(),
    });

    // Pylum — starts as egg
    let pylum_genome = Genome::random_for(Species::Pylum);
    let pylum_anatomy = AnatomyState::new_for(&Species::Pylum, &pylum_genome);
    collection.creatures.push(StoredCreature {
        name: "Pylum".to_string(),
        genome: pylum_genome,
        mind: Mind::new(),
        egg: EggData::default(),
        anatomy: pylum_anatomy,
    });

    // Skael — starts as egg
    let skael_genome = Genome::random_for(Species::Skael);
    let skael_anatomy = AnatomyState::new_for(&Species::Skael, &skael_genome);
    collection.creatures.push(StoredCreature {
        name: "Skael".to_string(),
        genome: skael_genome,
        mind: Mind::new(),
        egg: EggData::default(),
        anatomy: skael_anatomy,
    });

    // Nyxal — starts as egg
    let nyxal_genome = Genome::random_for(Species::Nyxal);
    let nyxal_anatomy = AnatomyState::new_for(&Species::Nyxal, &nyxal_genome);
    collection.creatures.push(StoredCreature {
        name: "Nyxal".to_string(),
        genome: nyxal_genome,
        mind: Mind::new(),
        egg: EggData::default(),
        anatomy: nyxal_anatomy,
    });

    collection.active_index = 0;
}

/// When the player clicks a species button, switch to that species' creature.
fn handle_select_species(
    mut events: EventReader<SelectSpeciesEvent>,
    mut collection: ResMut<CreatureCollection>,
    mut genome: ResMut<Genome>,
    mut mind: ResMut<Mind>,
    mut anatomy: ResMut<AnatomyState>,
) {
    for event in events.read() {
        // Find the creature of this species
        let Some(target_index) = collection.creatures.iter()
            .position(|c| c.genome.species == event.species)
        else {
            warn!("No creature found for species {:?}", event.species);
            continue;
        };

        if target_index == collection.active_index {
            continue;
        }

        // Save current creature's state
        let old_index = collection.active_index;
        if let Some(current) = collection.creatures.get_mut(old_index) {
            current.genome = genome.clone();
            current.mind = mind.clone();
            current.anatomy = anatomy.clone();
        }

        // Load target creature
        collection.active_index = target_index;
        let creature = &collection.creatures[target_index];
        *genome = creature.genome.clone();
        *mind = creature.mind.clone();
        *anatomy = creature.anatomy.clone();

        let name = creature.name.clone();
        let species = creature.genome.species.clone();
        let age = creature.mind.age_ticks;

        collection.visuals_dirty = true;

        info!("Switched to {} (Species: {:?}, Age: {} ticks)", name, species, age);
    }
}
