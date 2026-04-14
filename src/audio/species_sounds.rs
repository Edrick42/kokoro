//! Species-specific procedural vocalizations — retro chiptune style.
//!
//! Each species has 3 sounds (happy, hungry, sleepy) generated from pure math.
//! No .ogg files needed. The synth module creates waveforms, this module
//! shapes them into recognizable creature voices.
//!
//! ## Species voice design
//!
//! - **Moluun** (mammal): warm triangle waves, purring, gentle hums
//! - **Pylum** (bird): sharp square chirps, fast trills, staccato
//! - **Skael** (reptile): low sine rumbles, dry noise hisses, sparse clicks
//! - **Nyxal** (cephalopod): eerie sine drones, bubbly pops, alien warbles

use crate::genome::Species;
use super::synth::{self, SAMPLE_RATE};

/// Generates a happy vocalization for the given species.
pub fn vocalize_happy(species: &Species) -> Vec<f32> {
    match species {
        // Moluun: warm purr — triangle wave layered with a gentle hum
        Species::Moluun => {
            let mut purr = synth::triangle(120.0, 0.4);
            synth::apply_envelope(&mut purr, 0.05, 0.1, 0.6, 0.15);
            let mut hum = synth::sine(180.0, 0.3);
            synth::apply_envelope(&mut hum, 0.08, 0.05, 0.3, 0.1);
            synth::scale(&mut hum, 0.4);
            synth::mix_into(&mut purr, &hum);
            purr
        }
        // Pylum: chirp-trill — two fast square chirps + ascending sine
        Species::Pylum => {
            let mut out = Vec::new();
            // First chirp
            let mut c1 = synth::square(1400.0, 0.06, 0.3);
            synth::apply_envelope(&mut c1, 0.005, 0.02, 0.5, 0.02);
            out.extend(&c1);
            // Tiny pause
            out.extend(std::iter::repeat(0.0f32).take((SAMPLE_RATE as f32 * 0.03) as usize));
            // Second chirp (higher)
            let mut c2 = synth::square(1800.0, 0.05, 0.25);
            synth::apply_envelope(&mut c2, 0.005, 0.02, 0.5, 0.02);
            out.extend(&c2);
            // Ascending trill
            out.extend(std::iter::repeat(0.0f32).take((SAMPLE_RATE as f32 * 0.02) as usize));
            let mut trill = synth::sine_sweep(1200.0, 2000.0, 0.12);
            synth::apply_envelope(&mut trill, 0.01, 0.03, 0.4, 0.04);
            out.extend(&trill);
            out
        }
        // Skael: rumble-purr — deep sine with subtle vibrato
        Species::Skael => {
            let mut rumble = synth::sine(65.0, 0.5);
            // Add vibrato by layering a slightly detuned tone
            let mut vibrato = synth::sine(68.0, 0.5);
            synth::scale(&mut vibrato, 0.5);
            synth::mix_into(&mut rumble, &vibrato);
            synth::apply_envelope(&mut rumble, 0.1, 0.1, 0.5, 0.2);
            rumble
        }
        // Nyxal: alien warble — sine sweep with bubble overlay
        Species::Nyxal => {
            let mut drone = synth::sine_sweep(200.0, 350.0, 0.4);
            synth::apply_envelope(&mut drone, 0.08, 0.1, 0.5, 0.12);
            // Bubble pops (short high blips)
            let mut pop = synth::sine(900.0, 0.03);
            synth::apply_envelope(&mut pop, 0.002, 0.01, 0.3, 0.01);
            // Place pops at different positions
            let pop_offsets = [0.1, 0.2, 0.32];
            for offset in pop_offsets {
                let start = (SAMPLE_RATE as f32 * offset) as usize;
                for (i, &s) in pop.iter().enumerate() {
                    if start + i < drone.len() {
                        drone[start + i] = (drone[start + i] + s * 0.6).clamp(-1.0, 1.0);
                    }
                }
            }
            drone
        }
    }
}

/// Generates a hungry vocalization for the given species.
pub fn vocalize_hungry(species: &Species) -> Vec<f32> {
    match species {
        // Moluun: soft descending whine
        Species::Moluun => {
            let mut whine = synth::sine_sweep(400.0, 200.0, 0.35);
            synth::apply_envelope(&mut whine, 0.03, 0.1, 0.5, 0.12);
            // Add a tiny chirp at the end (pleading)
            let pause_len = (SAMPLE_RATE as f32 * 0.05) as usize;
            whine.extend(std::iter::repeat(0.0f32).take(pause_len));
            let mut chirp = synth::triangle(350.0, 0.08);
            synth::apply_envelope(&mut chirp, 0.01, 0.02, 0.4, 0.03);
            whine.extend(&chirp);
            whine
        }
        // Pylum: repeated short calls (insistent chirp-chirp)
        Species::Pylum => {
            let mut out = Vec::new();
            for i in 0..3 {
                let freq = 1000.0 + i as f32 * 100.0; // slightly ascending
                let mut chirp = synth::square(freq, 0.07, 0.4);
                synth::apply_envelope(&mut chirp, 0.005, 0.02, 0.5, 0.02);
                out.extend(&chirp);
                out.extend(std::iter::repeat(0.0f32).take((SAMPLE_RATE as f32 * 0.06) as usize));
            }
            out
        }
        // Skael: dry hiss + low growl
        Species::Skael => {
            let mut hiss = synth::noise(0.2);
            synth::apply_envelope(&mut hiss, 0.02, 0.05, 0.3, 0.1);
            synth::scale(&mut hiss, 0.5);
            // Low growl underneath
            let mut growl = synth::sine(50.0, 0.3);
            synth::apply_envelope(&mut growl, 0.05, 0.08, 0.4, 0.12);
            synth::scale(&mut growl, 0.6);
            // Combine: hiss first, growl overlaps
            let mut out = hiss;
            let growl_start = (SAMPLE_RATE as f32 * 0.08) as usize;
            for (i, &s) in growl.iter().enumerate() {
                let idx = growl_start + i;
                if idx < out.len() {
                    out[idx] = (out[idx] + s).clamp(-1.0, 1.0);
                } else {
                    out.push(s);
                }
            }
            out
        }
        // Nyxal: sonar click train (rapid clicking)
        Species::Nyxal => {
            let mut out = Vec::new();
            for _ in 0..5 {
                let mut click = synth::square(2000.0, 0.015, 0.5);
                synth::apply_envelope(&mut click, 0.001, 0.005, 0.3, 0.005);
                out.extend(&click);
                out.extend(std::iter::repeat(0.0f32).take((SAMPLE_RATE as f32 * 0.04) as usize));
            }
            out
        }
    }
}

/// Generates a sleepy vocalization for the given species.
pub fn vocalize_sleepy(species: &Species) -> Vec<f32> {
    match species {
        // Moluun: long slow hum that fades
        Species::Moluun => {
            let mut hum = synth::triangle(100.0, 0.6);
            synth::apply_envelope(&mut hum, 0.1, 0.15, 0.3, 0.25);
            hum
        }
        // Pylum: single quiet peep
        Species::Pylum => {
            let mut peep = synth::sine(800.0, 0.15);
            synth::apply_envelope(&mut peep, 0.01, 0.05, 0.2, 0.08);
            peep
        }
        // Skael: single low click (reptile settling)
        Species::Skael => {
            let mut click = synth::square(200.0, 0.05, 0.5);
            synth::apply_envelope(&mut click, 0.002, 0.015, 0.2, 0.02);
            click
        }
        // Nyxal: deep slow drone (ocean at night)
        Species::Nyxal => {
            let mut drone = synth::sine(55.0, 0.8);
            synth::apply_envelope(&mut drone, 0.15, 0.2, 0.25, 0.3);
            // Ghostly overtone
            let mut overtone = synth::sine(110.0, 0.6);
            synth::apply_envelope(&mut overtone, 0.2, 0.1, 0.1, 0.2);
            synth::scale(&mut overtone, 0.25);
            synth::mix_into(&mut drone, &overtone);
            drone
        }
    }
}

/// Generates a heartbeat thump — universal, not species-specific.
/// A short low-frequency pulse like a kick drum.
pub fn heartbeat_thump() -> Vec<f32> {
    let mut thump = synth::sine_sweep(80.0, 30.0, 0.12);
    synth::apply_envelope(&mut thump, 0.005, 0.03, 0.3, 0.06);
    thump
}

/// Generates an eating crunch — noise burst with low thump.
pub fn eating_sound() -> Vec<f32> {
    let mut crunch = synth::noise(0.06);
    synth::apply_envelope(&mut crunch, 0.002, 0.02, 0.4, 0.02);
    synth::scale(&mut crunch, 0.5);
    let mut thump = synth::sine(100.0, 0.08);
    synth::apply_envelope(&mut thump, 0.005, 0.02, 0.3, 0.03);
    synth::mix_into(&mut crunch, &thump);
    crunch
}
