//! Species-specific procedural vocalizations — organic retro style.
//!
//! Uses layered harmonics and formant-like frequency combinations to create
//! sounds that feel like stylized animal calls, not raw chiptune beeps.
//!
//! ## Design principles
//!
//! Real animal sounds have:
//! 1. **Formants** — multiple resonant frequencies (not a single tone)
//! 2. **Pitch contour** — the pitch rises/falls naturally (not constant)
//! 3. **Breath noise** — air turbulence mixed with the tone
//! 4. **Harmonics** — overtones at 2×, 3×, 4× the base frequency
//!
//! ## Species voice design
//!
//! - **Moluun** (mammal): cat-like purr/mew — layered harmonics + breath noise
//! - **Pylum** (bird): finch-like chirp — fast frequency sweep + harmonics
//! - **Skael** (reptile): gecko-like click/hiss — noise bursts + low resonance
//! - **Nyxal** (cephalopod): whale-like song — slow sweeps + deep harmonics

use crate::genome::Species;
use super::synth::{self, SAMPLE_RATE};

/// Helper: create a formant-like tone with harmonics and breath noise.
/// This is the building block for organic-sounding vocalizations.
fn formant_tone(base_freq: f32, duration: f32, harmonics: &[(f32, f32)], breath: f32) -> Vec<f32> {
    // Base tone
    let mut out = synth::sine(base_freq, duration);

    // Add harmonics (overtones at multiples of base frequency)
    for &(multiplier, volume) in harmonics {
        let mut h = synth::sine(base_freq * multiplier, duration);
        synth::scale(&mut h, volume);
        synth::mix_into(&mut out, &h);
    }

    // Add breath noise (turbulence — makes it sound organic)
    if breath > 0.0 {
        let mut noise = synth::noise(duration);
        synth::scale(&mut noise, breath);
        synth::mix_into(&mut out, &noise);
    }

    out
}

/// Helper: pitch contour — modulates frequency over time.
/// Creates a sound that rises or falls naturally.
fn swept_formant(start: f32, end: f32, duration: f32, harmonics: &[(f32, f32)], breath: f32) -> Vec<f32> {
    let mut out = synth::sine_sweep(start, end, duration);

    for &(mult, vol) in harmonics {
        let mut h = synth::sine_sweep(start * mult, end * mult, duration);
        synth::scale(&mut h, vol);
        synth::mix_into(&mut out, &h);
    }

    if breath > 0.0 {
        let mut noise = synth::noise(duration);
        synth::scale(&mut noise, breath);
        synth::mix_into(&mut out, &noise);
    }

    out
}

// ===================================================================
// MOLUUN — mammal: cat-like purr, soft mew, gentle yawn
// ===================================================================

fn moluun_happy() -> Vec<f32> {
    // Cat-like mew: rising then falling pitch, warm harmonics
    let mut mew = swept_formant(
        280.0, 380.0, 0.15,
        &[(2.0, 0.3), (3.0, 0.1)], // 2nd and 3rd harmonics
        0.05, // slight breath
    );
    synth::apply_envelope(&mut mew, 0.02, 0.04, 0.5, 0.05);

    // Second part: falls back down
    let mut fall = swept_formant(
        370.0, 250.0, 0.2,
        &[(2.0, 0.25), (3.0, 0.08)],
        0.04,
    );
    synth::apply_envelope(&mut fall, 0.01, 0.05, 0.4, 0.08);

    mew.extend(&fall);
    synth::scale(&mut mew, 0.7);
    mew
}

fn moluun_hungry() -> Vec<f32> {
    // Soft whine: descending pitch with slight vibrato
    let n = (SAMPLE_RATE as f32 * 0.4) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let progress = t / 0.4;
        let freq = 350.0 - 120.0 * progress; // descends from 350 to 230
        let vibrato = (t * 6.0 * std::f32::consts::TAU).sin() * 8.0; // 6Hz vibrato
        let sample = ((freq + vibrato) * t * std::f32::consts::TAU).sin();
        // Add 2nd harmonic
        let h2 = ((freq * 2.0 + vibrato) * t * std::f32::consts::TAU).sin() * 0.2;
        out.push((sample + h2).clamp(-1.0, 1.0));
    }
    synth::apply_envelope(&mut out, 0.03, 0.08, 0.4, 0.15);

    // Tiny mew at the end (pleading)
    let pause = (SAMPLE_RATE as f32 * 0.06) as usize;
    out.extend(std::iter::repeat(0.0f32).take(pause));
    let mut plea = swept_formant(300.0, 260.0, 0.1, &[(2.0, 0.2)], 0.03);
    synth::apply_envelope(&mut plea, 0.01, 0.03, 0.3, 0.04);
    out.extend(&plea);

    synth::scale(&mut out, 0.6);
    out
}

fn moluun_sleepy() -> Vec<f32> {
    // Purr: low rumble with natural irregularity (like a real purr ~25Hz modulation)
    let duration = 0.6;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        // Purr is amplitude-modulated at ~25Hz (the characteristic purr rate)
        let purr_mod = ((25.0 * t * std::f32::consts::TAU).sin() * 0.5 + 0.5).max(0.0);
        let tone = (80.0 * t * std::f32::consts::TAU).sin();
        let h2 = (160.0 * t * std::f32::consts::TAU).sin() * 0.3;
        let breath = synth::noise(0.001)[0] * 0.05; // micro noise
        out.push(((tone + h2) * purr_mod + breath).clamp(-1.0, 1.0));
    }
    synth::apply_envelope(&mut out, 0.1, 0.1, 0.3, 0.25);
    synth::scale(&mut out, 0.5);
    out
}

// ===================================================================
// PYLUM — bird: finch-like chirps, trills, peeps
// ===================================================================

fn pylum_happy() -> Vec<f32> {
    // Bird chirp: fast upward sweep + downward sweep (like a finch song)
    let mut up = swept_formant(
        1200.0, 2200.0, 0.06,
        &[(2.0, 0.15), (0.5, 0.1)], // sub-harmonic gives body
        0.02,
    );
    synth::apply_envelope(&mut up, 0.003, 0.015, 0.5, 0.015);

    let pause = (SAMPLE_RATE as f32 * 0.02) as usize;
    up.extend(std::iter::repeat(0.0f32).take(pause));

    let mut down = swept_formant(
        2200.0, 1500.0, 0.08,
        &[(2.0, 0.12)],
        0.02,
    );
    synth::apply_envelope(&mut down, 0.003, 0.02, 0.4, 0.02);
    up.extend(&down);

    // Short trill (rapid frequency oscillation)
    up.extend(std::iter::repeat(0.0f32).take(pause));
    let trill_n = (SAMPLE_RATE as f32 * 0.1) as usize;
    let mut trill = Vec::with_capacity(trill_n);
    for i in 0..trill_n {
        let t = i as f32 / SAMPLE_RATE as f32;
        // Rapid oscillation between two frequencies (~30Hz modulation rate)
        let mod_freq = (30.0 * t * std::f32::consts::TAU).sin();
        let freq = 1800.0 + mod_freq * 400.0; // oscillates 1400-2200 Hz
        trill.push((freq * t * std::f32::consts::TAU).sin());
    }
    synth::apply_envelope(&mut trill, 0.005, 0.02, 0.35, 0.03);
    up.extend(&trill);

    synth::scale(&mut up, 0.5);
    up
}

fn pylum_hungry() -> Vec<f32> {
    // Insistent peep-peep-peep (chick begging for food)
    let mut out = Vec::new();
    for i in 0..3 {
        let freq = 1600.0 + i as f32 * 150.0; // slightly ascending = more urgent
        let mut peep = swept_formant(
            freq, freq + 200.0, 0.07,
            &[(2.0, 0.1)],
            0.02,
        );
        synth::apply_envelope(&mut peep, 0.003, 0.02, 0.45, 0.02);
        out.extend(&peep);
        let pause = (SAMPLE_RATE as f32 * 0.08) as usize;
        out.extend(std::iter::repeat(0.0f32).take(pause));
    }
    synth::scale(&mut out, 0.45);
    out
}

fn pylum_sleepy() -> Vec<f32> {
    // Single soft descending peep (settling down)
    let mut peep = swept_formant(
        1000.0, 600.0, 0.2,
        &[(2.0, 0.08)],
        0.03,
    );
    synth::apply_envelope(&mut peep, 0.01, 0.06, 0.2, 0.1);
    synth::scale(&mut peep, 0.35);
    peep
}

// ===================================================================
// SKAEL — reptile: gecko clicks, low hisses, rumbles
// ===================================================================

fn skael_happy() -> Vec<f32> {
    // Gecko-like chirp: short burst with resonance (geckos click rapidly)
    let mut out = Vec::new();
    for _ in 0..2 {
        // Click: very short, sharp attack, with body resonance
        let mut click = formant_tone(
            800.0, 0.03,
            &[(2.5, 0.3), (4.0, 0.15)], // non-harmonic overtones = click quality
            0.1, // noisy attack
        );
        synth::apply_envelope(&mut click, 0.001, 0.01, 0.3, 0.01);
        out.extend(&click);
        let pause = (SAMPLE_RATE as f32 * 0.05) as usize;
        out.extend(std::iter::repeat(0.0f32).take(pause));
    }
    synth::scale(&mut out, 0.6);
    out
}

fn skael_hungry() -> Vec<f32> {
    // Hiss: shaped noise with low resonance underneath
    let mut hiss = synth::noise(0.25);
    synth::apply_envelope(&mut hiss, 0.03, 0.05, 0.35, 0.1);
    synth::scale(&mut hiss, 0.35);

    // Low guttural rumble underneath
    let mut rumble = formant_tone(
        55.0, 0.3,
        &[(2.0, 0.4), (3.0, 0.15)],
        0.08,
    );
    synth::apply_envelope(&mut rumble, 0.05, 0.08, 0.3, 0.12);
    synth::scale(&mut rumble, 0.5);

    // Layer together
    let offset = (SAMPLE_RATE as f32 * 0.05) as usize;
    for (i, &s) in rumble.iter().enumerate() {
        let idx = offset + i;
        if idx < hiss.len() {
            hiss[idx] = (hiss[idx] + s).clamp(-1.0, 1.0);
        } else {
            hiss.push(s);
        }
    }
    hiss
}

fn skael_sleepy() -> Vec<f32> {
    // Single deep click + low resonant decay (reptile settling on rock)
    let mut click = formant_tone(
        300.0, 0.08,
        &[(2.0, 0.3), (5.0, 0.1)],
        0.15,
    );
    synth::apply_envelope(&mut click, 0.001, 0.02, 0.2, 0.04);
    synth::scale(&mut click, 0.5);
    click
}

// ===================================================================
// NYXAL — cephalopod: whale-like songs, sonar pings, deep drones
// ===================================================================

fn nyxal_happy() -> Vec<f32> {
    // Whale-like song: slow ascending sweep with rich harmonics
    let mut song = swept_formant(
        150.0, 280.0, 0.5,
        &[(2.0, 0.35), (3.0, 0.15), (5.0, 0.05)],
        0.02,
    );
    synth::apply_envelope(&mut song, 0.1, 0.1, 0.4, 0.2);

    // Sonar ping at the end
    let pause = (SAMPLE_RATE as f32 * 0.08) as usize;
    song.extend(std::iter::repeat(0.0f32).take(pause));
    let mut ping = synth::sine(1200.0, 0.04);
    synth::apply_envelope(&mut ping, 0.002, 0.01, 0.2, 0.02);
    synth::scale(&mut ping, 0.3);
    song.extend(&ping);

    synth::scale(&mut song, 0.5);
    song
}

fn nyxal_hungry() -> Vec<f32> {
    // Echolocation clicks: rapid, spaced, with slight frequency variation
    let mut out = Vec::new();
    for i in 0..4 {
        let freq = 1800.0 + i as f32 * 200.0;
        let mut ping = synth::sine(freq, 0.02);
        synth::apply_envelope(&mut ping, 0.001, 0.005, 0.3, 0.008);
        out.extend(&ping);
        // Varying pauses (natural rhythm)
        let pause_ms = 40.0 + (i as f32 * 5.0);
        let pause = (SAMPLE_RATE as f32 * pause_ms / 1000.0) as usize;
        out.extend(std::iter::repeat(0.0f32).take(pause));
    }
    synth::scale(&mut out, 0.45);
    out
}

fn nyxal_sleepy() -> Vec<f32> {
    // Deep ocean drone: very low, with overtones phasing in and out
    let duration = 0.8;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let base = (45.0 * t * std::f32::consts::TAU).sin();
        // Phasing overtone (comes and goes)
        let phase_mod = (0.5 * t * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        let h2 = (90.0 * t * std::f32::consts::TAU).sin() * 0.3 * phase_mod;
        let h3 = (135.0 * t * std::f32::consts::TAU).sin() * 0.1 * (1.0 - phase_mod);
        out.push((base + h2 + h3).clamp(-1.0, 1.0));
    }
    synth::apply_envelope(&mut out, 0.2, 0.15, 0.2, 0.35);
    synth::scale(&mut out, 0.4);
    out
}

// ===================================================================
// PUBLIC API
// ===================================================================

pub fn vocalize_happy(species: &Species) -> Vec<f32> {
    match species {
        Species::Moluun => moluun_happy(),
        Species::Pylum  => pylum_happy(),
        Species::Skael  => skael_happy(),
        Species::Nyxal  => nyxal_happy(),
    }
}

pub fn vocalize_hungry(species: &Species) -> Vec<f32> {
    match species {
        Species::Moluun => moluun_hungry(),
        Species::Pylum  => pylum_hungry(),
        Species::Skael  => skael_hungry(),
        Species::Nyxal  => nyxal_hungry(),
    }
}

pub fn vocalize_sleepy(species: &Species) -> Vec<f32> {
    match species {
        Species::Moluun => moluun_sleepy(),
        Species::Pylum  => pylum_sleepy(),
        Species::Skael  => skael_sleepy(),
        Species::Nyxal  => nyxal_sleepy(),
    }
}

/// Heartbeat thump — low kick with body resonance.
pub fn heartbeat_thump() -> Vec<f32> {
    let mut thump = formant_tone(
        50.0, 0.12,
        &[(2.0, 0.4), (3.0, 0.1)],
        0.05,
    );
    synth::apply_envelope(&mut thump, 0.003, 0.03, 0.2, 0.06);
    synth::scale(&mut thump, 0.6);
    thump
}

/// Eating crunch — noise burst with low body thump.
pub fn eating_sound() -> Vec<f32> {
    let mut crunch = synth::noise(0.06);
    synth::apply_envelope(&mut crunch, 0.002, 0.015, 0.4, 0.02);
    synth::scale(&mut crunch, 0.4);
    let mut thump = formant_tone(80.0, 0.08, &[(2.0, 0.3)], 0.03);
    synth::apply_envelope(&mut thump, 0.003, 0.02, 0.3, 0.03);
    synth::mix_into(&mut crunch, &thump);
    crunch
}
