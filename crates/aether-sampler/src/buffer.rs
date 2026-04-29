//! Audio buffer loading and storage.

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BufferError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("WAV decode error: {0}")]
    Wav(#[from] hound::Error),
    #[error("Unsupported format: {0}")]
    Format(String),
}

/// A decoded audio buffer stored as interleaved f32 samples.
#[derive(Debug, Clone)]
pub struct SampleBuffer {
    /// Interleaved samples (mono or stereo).
    pub samples: Vec<f32>,
    /// Number of channels (1=mono, 2=stereo).
    pub channels: u16,
    /// Original sample rate in Hz.
    pub sample_rate: u32,
    /// Total number of frames (samples / channels).
    pub frames: usize,
    /// File path this was loaded from.
    pub path: String,
}

impl SampleBuffer {
    /// Load a WAV file into memory as f32 samples.
    pub fn load_wav(path: &Path) -> Result<Self, BufferError> {
        let mut reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        let channels = spec.channels;
        let sample_rate = spec.sample_rate;

        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => {
                reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect()
            }
            hound::SampleFormat::Int => {
                let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
                match spec.bits_per_sample {
                    16 => reader.samples::<i16>()
                        .map(|s| s.unwrap_or(0) as f32 / max)
                        .collect(),
                    24 | 32 => reader.samples::<i32>()
                        .map(|s| s.unwrap_or(0) as f32 / max)
                        .collect(),
                    _ => return Err(BufferError::Format(
                        format!("Unsupported bit depth: {}", spec.bits_per_sample)
                    )),
                }
            }
        };

        let frames = samples.len() / channels as usize;
        Ok(Self {
            samples,
            channels,
            sample_rate,
            frames,
            path: path.to_string_lossy().into_owned(),
        })
    }

    /// Get a mono sample at a given frame position (linear interpolation).
    /// If stereo, returns the average of both channels.
    #[inline]
    pub fn sample_at(&self, frame: f64) -> f32 {
        let frame_floor = frame as usize;
        let frac = (frame - frame as f64) as f32;

        if frame_floor + 1 >= self.frames {
            return self.frame_mono(self.frames.saturating_sub(1));
        }

        let s0 = self.frame_mono(frame_floor);
        let s1 = self.frame_mono(frame_floor + 1);
        s0 + (s1 - s0) * frac
    }

    /// Get mono value at an exact frame index.
    #[inline]
    fn frame_mono(&self, frame: usize) -> f32 {
        if self.channels == 1 {
            self.samples.get(frame).copied().unwrap_or(0.0)
        } else {
            let i = frame * self.channels as usize;
            let l = self.samples.get(i).copied().unwrap_or(0.0);
            let r = self.samples.get(i + 1).copied().unwrap_or(0.0);
            (l + r) * 0.5
        }
    }

    /// Load a WAV from any `Read + Seek` source (e.g. an in-memory `Cursor<&[u8]>`).
    /// This is used by the CLAP plugin to decode audio that was embedded at build time.
    pub fn load_wav_reader<R: std::io::Read + std::io::Seek>(
        reader: R,
    ) -> Result<Self, BufferError> {
        let mut wav = hound::WavReader::new(reader)?;
        let spec = wav.spec();
        let channels = spec.channels;
        let sample_rate = spec.sample_rate;

        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => {
                wav.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect()
            }
            hound::SampleFormat::Int => {
                let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
                match spec.bits_per_sample {
                    16 => wav
                        .samples::<i16>()
                        .map(|s| s.unwrap_or(0) as f32 / max)
                        .collect(),
                    24 | 32 => wav
                        .samples::<i32>()
                        .map(|s| s.unwrap_or(0) as f32 / max)
                        .collect(),
                    _ => {
                        return Err(BufferError::Format(format!(
                            "Unsupported bit depth: {}",
                            spec.bits_per_sample
                        )))
                    }
                }
            }
        };

        let frames = samples.len() / channels as usize;
        Ok(Self {
            samples,
            channels,
            sample_rate,
            frames,
            path: "<embedded>".into(),
        })
    }

    /// Duration in seconds.
    pub fn duration_secs(&self) -> f32 {
        self.frames as f32 / self.sample_rate as f32
    }
}
