//! Reproduction system — breeding Kobaras to produce offspring.
//!
//! The player can breed at any time via the "Breed" button.
//! The creature mates with a "wild" partner (random genome of the same species).
//! The child inherits genes from both parents through crossover + mutation.
//!
//! ## Rules
//! - Always available (no age/stat restrictions)
//! - Cooldown of 60 ticks between breedings
//! - Child species matches the parent
//! - Each gene is randomly picked from either parent, with 15% mutation chance

use bevy::prelude::*;
use crate::genome::Genome;
use crate::mind::Mind;

pub struct ReproductionPlugin;

impl Plugin for ReproductionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BreedingState::default())
           .add_event::<BreedRequestEvent>()
           .add_event::<OffspringBornEvent>()
           .add_systems(Update, handle_breed_request);
    }
}

/// Tracks breeding cooldown.
#[derive(Resource, Default)]
pub struct BreedingState {
    /// Tick when last breeding occurred.
    pub last_breed_tick: u64,
    /// Total offspring produced.
    pub offspring_count: u32,
}

const BREED_COOLDOWN: u64 = 60;

/// Fired when the player requests a breeding.
#[derive(Event)]
pub struct BreedRequestEvent;

/// Fired when an offspring is successfully created.
/// Carries the child's genome for the collection system to store.
#[derive(Event)]
pub struct OffspringBornEvent {
    pub genome: Genome,
}

/// Checks if breeding conditions are met and produces offspring.
fn handle_breed_request(
    mut events: EventReader<BreedRequestEvent>,
    mut offspring_events: EventWriter<OffspringBornEvent>,
    mind: Res<Mind>,
    genome: Res<Genome>,
    mut state: ResMut<BreedingState>,
) {
    for _ in events.read() {
        // Check cooldown only
        if mind.age_ticks < state.last_breed_tick + BREED_COOLDOWN && state.offspring_count > 0 {
            let remaining = (state.last_breed_tick + BREED_COOLDOWN) - mind.age_ticks;
            info!("Breeding on cooldown — {remaining} ticks remaining");
            continue;
        }

        // Generate a "wild" partner of the same species
        let wild_partner = Genome::random_for(genome.species.clone());

        // Create offspring through crossover
        let child = Genome::crossover(&genome, &wild_partner, genome.species.clone());

        info!(
            "Offspring #{} born! Hue: {:.0}, Curiosity: {:.2}, Resilience: {:.2}",
            state.offspring_count + 1, child.hue, child.curiosity, child.resilience
        );

        state.last_breed_tick = mind.age_ticks;
        state.offspring_count += 1;

        offspring_events.write(OffspringBornEvent { genome: child });
    }
}

/// Returns true if the current creature can breed right now.
pub fn can_breed(mind: &Mind, state: &BreedingState) -> bool {
    state.offspring_count == 0 || mind.age_ticks >= state.last_breed_tick + BREED_COOLDOWN
}
