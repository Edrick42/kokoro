//! Species-specific vocal sound synthesis.
//!
//! Each species has a distinct audio character derived from its biology:
//! - Moluun: warm triangle waves (mammalian purrs)
//! - Pylum: sharp square waves (avian chirps and trills)
//! - Skael: low sine + noise (reptilian rumbles and hisses)
//! - Nyxal: pure sine drones (alien electronic pulses)

use crate::genome::Species;
use super::synth;

/// Generates a mood vocalization for the given species.
/// Returns f32 samples ready for WAV conversion.
pub fn vocalize_happy(species: &Species) -> Vec<f32> {
    match species {
        Species::Moluun => moluun_purr(),
        Species::Pylum  => pylum_chirp(),
        Species::Skael  => skael_rumble(),
        Species::Nyxal  => nyxal_drone(),
    }
}

pub fn vocalize_hungry(species: &Species) -> Vec<f32> {
    match species {
        Species::Moluun => moluun_whine(),
        Species::Pylum  => pylum_screech(),
        Species::Skael  => skael_hiss(),
        Species::Nyxal  => nyxal_pulse(),
    }
}

pub fn vocalize_sleepy(species: &Species) -> Vec<f32> {
    match species {
        Species::Moluun => moluun_sigh(),
        Species::Pylum  => pylum_coo(),
        Species::Skael  => skael_click(),
        Species::Nyxal  => nyxal_low_drone(),
    }
}

// --- Moluun: warm, mammalian ---

fn moluun_purr() -> Vec<f32> {
    let mut s = synth::triangle(200.0, 0.4);
    synth::apply_envelope(&mut s, 0.1, 0.05, 0.8, 0.15);
    s
}

fn moluun_whine() -> Vec<f32> {
    let mut s = synth::sine_sweep(400.0, 280.0, 0.3);
    synth::apply_envelope(&mut s, 0.05, 0.1, 0.6, 0.1);
    s
}

fn moluun_sigh() -> Vec<f32> {
    let mut s = synth::triangle(180.0, 0.5);
    synth::apply_envelope(&mut s, 0.15, 0.1, 0.4, 0.2);
    s
}

// --- Pylum: bright, avian ---

fn pylum_chirp() -> Vec<f32> {
    let mut s = synth::square(900.0, 0.08, 0.4);
    synth::apply_envelope(&mut s, 0.005, 0.02, 0.6, 0.03);
    s
}

fn pylum_screech() -> Vec<f32> {
    let mut s = synth::square(1500.0, 0.12, 0.5);
    synth::apply_envelope(&mut s, 0.005, 0.02, 0.8, 0.05);
    s
}

fn pylum_coo() -> Vec<f32> {
    let mut s = synth::sine(550.0, 0.25);
    synth::apply_envelope(&mut s, 0.08, 0.05, 0.5, 0.1);
    s
}

// --- Skael: deep, reptilian ---

fn skael_rumble() -> Vec<f32> {
    let mut s = synth::sine(100.0, 0.5);
    synth::apply_envelope(&mut s, 0.15, 0.1, 0.7, 0.2);
    s
}

fn skael_hiss() -> Vec<f32> {
    let mut s = synth::noise(0.2);
    synth::apply_envelope(&mut s, 0.02, 0.05, 0.6, 0.08);
    synth::scale(&mut s, 0.7);
    s
}

fn skael_click() -> Vec<f32> {
    let mut s = synth::square(2000.0, 0.01, 0.5);
    synth::apply_envelope(&mut s, 0.001, 0.003, 0.5, 0.003);
    s
}

// --- Nyxal: alien, electronic ---

fn nyxal_drone() -> Vec<f32> {
    let mut s = synth::sine(150.0, 0.8);
    synth::apply_envelope(&mut s, 0.2, 0.1, 0.6, 0.3);
    s
}

fn nyxal_pulse() -> Vec<f32> {
    let mut s = synth::square(300.0, 0.05, 0.2);
    synth::apply_envelope(&mut s, 0.002, 0.01, 0.5, 0.02);
    s
}

fn nyxal_low_drone() -> Vec<f32> {
    let mut s = synth::sine(100.0, 1.0);
    synth::apply_envelope(&mut s, 0.3, 0.1, 0.4, 0.4);
    s
}
