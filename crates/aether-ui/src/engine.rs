//! Audio engine bridge — connects the Iced UI to the AetherDSP real-time engine.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use aether_core::arena::NodeId;
use aether_core::param::Param;
use aether_core::scheduler::Scheduler;
use aether_midi::tuning::TuningTable;
use aether_nodes::{envelope::AdsrEnvelope, gain::Gain, oscillator::Oscillator};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::project::{TrackId, TuningSystem};

// ── Per-track node IDs ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
struct TrackNodes {
    oscillator: NodeId,
    envelope: NodeId,
    gain: NodeId,
}

// ── Playback commands ─────────────────────────────────────────────────────────

pub enum PlaybackCmd {
    Play,
    Stop,
    SetNotes(Vec<ScheduledNote>),
    SetBpm(f32),
}

/// A single note event for the playback scheduler.
#[derive(Clone)]
pub struct ScheduledNote {
    pub track_id: TrackId,
    pub pitch: u8,
    pub start_beat: f32,
    pub length_beats: f32,
}

// ── Engine ────────────────────────────────────────────────────────────────────

pub struct Engine {
    scheduler: Arc<Mutex<Scheduler>>,
    sample_rate: f32,
    tracks: HashMap<TrackId, TrackNodes>,
    _stream: cpal::Stream,
    playback_tx: std::sync::mpsc::SyncSender<PlaybackCmd>,
}

impl Engine {
    pub fn start() -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No audio output device found"))?;
        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0 as f32;

        let scheduler = Arc::new(Mutex::new(Scheduler::new(sample_rate)));

        // Build CPAL stream
        let stream = {
            let scheduler = Arc::clone(&scheduler);
            let channels = config.channels() as usize;
            let mut fallback = vec![0.0f32; aether_core::BUFFER_SIZE * 2];
            device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _| {
                    let frames = data.len() / channels;
                    let mut buf = [0.0f32; aether_core::BUFFER_SIZE * 2];
                    let mut offset = 0;
                    while offset < frames {
                        let chunk = (frames - offset).min(aether_core::BUFFER_SIZE);
                        match scheduler.try_lock() {
                            Ok(mut sched) => {
                                sched.process_block_simple(&mut buf[..chunk * 2]);
                                fallback[..chunk * 2].copy_from_slice(&buf[..chunk * 2]);
                            }
                            Err(_) => {
                                buf[..chunk * 2].copy_from_slice(&fallback[..chunk * 2]);
                            }
                        }
                        for i in 0..chunk {
                            let ch0 = buf[i * 2];
                            let ch1 = buf[i * 2 + 1];
                            for ch in 0..channels {
                                let idx = (offset + i) * channels + ch;
                                if idx < data.len() {
                                    data[idx] = if ch == 0 { ch0 } else { ch1 };
                                }
                            }
                        }
                        offset += chunk;
                    }
                },
                |err| eprintln!("[engine] audio error: {err}"),
                None,
            )?
        };
        stream.play()?;

        // Spawn playback scheduler thread
        let (tx, rx) = std::sync::mpsc::sync_channel::<PlaybackCmd>(64);
        let sched_clone = Arc::clone(&scheduler);
        std::thread::spawn(move || {
            playback_thread(rx, sched_clone);
        });

        println!("[engine] started at {sample_rate} Hz");

        Ok(Self {
            scheduler,
            sample_rate,
            tracks: HashMap::new(),
            _stream: stream,
            playback_tx: tx,
        })
    }

    // ── Playback ──────────────────────────────────────────────────────────────

    /// Start playback — sends notes + bpm to the playback thread.
    pub fn start_playback(&self, notes: Vec<ScheduledNote>, bpm: f32) {
        let _ = self.playback_tx.try_send(PlaybackCmd::SetNotes(notes));
        let _ = self.playback_tx.try_send(PlaybackCmd::SetBpm(bpm));
        let _ = self.playback_tx.try_send(PlaybackCmd::Play);
    }

    /// Stop playback.
    pub fn stop_playback(&self) {
        let _ = self.playback_tx.try_send(PlaybackCmd::Stop);
    }

    // ── Track management ──────────────────────────────────────────────────────

    pub fn add_track(&mut self, id: TrackId, tuning: TuningSystem, volume: f32) {
        let mut sched = self.scheduler.lock().unwrap();
        let table = build_tuning_table(tuning);

        let mut osc = Oscillator::new();
        osc.set_tuning(table.frequencies.try_into().unwrap_or([440.0f32; 128]));
        let osc_id = match sched.graph.add_node(Box::new(osc)) {
            Some(id) => id,
            None => { eprintln!("[engine] failed to add oscillator"); return; }
        };
        {
            let rec = sched.graph.arena.get_mut(osc_id).unwrap();
            rec.params.add(440.0); // 0: frequency
            rec.params.add(0.3);   // 1: amplitude
            rec.params.add(0.0);   // 2: waveform (sine)
            rec.params.add(-1.0);  // 3: midi_note (-1 = use freq)
        }

        let env_id = match sched.graph.add_node(Box::new(AdsrEnvelope::new())) {
            Some(id) => id,
            None => { eprintln!("[engine] failed to add envelope"); return; }
        };
        {
            let rec = sched.graph.arena.get_mut(env_id).unwrap();
            rec.params.add(0.005); // 0: attack
            rec.params.add(0.1);   // 1: decay
            rec.params.add(0.7);   // 2: sustain
            rec.params.add(0.15);  // 3: release
            rec.params.add(0.0);   // 4: gate
        }

        let gain_id = match sched.graph.add_node(Box::new(Gain)) {
            Some(id) => id,
            None => { eprintln!("[engine] failed to add gain"); return; }
        };
        {
            let rec = sched.graph.arena.get_mut(gain_id).unwrap();
            rec.params.add(volume); // 0: gain
        }

        sched.graph.connect(osc_id, env_id, 0);
        sched.graph.connect(env_id, gain_id, 0);
        sched.graph.set_output_node(gain_id);

        self.tracks.insert(id, TrackNodes { oscillator: osc_id, envelope: env_id, gain: gain_id });
        println!("[engine] track {} added", id.0);
    }

    pub fn remove_track(&mut self, id: TrackId) {
        if let Some(nodes) = self.tracks.remove(&id) {
            let mut sched = self.scheduler.lock().unwrap();
            sched.graph.remove_node(nodes.gain);
            sched.graph.remove_node(nodes.envelope);
            sched.graph.remove_node(nodes.oscillator);
        }
    }

    // ── Note control ──────────────────────────────────────────────────────────

    pub fn note_on(&mut self, track_id: TrackId, midi_note: u8) {
        let nodes = match self.tracks.get(&track_id) { Some(n) => *n, None => return };
        let mut sched = self.scheduler.lock().unwrap();
        if let Some(rec) = sched.graph.arena.get_mut(nodes.oscillator) {
            rec.params.params[3] = Param::new(midi_note as f32);
        }
        if let Some(rec) = sched.graph.arena.get_mut(nodes.envelope) {
            rec.params.params[4] = Param::new(1.0);
        }
    }

    pub fn note_off(&mut self, track_id: TrackId) {
        let nodes = match self.tracks.get(&track_id) { Some(n) => *n, None => return };
        let mut sched = self.scheduler.lock().unwrap();
        if let Some(rec) = sched.graph.arena.get_mut(nodes.envelope) {
            rec.params.params[4] = Param::new(0.0);
        }
    }

    // ── Parameter control ─────────────────────────────────────────────────────

    pub fn set_track_volume(&mut self, track_id: TrackId, volume: f32) {
        let nodes = match self.tracks.get(&track_id) { Some(n) => *n, None => return };
        let ramp = (self.sample_rate * 0.010) as u32;
        let mut sched = self.scheduler.lock().unwrap();
        if let Some(rec) = sched.graph.arena.get_mut(nodes.gain) {
            rec.params.params[0].set_target(volume.clamp(0.0, 1.0), ramp);
        }
    }

    pub fn set_track_mute(&mut self, track_id: TrackId, muted: bool) {
        self.set_track_volume(track_id, if muted { 0.0 } else { 0.75 });
    }

    pub fn set_track_tuning(&mut self, track_id: TrackId, tuning: TuningSystem) {
        let nodes = match self.tracks.get(&track_id) { Some(n) => *n, None => return };
        let table = build_tuning_table(tuning);
        let freqs: [f32; 128] = table.frequencies.try_into().unwrap_or([440.0f32; 128]);
        let mut sched = self.scheduler.lock().unwrap();
        if let Some(rec) = sched.graph.arena.get_mut(nodes.oscillator) {
            if rec.processor.type_name() == "Oscillator" {
                let ptr = &mut *rec.processor as *mut dyn aether_core::node::DspNode;
                // SAFETY: verified via type_name, exclusive ownership through the mutex
                let osc = unsafe { &mut *(ptr as *mut Oscillator) };
                osc.set_tuning(freqs);
            }
        }
    }

    pub fn set_mute(&self, muted: bool) {
        let mut sched = self.scheduler.lock().unwrap();
        sched.muted = muted;
    }

    pub fn sample_rate(&self) -> f32 { self.sample_rate }
}

// ── Playback thread ───────────────────────────────────────────────────────────

fn playback_thread(
    rx: std::sync::mpsc::Receiver<PlaybackCmd>,
    scheduler: Arc<Mutex<Scheduler>>,
) {
    let mut notes: Vec<ScheduledNote> = Vec::new();
    let mut bpm: f32 = 120.0;
    let mut playing = false;
    let mut start_time: Option<std::time::Instant> = None;
    let mut triggered: Vec<bool> = Vec::new();
    let mut released: Vec<bool> = Vec::new();

    loop {
        // Drain commands without blocking
        loop {
            match rx.try_recv() {
                Ok(cmd) => match cmd {
                    PlaybackCmd::Play => {
                        playing = true;
                        start_time = Some(std::time::Instant::now());
                        triggered = vec![false; notes.len()];
                        released = vec![false; notes.len()];
                        // Unmute
                        if let Ok(mut sched) = scheduler.lock() {
                            sched.muted = false;
                        }
                    }
                    PlaybackCmd::Stop => {
                        playing = false;
                        start_time = None;
                        // Close all gates + mute
                        if let Ok(mut sched) = scheduler.lock() {
                            for node_id in sched.graph.execution_order.clone() {
                                if let Some(rec) = sched.graph.arena.get_mut(node_id) {
                                    if rec.processor.type_name() == "AdsrEnvelope"
                                        && rec.params.count >= 5
                                    {
                                        rec.params.params[4] = Param::new(0.0);
                                    }
                                }
                            }
                            sched.muted = true;
                        }
                    }
                    PlaybackCmd::SetNotes(n) => {
                        notes = n;
                        triggered = vec![false; notes.len()];
                        released = vec![false; notes.len()];
                    }
                    PlaybackCmd::SetBpm(b) => {
                        bpm = b.max(1.0);
                    }
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
            }
        }

        if playing {
            if let Some(start) = start_time {
                let elapsed_secs = start.elapsed().as_secs_f64();
                let beats_per_sec = bpm as f64 / 60.0;
                let current_beat = elapsed_secs * beats_per_sec;

                for i in 0..notes.len() {
                    let note = &notes[i];

                    // Note on
                    if !triggered[i] && current_beat >= note.start_beat as f64 {
                        triggered[i] = true;
                        if let Ok(mut sched) = scheduler.try_lock() {
                            for node_id in sched.graph.execution_order.clone() {
                                if let Some(rec) = sched.graph.arena.get_mut(node_id) {
                                    if rec.processor.type_name() == "Oscillator"
                                        && rec.params.count >= 4
                                    {
                                        rec.params.params[3] = Param::new(note.pitch as f32);
                                    }
                                    if rec.processor.type_name() == "AdsrEnvelope"
                                        && rec.params.count >= 5
                                    {
                                        rec.params.params[4] = Param::new(1.0);
                                    }
                                }
                            }
                        }
                    }

                    // Note off
                    let note_end = note.start_beat as f64 + note.length_beats as f64;
                    if triggered[i] && !released[i] && current_beat >= note_end {
                        released[i] = true;
                        if let Ok(mut sched) = scheduler.try_lock() {
                            for node_id in sched.graph.execution_order.clone() {
                                if let Some(rec) = sched.graph.arena.get_mut(node_id) {
                                    if rec.processor.type_name() == "AdsrEnvelope"
                                        && rec.params.count >= 5
                                    {
                                        rec.params.params[4] = Param::new(0.0);
                                    }
                                }
                            }
                        }
                    }
                }

                // Auto-loop when all notes have finished
                let all_done = !notes.is_empty()
                    && triggered.iter().all(|&t| t)
                    && released.iter().all(|&r| r);
                if all_done {
                    start_time = Some(std::time::Instant::now());
                    triggered = vec![false; notes.len()];
                    released = vec![false; notes.len()];
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

// ── Tuning table builder ──────────────────────────────────────────────────────

pub fn build_tuning_table(tuning: TuningSystem) -> TuningTable {
    use aether_midi::tuning::TuningTable as MidiTuning;
    match tuning {
        TuningSystem::EthiopianTizitaMajor => MidiTuning::ethiopian_tizita(440.0),
        TuningSystem::EthiopianTizitaMinor => MidiTuning::ethiopian_tizita_minor(440.0),
        TuningSystem::EthiopianBatiMinor   => MidiTuning::ethiopian_bati(440.0),
        TuningSystem::EthiopianBatiMajor   => MidiTuning::ethiopian_bati_major(440.0),
        TuningSystem::EthiopianAmbassel    => MidiTuning::ethiopian_ambassel(440.0),
        TuningSystem::EthiopianAnchihoye   => MidiTuning::ethiopian_anchihoye(440.0),
        TuningSystem::ArabicRast           => MidiTuning::arabic_maqam_rast(440.0),
        TuningSystem::ArabicBayati         => MidiTuning::arabic_maqam_bayati(440.0),
        TuningSystem::ArabicHijaz          => MidiTuning::arabic_maqam_hijaz(440.0),
        TuningSystem::IndianYaman          => MidiTuning::indian_raga_yaman(440.0),
        TuningSystem::GamelanSlendro       => MidiTuning::gamelan_slendro(440.0),
        TuningSystem::GamelanPelog         => MidiTuning::gamelan_pelog(440.0),
        TuningSystem::EqualTemperament     => MidiTuning::equal_temperament(440.0),
        TuningSystem::JustIntonation       => MidiTuning::just_intonation(440.0),
    }
}
