//! Day/night cycle based on the system clock.
//!
//! Reads the local hour and determines the time of day.
//! Colors are handled by the background plugin — this module
//! only provides the `DayCycle` resource with the current state.

use bevy::prelude::*;
use std::time::SystemTime;

/// Broad time-of-day categories.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Sunset,
    Night,
}

/// Shared resource holding the current time-of-day state.
#[derive(Resource, Debug)]
pub struct DayCycle {
    pub hour: u32,
    pub time_of_day: TimeOfDay,
}

impl DayCycle {
    fn from_system_clock() -> Self {
        let hour = current_hour();
        Self {
            hour,
            time_of_day: hour_to_time_of_day(hour),
        }
    }
}

pub struct DayCyclePlugin;

impl Plugin for DayCyclePlugin {
    fn build(&self, app: &mut App) {
        let cycle = DayCycle::from_system_clock();
        app.insert_resource(cycle)
           .add_systems(Update, update_cycle);
    }
}

/// Re-reads the system clock and updates the DayCycle resource.
fn update_cycle(mut cycle: ResMut<DayCycle>) {
    let hour = current_hour();
    if hour != cycle.hour {
        cycle.hour = hour;
        cycle.time_of_day = hour_to_time_of_day(hour);
        info!("Time of day changed: {:?} (hour {})", cycle.time_of_day, hour);
    }
}

fn hour_to_time_of_day(hour: u32) -> TimeOfDay {
    match hour {
        6..=11  => TimeOfDay::Morning,
        12..=16 => TimeOfDay::Afternoon,
        17..=19 => TimeOfDay::Sunset,
        _       => TimeOfDay::Night,
    }
}

fn current_hour() -> u32 {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let local_secs = secs as i64 + local_utc_offset_secs();
    let secs_in_day = local_secs.rem_euclid(86400);
    (secs_in_day / 3600) as u32
}

#[cfg(unix)]
fn local_utc_offset_secs() -> i64 {
    use std::mem::MaybeUninit;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as libc::time_t;

    let mut tm = MaybeUninit::uninit();
    let result = unsafe { libc::localtime_r(&now, tm.as_mut_ptr()) };
    if result.is_null() {
        return 0;
    }
    let tm = unsafe { tm.assume_init() };
    tm.tm_gmtoff as i64
}

#[cfg(not(unix))]
fn local_utc_offset_secs() -> i64 {
    0
}
