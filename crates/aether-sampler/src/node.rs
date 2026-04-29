//! SamplerNode — integrates the sampler into the AetherDSP graph.
//!
//! This is a DspNode that:
//!   1. Receives MIDI events via a lock-free queue
//!   2. Manages polyphonic voices
//!   3. Renders audio into the output buffer

use std::sync::{Arc, Mutex};
use aether_core::{
    node::DspNode,
    param::ParamBlock,
    BUFFER_SIZE, MAX_INPUTS,
};
use crate::{
    instrument::{LoadedInstrument, RoundRobinState},
    voice::SamplerVoice,
};
use aether_midi::event::{MidiEvent, MidiEventKind};

/// A polyphonic sampler node.
pub struct SamplerNode {
    /// The loaded instrument (shared with the instrument maker UI).
    instrument: Arc<Mutex<Option<LoadedInstrument>>>,
    /// Active voices.
    voices: Vec<SamplerVoice>,
    /// Pending MIDI events (written by MIDI thread, read by audio thread).
    midi_queue: Arc<Mutex<Vec<MidiEvent>>>,
    /// Sample rate.
    sample_rate: f32,
    /// Sustain pedal state per channel.
    sustain_pedal: [bool; 16],
    /// Notes held by sustain pedal (released by key but sustained by pedal).
    sustained_notes: Vec<(u8, u8)>, // (channel, note)
    /// Round-robin state for zone selection.
    rr_state: RoundRobinState,
    /// Last loaded instrument name (to detect instrument changes).
    last_instrument_name: Option<String>,
}

impl SamplerNode {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            instrument: Arc::new(Mutex::new(None)),
            voices: Vec::with_capacity(32),
            midi_queue: Arc::new(Mutex::new(Vec::new())),
            sample_rate,
            sustain_pedal: [false; 16],
            sustained_notes: Vec::new(),
            rr_state: RoundRobinState::new(),
            last_instrument_name: None,
        }
    }

    /// Get the MIDI queue for pushing events from the MIDI thread.
    pub fn midi_queue(&self) -> Arc<Mutex<Vec<MidiEvent>>> {
        Arc::clone(&self.midi_queue)
    }

    /// Get the instrument slot for loading/replacing instruments.
    pub fn instrument_slot(&self) -> Arc<Mutex<Option<LoadedInstrument>>> {
        Arc::clone(&self.instrument)
    }

    /// Reset the round-robin state (call when loading a new instrument).
    pub fn reset_round_robin(&mut self) {
        self.rr_state.reset();
    }

    fn process_midi_events(&mut self) {
        let events: Vec<MidiEvent> = {
            let mut q = self.midi_queue.lock().unwrap();
            std::mem::take(&mut *q)
        };

        let inst_guard = self.instrument.lock().unwrap();
        let inst = match inst_guard.as_ref() {
            Some(i) => i,
            None => {
                // No instrument loaded, reset tracking
                if self.last_instrument_name.is_some() {
                    self.last_instrument_name = None;
                    self.rr_state.reset();
                }
                return;
            }
        };

        // Check if instrument changed and reset round-robin state if so
        let current_name = &inst.instrument.name;
        if self.last_instrument_name.as_ref() != Some(current_name) {
            self.last_instrument_name = Some(current_name.clone());
            self.rr_state.reset();
        }

        for event in events {
            match event.kind {
                MidiEventKind::NoteOn { note, velocity } => {
                    // Steal oldest voice if at max polyphony
                    let max_voices = inst.instrument.max_voices;
                    if self.voices.len() >= max_voices {
                        self.voices.remove(0);
                    }

                    if let Some(zone) = inst.instrument.find_zone_rr(note, velocity, &mut self.rr_state) {
                        if inst.buffers.contains_key(&zone.id) {
                            let vel_linear = velocity as f32 / 127.0;
                            let pitch_ratio = zone.pitch_ratio(note, &inst.instrument.tuning) as f64;
                            let volume = zone.volume_linear() * vel_linear;
                            let voice = SamplerVoice::new(
                                note, event.channel, vel_linear,
                                pitch_ratio, volume, zone,
                            );
                            self.voices.push(voice);
                        }
                    }
                }

                MidiEventKind::NoteOff { note, .. } => {
                    let ch = event.channel;
                    if self.sustain_pedal[ch as usize] {
                        self.sustained_notes.push((ch, note));
                    } else {
                        for v in self.voices.iter_mut() {
                            if v.note == note && v.channel == ch && v.key_held {
                                v.release();
                            }
                        }
                    }
                }

                MidiEventKind::ControlChange { cc, value } => {
                    let ch = event.channel as usize;
                    if cc == aether_midi::event::cc::SUSTAIN_PEDAL {
                        let held = value >= 64;
                        self.sustain_pedal[ch] = held;
                        if !held {
                            // Release all sustained notes
                            let to_release: Vec<(u8, u8)> = self.sustained_notes.drain(..).collect();
                            for (c, n) in to_release {
                                for v in self.voices.iter_mut() {
                                    if v.note == n && v.channel == c && v.key_held {
                                        v.release();
                                    }
                                }
                            }
                        }
                    }
                }

                MidiEventKind::AllNotesOff | MidiEventKind::AllSoundOff => {
                    for v in self.voices.iter_mut() {
                        v.release();
                    }
                    self.sustained_notes.clear();
                }

                _ => {}
            }
        }
    }

    fn render_voices(&mut self, output: &mut [f32; BUFFER_SIZE]) {
        let inst_guard = self.instrument.lock().unwrap();
        let inst = match inst_guard.as_ref() {
            Some(i) => i,
            None => return,
        };

        let sr = self.sample_rate;
        let attack_rate = 1.0 / (inst.instrument.attack * sr).max(1.0);
        let decay_rate = 1.0 / (inst.instrument.decay * sr).max(1.0);
        let sustain = inst.instrument.sustain;
        let release_rate = 1.0 / (inst.instrument.release * sr).max(1.0);

        for voice in self.voices.iter_mut() {
            if voice.is_done() { continue; }

            let buf = match inst.buffers.get(&voice.zone_id) {
                Some(b) => b,
                None => continue,
            };

            for sample in output.iter_mut() {
                voice.envelope.tick(attack_rate, decay_rate, sustain, release_rate);
                let frame_pos = voice.advance(buf.frames);
                let raw = buf.sample_at(frame_pos);
                *sample += raw * voice.volume * voice.envelope.level;
            }
        }

        // Remove finished voices
        self.voices.retain(|v| !v.is_done());
    }
}

impl DspNode for SamplerNode {
    fn process(
        &mut self,
        _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        _params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        output.fill(0.0);
        self.process_midi_events();
        self.render_voices(output);
    }

    fn type_name(&self) -> &'static str { "SamplerNode" }
}
