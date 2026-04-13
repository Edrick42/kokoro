//! Heartbeat audio — low-frequency pulse synced with HeartbeatState.
//!
//! Plays a short bass thump each time the heart beats. The sound is
//! a sine wave at 55 Hz (sub-bass A1) with a fast envelope.

use bevy::prelude::*;
use bevy::audio::PlaybackSettings;

use crate::creature::identity::species::CreatureRoot;
use crate::visuals::breathing::HeartbeatState;
use super::SoundBank;

/// Tracks previous pulse state to detect rising edges.
#[derive(Resource, Default)]
pub struct HeartbeatAudioState {
    was_active: bool,
}

/// Plays heartbeat sound when pulse transitions from inactive to active.
pub fn heartbeat_audio_system(
    mut state: ResMut<HeartbeatAudioState>,
    root_q: Query<&HeartbeatState, With<CreatureRoot>>,
    sound_bank: Option<Res<SoundBank>>,
    mut commands: Commands,
) {
    let Ok(heartbeat) = root_q.single() else { return };
    let Some(bank) = sound_bank else { return };

    let is_active = heartbeat.pulse_active > 0.0;

    // Rising edge: was inactive, now active
    if is_active && !state.was_active {
        if let Some(handle) = bank.get_heartbeat() {
            // Volume inversely proportional to BPM (quieter when fast)
            let vol = (72.0 / heartbeat.bpm.max(30.0)).min(1.0) * crate::config::audio::HEARTBEAT_VOLUME;
            commands.spawn((
                AudioPlayer::new(handle),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(vol)),
            ));
        }
    }

    state.was_active = is_active;
}

/// Generates the heartbeat thump samples (used when no .ogg file exists).
#[allow(dead_code)]
pub fn generate_heartbeat() -> Vec<f32> {
    use super::synth;

    // Bass thump: sine at 55 Hz + harmonic at 110 Hz
    let mut base = synth::sine(55.0, 0.08);
    let harmonic = synth::sine(110.0, 0.08);
    synth::mix_into(&mut base, &harmonic);
    synth::scale(&mut base, 0.7); // reduce harmonic after mix
    synth::apply_envelope(&mut base, 0.005, 0.015, 0.6, 0.04);
    base
}
