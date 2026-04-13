//! Reproduction system — breeding Kobaras to produce offspring.
//!
//! Future feature: player breeds creatures, child inherits genes
//! from both parents through crossover + mutation.

use bevy::prelude::*;
use crate::genome::Genome;
use crate::mind::Mind;

#[allow(dead_code)]
pub struct ReproductionPlugin;

impl Plugin for ReproductionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BreedingState::default())
           .add_event::<BreedRequestEvent>()
           .add_event::<OffspringBornEvent>()
           .add_systems(Update, handle_breed_request);
    }
}

#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct BreedingState {
    pub last_breed_tick: u64,
    pub offspring_count: u32,
}

#[allow(dead_code)]
const BREED_COOLDOWN: u64 = crate::config::timing::breeding::BREED_COOLDOWN;

#[derive(Event)]
#[allow(dead_code)]
pub struct BreedRequestEvent;

#[derive(Event)]
#[allow(dead_code)]
pub struct OffspringBornEvent {
    pub genome: Genome,
}

#[allow(dead_code)]
fn handle_breed_request(
    mut events: EventReader<BreedRequestEvent>,
    mut offspring_events: EventWriter<OffspringBornEvent>,
    mind: Res<Mind>,
    genome: Res<Genome>,
    mut state: ResMut<BreedingState>,
) {
    for _ in events.read() {
        if mind.age_ticks < state.last_breed_tick + BREED_COOLDOWN && state.offspring_count > 0 {
            continue;
        }
        let wild_partner = Genome::random_for(genome.species.clone());
        let child = Genome::crossover(&genome, &wild_partner, genome.species.clone());
        state.last_breed_tick = mind.age_ticks;
        state.offspring_count += 1;
        offspring_events.write(OffspringBornEvent { genome: child });
    }
}

#[allow(dead_code)]
pub fn can_breed(mind: &Mind, state: &BreedingState) -> bool {
    state.offspring_count == 0 || mind.age_ticks >= state.last_breed_tick + BREED_COOLDOWN
}
