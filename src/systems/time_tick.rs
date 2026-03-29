//! Game tick plugin.
//!
//! Every `TICK_INTERVAL` real-world seconds, the creature's mind is updated.
//! Keeping the tick rate separate from the frame rate lets us slow or speed up
//! time independently of rendering — useful for simulating day/night cycles later.

use bevy::prelude::*;
use crate::{genome::Genome, mind::Mind};

/// How many real-world seconds pass between game ticks.
const TICK_INTERVAL: f32 = 1.0;

#[derive(Resource)]
struct TickTimer(Timer);

pub struct TimeTickPlugin;

impl Plugin for TimeTickPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TickTimer(Timer::from_seconds(
            TICK_INTERVAL,
            TimerMode::Repeating,
        )))
        .add_systems(Update, tick_system);
    }
}

fn tick_system(
    time:       Res<Time>,
    mut timer:  ResMut<TickTimer>,
    mut mind:   ResMut<Mind>,
    genome:     Res<Genome>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        mind.update_mood(&genome);
    }
}
