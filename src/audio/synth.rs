//! Waveform synthesis — generates audio samples from pure math.
//!
//! All generators produce `Vec<f32>` samples in the range [-1.0, 1.0].
//! These are the building blocks for every sound in the game:
//! creature vocalizations, heartbeat, breathing, UI clicks.

use std::f32::consts::TAU;

/// Sample rate for all synthesized audio. 22050 Hz is enough for
/// chiptune-style sounds and uses half the memory of CD-quality 44100.
pub const SAMPLE_RATE: u32 = 22050;

/// Pure sine wave — smooth, warm tone.
/// Used for Nyxal drones, ambient pads, heartbeat bass.
pub fn sine(freq: f32, duration: f32) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            (TAU * freq * t).sin()
        })
        .collect()
}

/// Square wave — harsh, classic chiptune.
/// `duty` controls the pulse width (0.5 = classic square, 0.25 = narrow pulse).
/// Used for Pylum chirps, UI clicks.
pub fn square(freq: f32, duration: f32, duty: f32) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let phase = (t * freq).fract();
            if phase < duty { 1.0 } else { -1.0 }
        })
        .collect()
}

/// Triangle wave — softer than square, retro feel.
/// Used for Moluun purrs, gentle tones.
pub fn triangle(freq: f32, duration: f32) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let phase = (t * freq).fract();
            if phase < 0.5 {
                4.0 * phase - 1.0
            } else {
                3.0 - 4.0 * phase
            }
        })
        .collect()
}

/// White noise — random samples.
/// Used for breathing, Skael hisses, ambient texture.
pub fn noise(duration: f32) -> Vec<f32> {
    use rand::Rng;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut rng = rand::rng();
    (0..n)
        .map(|_| rng.random_range(-1.0f32..1.0))
        .collect()
}

/// Applies an ADSR envelope to a sample buffer.
///
/// - **Attack**: time (seconds) to ramp from 0 to full volume
/// - **Decay**: time to drop from full to sustain level
/// - **Sustain**: held volume level (0.0-1.0)
/// - **Release**: time to fade from sustain to silence
pub fn apply_envelope(samples: &mut [f32], attack: f32, decay: f32, sustain: f32, release: f32) {
    let total = samples.len() as f32 / SAMPLE_RATE as f32;
    let attack_end = attack;
    let decay_end = attack_end + decay;
    let release_start = (total - release).max(decay_end);

    for (i, sample) in samples.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let gain = if t < attack_end {
            if attack > 0.0 { t / attack } else { 1.0 }
        } else if t < decay_end {
            let decay_t = (t - attack_end) / decay.max(0.001);
            1.0 - (1.0 - sustain) * decay_t
        } else if t < release_start {
            sustain
        } else {
            let release_t = (t - release_start) / release.max(0.001);
            sustain * (1.0 - release_t).max(0.0)
        };
        *sample *= gain;
    }
}

/// Scales all samples by a volume factor.
pub fn scale(samples: &mut [f32], volume: f32) {
    for s in samples.iter_mut() {
        *s *= volume;
    }
}

/// Mixes source samples into destination (additive, clamped).
pub fn mix_into(dst: &mut [f32], src: &[f32]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d = (*d + s).clamp(-1.0, 1.0);
    }
}

/// Frequency sweep from `start_freq` to `end_freq` over the duration.
pub fn sine_sweep(start_freq: f32, end_freq: f32, duration: f32) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let progress = t / duration;
            let freq = start_freq + (end_freq - start_freq) * progress;
            (TAU * freq * t).sin()
        })
        .collect()
}
