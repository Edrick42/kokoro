//! Day/night cycle based on the system clock.
//!
//! Reads the local hour and smoothly transitions the background color:
//! - Morning (6–12):   light sky blue
//! - Afternoon (12–17): warm white
//! - Sunset (17–20):   orange/pink
//! - Night (20–6):     dark blue
//!
//! The `DayCycle` resource is also read by the tick system to apply
//! a small circadian happiness modifier based on the creature's genome.

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
    /// Creates a new DayCycle by reading the system clock.
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
        let bg = background_color(&cycle);

        app.insert_resource(cycle)
           .insert_resource(ClearColor(bg))
           .add_systems(Update, (update_cycle, update_background).chain());
    }
}

/// Re-reads the system clock and updates the DayCycle resource.
///
/// Runs every frame but the values only change once per real hour,
/// so the overhead is negligible.
fn update_cycle(mut cycle: ResMut<DayCycle>) {
    let hour = current_hour();
    if hour != cycle.hour {
        cycle.hour = hour;
        cycle.time_of_day = hour_to_time_of_day(hour);
        info!("Time of day changed: {:?} (hour {})", cycle.time_of_day, hour);
    }
}

/// Sets the window background color based on the current time of day.
fn update_background(cycle: Res<DayCycle>, mut clear: ResMut<ClearColor>) {
    if cycle.is_changed() {
        clear.0 = background_color(&cycle);
    }
}

/// Maps the current DayCycle to a background color.
fn background_color(cycle: &DayCycle) -> Color {
    match cycle.time_of_day {
        TimeOfDay::Morning   => Color::srgb(0.53, 0.81, 0.98), // light sky blue
        TimeOfDay::Afternoon => Color::srgb(0.94, 0.94, 0.90), // warm white
        TimeOfDay::Sunset    => Color::srgb(0.98, 0.60, 0.40), // orange/pink
        TimeOfDay::Night     => Color::srgb(0.10, 0.10, 0.28), // dark navy
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

/// Returns the current local hour (0–23) from the system clock.
///
/// Uses UTC offset calculation from libc on Unix. Falls back to UTC
/// if the offset cannot be determined.
fn current_hour() -> u32 {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Try to get local time offset from the C runtime
    let local_secs = secs as i64 + local_utc_offset_secs();
    let secs_in_day = local_secs.rem_euclid(86400);
    (secs_in_day / 3600) as u32
}

/// Returns the local UTC offset in seconds.
///
/// On Unix, uses libc::localtime_r. On other platforms, returns 0 (UTC).
#[cfg(unix)]
fn local_utc_offset_secs() -> i64 {
    use std::mem::MaybeUninit;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as libc::time_t;

    let mut tm = MaybeUninit::uninit();
    // SAFETY: localtime_r is thread-safe and writes into our buffer.
    let result = unsafe { libc::localtime_r(&now, tm.as_mut_ptr()) };
    if result.is_null() {
        return 0;
    }
    let tm = unsafe { tm.assume_init() };
    tm.tm_gmtoff as i64
}

#[cfg(not(unix))]
fn local_utc_offset_secs() -> i64 {
    0 // Fallback to UTC on non-Unix platforms
}
