//! Multi-creature system — manages a collection of Kobaras.
//!
//! Only ONE creature is active (visible + interactable) at a time.
//! The player can switch between creatures using a selector UI.
//! Offspring created through reproduction are added to the collection
//! and can be selected later.
//!
//! ## Architecture
//!
//! - `CreatureCollection` (Resource): stores all creature data (genome + mind)
//! - The active creature's data is synced to/from the global `Genome` and `Mind`
//!   resources, so all existing systems (HUD, effects, animation) work unchanged.
//! - Switching creatures: save current state → swap globals → respawn visuals.

use bevy::prelude::*;

use crate::genome::Genome;
use crate::mind::Mind;
use super::reproduction::OffspringBornEvent;

pub struct MultiCreaturePlugin;

impl Plugin for MultiCreaturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CreatureCollection::default())
           .add_event::<SwitchCreatureEvent>()
           .add_systems(Startup, init_collection)
           .add_systems(Update, (collect_offspring, handle_switch));
    }
}

/// A stored creature — genome + mind state.
#[derive(Debug, Clone)]
pub struct StoredCreature {
    pub name: String,
    pub genome: Genome,
    pub mind: Mind,
}

/// Holds all creatures the player has. Index 0 is always the first creature.
#[derive(Resource, Default)]
pub struct CreatureCollection {
    pub creatures: Vec<StoredCreature>,
    /// Index of the currently active creature.
    pub active_index: usize,
}

impl CreatureCollection {
    pub fn active(&self) -> Option<&StoredCreature> {
        self.creatures.get(self.active_index)
    }

    pub fn count(&self) -> usize {
        self.creatures.len()
    }
}

/// Event to switch to a different creature by index.
#[derive(Event)]
pub struct SwitchCreatureEvent {
    pub index: usize,
}

/// On startup, register the primary creature into the collection.
fn init_collection(
    genome: Res<Genome>,
    mind: Res<Mind>,
    mut collection: ResMut<CreatureCollection>,
) {
    collection.creatures.push(StoredCreature {
        name: format!("Kobara #1"),
        genome: genome.clone(),
        mind: mind.clone(),
    });
    collection.active_index = 0;
}

/// When offspring are born, add them to the collection.
fn collect_offspring(
    mut events: EventReader<OffspringBornEvent>,
    mut collection: ResMut<CreatureCollection>,
) {
    for event in events.read() {
        let idx = collection.creatures.len() + 1;
        collection.creatures.push(StoredCreature {
            name: format!("Kobara #{}", idx),
            genome: event.genome.clone(),
            mind: Mind::new(),
        });
        info!(
            "New Kobara added to collection! Total: {} (Species: {:?})",
            collection.creatures.len(), event.genome.species
        );
    }
}

/// Handles switching the active creature.
///
/// Saves the current creature's state back to the collection,
/// then loads the new creature's state into the global resources.
/// The visual respawn is handled by detecting the Genome resource change.
fn handle_switch(
    mut events: EventReader<SwitchCreatureEvent>,
    mut collection: ResMut<CreatureCollection>,
    mut genome: ResMut<Genome>,
    mut mind: ResMut<Mind>,
) {
    for event in events.read() {
        if event.index >= collection.creatures.len() {
            warn!("Invalid creature index: {}", event.index);
            continue;
        }

        if event.index == collection.active_index {
            continue; // Already active
        }

        // Save current creature's state
        let old_index = collection.active_index;
        if let Some(current) = collection.creatures.get_mut(old_index) {
            current.genome = genome.clone();
            current.mind = mind.clone();
        }

        // Load new creature
        collection.active_index = event.index;
        let new_creature = &collection.creatures[event.index];
        *genome = new_creature.genome.clone();
        *mind = new_creature.mind.clone();

        info!(
            "Switched to {} (Species: {:?}, Age: {} ticks)",
            new_creature.name, new_creature.genome.species, new_creature.mind.age_ticks
        );
    }
}
