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

use crate::genome::{Genome, Species};
use crate::mind::Mind;

pub struct MultiCreaturePlugin;

impl Plugin for MultiCreaturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CreatureCollection::default())
           .add_event::<SelectSpeciesEvent>()
           .add_systems(Startup, init_collection)
           .add_systems(Update, handle_select_species);
    }
}

/// A stored creature — genome + mind state.
#[derive(Debug, Clone)]
pub struct StoredCreature {
    pub name: String,
    pub genome: Genome,
    pub mind: Mind,
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

/// On startup, create one creature of each species.
/// The primary creature (from persistence) becomes Moluun.
/// Pylum and Skael get fresh random genomes.
fn init_collection(
    genome: Res<Genome>,
    mind: Res<Mind>,
    mut collection: ResMut<CreatureCollection>,
) {
    // Moluun — use the persisted genome/mind
    collection.creatures.push(StoredCreature {
        name: "Moluun".to_string(),
        genome: genome.clone(),
        mind: mind.clone(),
    });

    // Pylum — fresh random
    collection.creatures.push(StoredCreature {
        name: "Pylum".to_string(),
        genome: Genome::random_for(Species::Pylum),
        mind: Mind::new(),
    });

    // Skael — fresh random
    collection.creatures.push(StoredCreature {
        name: "Skael".to_string(),
        genome: Genome::random_for(Species::Skael),
        mind: Mind::new(),
    });

    // Nyxal — fresh random
    collection.creatures.push(StoredCreature {
        name: "Nyxal".to_string(),
        genome: Genome::random_for(Species::Nyxal),
        mind: Mind::new(),
    });

    collection.active_index = 0;
}

/// When the player clicks a species button, switch to that species' creature.
fn handle_select_species(
    mut events: EventReader<SelectSpeciesEvent>,
    mut collection: ResMut<CreatureCollection>,
    mut genome: ResMut<Genome>,
    mut mind: ResMut<Mind>,
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
        }

        // Load target creature
        collection.active_index = target_index;
        let creature = &collection.creatures[target_index];
        *genome = creature.genome.clone();
        *mind = creature.mind.clone();

        let name = creature.name.clone();
        let species = creature.genome.species.clone();
        let age = creature.mind.age_ticks;

        collection.visuals_dirty = true;

        info!("Switched to {} (Species: {:?}, Age: {} ticks)", name, species, age);
    }
}
