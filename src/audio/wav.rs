//! WAV file builder — converts f32 samples into WAV byte buffers.
//!
//! Produces standard RIFF/WAVE format (PCM 16-bit mono) that rodio
//! can decode and Bevy can play as `AudioSource`.

use bevy::audio::AudioSource;

use super::synth::SAMPLE_RATE;

/// Converts f32 samples [-1.0, 1.0] into a Bevy AudioSource (WAV format).
pub fn samples_to_source(samples: &[f32]) -> AudioSource {
    let bytes = encode_wav_16bit_mono(samples, SAMPLE_RATE);
    AudioSource { bytes: bytes.into() }
}

/// Encodes samples as a standard WAV file (RIFF/WAVE PCM 16-bit mono).
fn encode_wav_16bit_mono(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let bytes_per_sample = bits_per_sample / 8;
    let num_samples = samples.len() as u32;
    let data_chunk_size = num_samples * bytes_per_sample as u32 * num_channels as u32;

    // Total file size = 4 (WAVE) + 24 (fmt chunk) + 8 (data header) + data
    let riff_chunk_size = 4 + 24 + 8 + data_chunk_size;

    let byte_rate = sample_rate * num_channels as u32 * bytes_per_sample as u32;
    let block_align = num_channels * bytes_per_sample;

    let mut buf = Vec::with_capacity(12 + 24 + 8 + data_chunk_size as usize);

    // RIFF header (12 bytes)
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&riff_chunk_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt sub-chunk (24 bytes: 8 header + 16 data)
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());               // fmt chunk data size
    buf.extend_from_slice(&1u16.to_le_bytes());                // audio format: 1 = PCM
    buf.extend_from_slice(&num_channels.to_le_bytes());        // 1 channel
    buf.extend_from_slice(&sample_rate.to_le_bytes());         // e.g. 22050
    buf.extend_from_slice(&byte_rate.to_le_bytes());           // sample_rate * block_align
    buf.extend_from_slice(&block_align.to_le_bytes());         // channels * bytes_per_sample
    buf.extend_from_slice(&bits_per_sample.to_le_bytes());     // 16

    // data sub-chunk (8 byte header + PCM data)
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_chunk_size.to_le_bytes());

    // PCM samples: f32 → i16
    for &sample in samples {
        let s = sample.clamp(-1.0, 1.0);
        let val = if s >= 0.0 {
            (s * 32767.0) as i16
        } else {
            (s * 32768.0) as i16
        };
        buf.extend_from_slice(&val.to_le_bytes());
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_is_44_bytes() {
        let samples = vec![0.0f32; 100];
        let wav = encode_wav_16bit_mono(&samples, 22050);
        // Header: 12 (RIFF) + 24 (fmt) + 8 (data header) = 44
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
        // Total: 44 header + 200 data (100 samples × 2 bytes)
        assert_eq!(wav.len(), 44 + 200);
    }

    #[test]
    fn wav_data_size_matches() {
        let samples = crate::audio::synth::sine(440.0, 0.1);
        let wav = encode_wav_16bit_mono(&samples, 22050);
        // Read data chunk size from header
        let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
        // Should equal samples.len() * 2 (16-bit = 2 bytes per sample)
        assert_eq!(data_size as usize, samples.len() * 2);
        // Total file should be header (44) + data
        assert_eq!(wav.len(), 44 + data_size as usize);
    }

    #[test]
    fn wav_riff_size_is_correct() {
        let samples = vec![0.5f32; 50];
        let wav = encode_wav_16bit_mono(&samples, 22050);
        let riff_size = u32::from_le_bytes([wav[4], wav[5], wav[6], wav[7]]);
        // RIFF size = file_size - 8 (RIFF + size field itself)
        assert_eq!(riff_size as usize, wav.len() - 8);
    }
}
