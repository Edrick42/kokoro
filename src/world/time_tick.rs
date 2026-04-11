//! Game tick plugin.
//!
//! Every `TICK_INTERVAL` real-world seconds, the creature's mind is updated.
//! Keeping the tick rate separate from the frame rate lets us slow or speed up
//! time independently of rendering — useful for simulating day/night cycles later.

use bevy::prelude::*;
use crate::{config, game::state::AppState, genome::Genome, mind::Mind, world::daycycle::DayCycle};

#[derive(Resource)]
struct TickTimer(Timer);

pub struct TimeTickPlugin;

impl Plugin for TimeTickPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TickTimer(Timer::from_seconds(
            config::TICK_INTERVAL,
            TimerMode::Repeating,
        )))
        .add_systems(Update, tick_system.run_if(in_state(AppState::Gameplay)));
    }
}

fn tick_system(
    time:       Res<Time>,
    mut timer:  ResMut<TickTimer>,
    mut mind:   ResMut<Mind>,
    genome:     Res<Genome>,
    day_cycle:  Option<Res<DayCycle>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        mind.update_mood(&genome);

        // Circadian modifier: creatures with a night-owl gene (circadian < 0.3)
        // get a happiness boost at night, while early-birds (circadian > 0.7)
        // get a boost in the morning. This is a small emergent behaviour layer.
        if let Some(cycle) = &day_cycle {
            use crate::world::daycycle::TimeOfDay;
            let bonus = match cycle.time_of_day {
                TimeOfDay::Night   if genome.circadian < config::timing::circadian::NIGHT_OWL_THRESHOLD =>
                    config::timing::circadian::PREFERRED_BONUS,
                TimeOfDay::Morning if genome.circadian > config::timing::circadian::EARLY_BIRD_THRESHOLD =>
                    config::timing::circadian::PREFERRED_BONUS,
                TimeOfDay::Night   if genome.circadian > config::timing::circadian::EARLY_BIRD_THRESHOLD =>
                    config::timing::circadian::NON_PREFERRED_PENALTY,
                TimeOfDay::Morning if genome.circadian < config::timing::circadian::NIGHT_OWL_THRESHOLD =>
                    config::timing::circadian::NON_PREFERRED_PENALTY,
                _ => 0.0,
            };
            mind.stats.happiness = (mind.stats.happiness + bonus).clamp(0.0, 100.0);
        }
    }
}
