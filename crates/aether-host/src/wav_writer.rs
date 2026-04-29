//! WavWriter background thread — drains a lock-free ring buffer and writes
//! 16-bit PCM mono WAV to disk using `hound`.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};

use hound::{SampleFormat, WavSpec, WavWriter as HoundWriter};
use ringbuf::{traits::Consumer, HeapCons};

pub struct WavWriterThread {
    stop_flag: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl WavWriterThread {
    /// Spawn the background writer thread.
    pub fn spawn(consumer: HeapCons<f32>, path: String, sample_rate: u32) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&stop_flag);
        let handle = thread::spawn(move || {
            writer_loop(consumer, path, sample_rate, flag_clone);
        });
        Self { stop_flag, thread: Some(handle) }
    }

    /// Signal the writer to stop, wait for it to finish.
    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::Release);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

fn writer_loop(
    mut consumer: HeapCons<f32>,
    path: String,
    sample_rate: u32,
    stop_flag: Arc<AtomicBool>,
) {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = match HoundWriter::create(&path, spec) {
        Ok(w) => w,
        Err(e) => { eprintln!("WavWriter: failed to create '{}': {e}", path); return; }
    };

    let mut scratch = Vec::<f32>::with_capacity(4096);

    loop {
        while let Some(s) = consumer.try_pop() { scratch.push(s); }
        for &s in &scratch {
            let v = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
            if let Err(e) = writer.write_sample(v) {
                eprintln!("WavWriter: write error: {e}");
            }
        }
        scratch.clear();

        if stop_flag.load(Ordering::Acquire) {
            // Final drain
            while let Some(s) = consumer.try_pop() { scratch.push(s); }
            for &s in &scratch {
                let v = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
                let _ = writer.write_sample(v);
            }
            break;
        }
        thread::sleep(std::time::Duration::from_millis(5));
    }

    if let Err(e) = writer.finalize() {
        eprintln!("WavWriter: finalize error: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::io::Cursor;

    // Property 5: WAV serialization round-trip
    // **Validates: Requirements 2.7**
    proptest! {
        #[test]
        fn prop_wav_round_trip(
            samples in prop::collection::vec(
                prop::num::f32::NORMAL | prop::num::f32::ZERO,
                64..=4096
            ).prop_filter("values in [-1.0, 1.0]", |v| {
                v.iter().all(|&s| s >= -1.0 && s <= 1.0)
            })
        ) {
            // Write samples to an in-memory WAV buffer
            let spec = WavSpec {
                channels: 1,
                sample_rate: 48000,
                bits_per_sample: 16,
                sample_format: SampleFormat::Int,
            };

            let mut wav_buffer = Cursor::new(Vec::new());
            {
                let mut writer = HoundWriter::new(&mut wav_buffer, spec).unwrap();
                for &sample in &samples {
                    let quantized = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                    writer.write_sample(quantized).unwrap();
                }
                writer.finalize().unwrap();
            }

            // Read back the samples
            wav_buffer.set_position(0);
            let mut reader = hound::WavReader::new(wav_buffer).unwrap();
            let decoded: Vec<f32> = reader
                .samples::<i16>()
                .map(|s| s.unwrap() as f32 / 32767.0)
                .collect();

            // Assert length matches
            prop_assert_eq!(decoded.len(), samples.len());

            // Assert each sample is within quantization error
            let max_error = 1.0 / 32767.0;
            for (i, (&original, &decoded_val)) in samples.iter().zip(decoded.iter()).enumerate() {
                let error = (original - decoded_val).abs();
                prop_assert!(
                    error <= max_error,
                    "Sample {} failed: original={}, decoded={}, error={} > {}",
                    i, original, decoded_val, error, max_error
                );
            }
        }
    }
}
