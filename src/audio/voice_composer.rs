//! Voice composer — builds unique vocalizations from micro-sound atoms.
//!
//! Scaffold — not yet connected to the audio pipeline.
//!
//! Instead of playing one fixed .ogg per vocalization, the composer
//! combines small sound fragments ("atoms") into a unique voice each time.
//!
//! ## How it works
//!
//! 1. Each species has a **recipe** — which atoms to combine and how
//! 2. The genome applies **subtle variation** (pitch, timing, volume)
//! 3. Each call to `compose()` produces a slightly different result
//!
//! ## Design principle: same species = recognizable, different individual = subtle variation
//!
//! Like real dogs: all dogs bark, but each dog's bark is slightly different.
//! The species defines the STRUCTURE (which atoms, what order, what layers).
//! The genome defines the FLAVOR (pitch offset, volume, slight timing shifts).

use rand::Rng;

use crate::genome::Genome;

use super::synth::SAMPLE_RATE;

/// A loaded sound atom — a short audio fragment ready for composition.
#[derive(Clone)]
pub struct SoundAtom {
    /// Raw f32 samples (mono, SAMPLE_RATE Hz).
    pub samples: Vec<f32>,
    pub name: String,
}

/// Recipe for how to compose a vocalization.
/// Each species defines recipes for each mood/sound type.
#[derive(Clone)]
pub struct VoiceRecipe {
    /// Steps executed in sequence to build the final sound.
    pub steps: Vec<ComposeStep>,
}

#[derive(Clone)]
pub enum ComposeStep {
    /// Play this atom (by name).
    Play(String),
    /// Layer this atom ON TOP of the previous (mix, not sequence).
    Layer(String),
    /// Silence gap in seconds.
    Pause(f32),
}

/// Composes a vocalization from atoms using a recipe + genome variation.
///
/// Returns raw f32 samples ready for WAV encoding.
pub fn compose(
    recipe: &VoiceRecipe,
    atoms: &[SoundAtom],
    genome: &Genome,
) -> Vec<f32> {
    let mut rng = rand::rng();
    let mut output: Vec<f32> = Vec::new();

    // Genome-driven variation (subtle — same species stays recognizable)
    // hue → pitch variation: ±5% (not ±20% which would sound alien)
    let pitch_factor = 1.0 + (genome.hue / 360.0 - 0.5) * 0.1; // 0.95 to 1.05
    // resilience → volume variation: ±10%
    let volume_factor = 0.9 + genome.resilience * 0.2; // 0.9 to 1.1
    // Add tiny random timing jitter (±10ms) for naturalness
    let jitter_samples = (SAMPLE_RATE as f32 * 0.01) as usize;

    for step in &recipe.steps {
        match step {
            ComposeStep::Play(name) => {
                if let Some(atom) = atoms.iter().find(|a| a.name == *name) {
                    // Apply pitch by resampling
                    let pitched = resample(&atom.samples, pitch_factor);
                    // Apply volume
                    let scaled: Vec<f32> = pitched.iter().map(|s| s * volume_factor).collect();
                    // Add tiny random jitter (silence gap 0-10ms)
                    let jitter = rng.random_range(0..jitter_samples);
                    output.extend(std::iter::repeat(0.0f32).take(jitter));
                    output.extend(scaled);
                }
            }
            ComposeStep::Layer(name) => {
                if let Some(atom) = atoms.iter().find(|a| a.name == *name) {
                    let pitched = resample(&atom.samples, pitch_factor);
                    // Mix on top of the tail of the current output
                    let start = output.len().saturating_sub(pitched.len());
                    for (i, &sample) in pitched.iter().enumerate() {
                        let idx = start + i;
                        if idx < output.len() {
                            output[idx] = (output[idx] + sample * volume_factor).clamp(-1.0, 1.0);
                        } else {
                            output.push(sample * volume_factor);
                        }
                    }
                }
            }
            ComposeStep::Pause(secs) => {
                let pause_samples = (SAMPLE_RATE as f32 * secs) as usize;
                let jitter = rng.random_range(0..jitter_samples);
                output.extend(std::iter::repeat(0.0f32).take(pause_samples + jitter));
            }
        }
    }

    // Fade out last 30ms to avoid clicks
    let fade_len = (SAMPLE_RATE as f32 * 0.03) as usize;
    let len = output.len();
    if len > fade_len {
        for i in 0..fade_len {
            let t = 1.0 - (i as f32 / fade_len as f32);
            output[len - fade_len + i] *= t;
        }
    }

    // Normalize to prevent clipping
    let peak = output.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak > 0.95 {
        let norm = 0.9 / peak;
        for s in &mut output {
            *s *= norm;
        }
    }

    output
}

/// Simple pitch shift by resampling (linear interpolation).
/// factor > 1.0 = higher pitch (shorter), < 1.0 = lower pitch (longer).
fn resample(samples: &[f32], factor: f32) -> Vec<f32> {
    if (factor - 1.0).abs() < 0.001 {
        return samples.to_vec();
    }
    let new_len = (samples.len() as f32 / factor) as usize;
    let mut result = Vec::with_capacity(new_len);
    for i in 0..new_len {
        let src_pos = i as f32 * factor;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f32;
        let a = samples.get(idx).copied().unwrap_or(0.0);
        let b = samples.get(idx + 1).copied().unwrap_or(a);
        result.push(a + (b - a) * frac);
    }
    result
}

/// Loads a .ogg/.wav file from assets/ into a SoundAtom (f32 samples).
/// Returns None if the file doesn't exist.
#[allow(dead_code)]
pub fn load_atom_from_wav(path: &str) -> Option<SoundAtom> {
    let full_path = format!("assets/{path}");
    let data = std::fs::read(&full_path).ok()?;

    // Parse WAV: skip 44-byte header, read 16-bit PCM
    if data.len() < 44 { return None; }
    if &data[0..4] != b"RIFF" { return None; }

    let num_channels = u16::from_le_bytes([data[22], data[23]]) as usize;
    let bits = u16::from_le_bytes([data[34], data[35]]);
    if bits != 16 { return None; } // only 16-bit PCM

    let data_start = 44;
    let mut samples = Vec::new();
    let mut i = data_start;
    while i + 1 < data.len() {
        let sample = i16::from_le_bytes([data[i], data[i + 1]]);
        samples.push(sample as f32 / 32768.0);
        i += 2 * num_channels; // skip extra channels if stereo
    }

    let name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Some(SoundAtom { samples, name })
}

// ===================================================================
// Species voice recipes
// ===================================================================

/// Returns the voice recipe for a species × mood combination.
///
/// The recipe defines the STRUCTURE of the sound.
/// The genome (applied during compose()) defines the subtle individual variation.
pub fn species_recipe(species: &crate::genome::Species, mood: &super::MoodSound) -> VoiceRecipe {
    use crate::genome::Species;
    use super::MoodSound;
    use ComposeStep::*;

    match (species, mood) {
        // MOLUUN — warm mammalian: purr-based, layered
        (Species::Moluun, MoodSound::Happy) => VoiceRecipe {
            steps: vec![Play("purr_low".into()), Layer("hum_warm".into())],
        },
        (Species::Moluun, MoodSound::Hungry) => VoiceRecipe {
            steps: vec![Play("whine_soft".into()), Pause(0.05), Play("chirp_short".into())],
        },
        (Species::Moluun, MoodSound::Sleepy) => VoiceRecipe {
            steps: vec![Play("hum_warm".into()), Pause(0.1), Play("purr_low".into())],
        },

        // PYLUM — bright avian: chirp-based, sequential
        (Species::Pylum, MoodSound::Happy) => VoiceRecipe {
            steps: vec![Play("chirp_short".into()), Pause(0.03), Play("trill_fast".into())],
        },
        (Species::Pylum, MoodSound::Hungry) => VoiceRecipe {
            steps: vec![Play("chirp_long".into()), Pause(0.05), Play("chirp_short".into())],
        },
        (Species::Pylum, MoodSound::Sleepy) => VoiceRecipe {
            steps: vec![Play("hum_warm".into())],
        },

        // SKAEL — deep reptilian: rumble-based, sparse
        (Species::Skael, MoodSound::Happy) => VoiceRecipe {
            steps: vec![Play("growl_soft".into()), Layer("purr_low".into())],
        },
        (Species::Skael, MoodSound::Hungry) => VoiceRecipe {
            steps: vec![Play("hiss_dry".into()), Pause(0.08), Play("growl_soft".into())],
        },
        (Species::Skael, MoodSound::Sleepy) => VoiceRecipe {
            steps: vec![Play("click_sharp".into())],
        },

        // NYXAL — alien oceanic: drone-based, layered
        (Species::Nyxal, MoodSound::Happy) => VoiceRecipe {
            steps: vec![Play("drone_deep".into()), Layer("bubble_pop".into())],
        },
        (Species::Nyxal, MoodSound::Hungry) => VoiceRecipe {
            steps: vec![Play("click_sharp".into()), Pause(0.03), Play("click_sharp".into()), Pause(0.03), Play("click_sharp".into())],
        },
        (Species::Nyxal, MoodSound::Sleepy) => VoiceRecipe {
            steps: vec![Play("drone_deep".into()), Layer("hum_warm".into())],
        },
    }
}
