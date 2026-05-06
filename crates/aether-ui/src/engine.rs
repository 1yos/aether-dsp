//! Engine integration — starts the CPAL audio stream in the same process.

use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use aether_core::{scheduler::Scheduler, BUFFER_SIZE};
use crate::app_state::AppState;

pub fn start(state: AppState) {
    std::thread::spawn(move || {
        if let Err(e) = run_audio(state) {
            eprintln!("[engine] audio error: {e}");
        }
    });
}

fn run_audio(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or("No audio output device")?;
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;

    {
        let mut s = state.lock().unwrap();
        *s.scheduler.lock().unwrap() = Scheduler::new(sample_rate);
        s.engine_status = crate::app_state::EngineStatus::Running;
    }

    let scheduler: Arc<Mutex<Scheduler>> = {
        let s = state.lock().unwrap();
        Arc::clone(&s.scheduler)
    };

    let channels = config.channels() as usize;
    let mut fallback_buf = vec![0.0f32; BUFFER_SIZE * 2];
    let mut contention_count = 0u32;

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            let frames = data.len() / channels;
            let mut f32_buf = [0.0f32; BUFFER_SIZE * 2];
            let mut offset = 0;
            while offset < frames {
                let chunk = (frames - offset).min(BUFFER_SIZE);
                match scheduler.try_lock() {
                    Ok(mut sched) => {
                        sched.process_block_simple(&mut f32_buf[..chunk * 2]);
                        fallback_buf[..chunk * 2].copy_from_slice(&f32_buf[..chunk * 2]);
                        contention_count = 0;
                    }
                    Err(_) => {
                        contention_count += 1;
                        let fade = if contention_count <= 8 { 1.0 } else { (1.0 - ((contention_count - 8) as f32 / 8.0)).max(0.0) };
                        for (i, s) in fallback_buf[..chunk * 2].iter().enumerate() { f32_buf[i] = s * fade; }
                    }
                }
                for i in 0..chunk {
                    let ch0 = f32_buf[i * 2];
                    let ch1 = f32_buf[i * 2 + 1];
                    for ch in 0..channels {
                        let idx = (offset + i) * channels + ch;
                        if idx < data.len() { data[idx] = if ch == 0 { ch0 } else { ch1 }; }
                    }
                }
                offset += chunk;
            }
        },
        |err| eprintln!("[engine] stream error: {err}"),
        None,
    )?;

    stream.play()?;
    eprintln!("[engine] audio stream started at {sample_rate}Hz");
    loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
}
