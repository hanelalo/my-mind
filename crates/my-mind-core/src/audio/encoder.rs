use anyhow::Result;
use hound::{SampleFormat, WavSpec, WavWriter};
use std::io::Cursor;
use tracing::debug;

/// Encodes PCM i16 samples to WAV format in memory
pub struct WavEncoder;

impl WavEncoder {
    /// Encode PCM i16 samples to WAV bytes
    pub fn encode(samples: &[i16], sample_rate: u32) -> Result<Vec<u8>> {
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut cursor, spec)?;
            for &sample in samples {
                writer.write_sample(sample)?;
            }
            writer.finalize()?;
        }

        let wav_bytes = cursor.into_inner();
        debug!(
            "Encoded {} samples to {} bytes WAV",
            samples.len(),
            wav_bytes.len()
        );
        Ok(wav_bytes)
    }
}

/// Resample audio from one sample rate to another using linear interpolation.
/// For MVP this is sufficient; can upgrade to rubato for higher quality later.
pub fn resample_linear(samples: &[i16], from_rate: u32, to_rate: u32) -> Vec<i16> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = to_rate as f64 / from_rate as f64;
    let output_len = (samples.len() as f64 * ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 / ratio;
        let idx_floor = src_idx.floor() as usize;
        let frac = src_idx - idx_floor as f64;

        let sample = if idx_floor + 1 < samples.len() {
            let s0 = samples[idx_floor] as f64;
            let s1 = samples[idx_floor + 1] as f64;
            (s0 + frac * (s1 - s0)) as i16
        } else if idx_floor < samples.len() {
            samples[idx_floor]
        } else {
            0
        };

        output.push(sample);
    }

    output
}
