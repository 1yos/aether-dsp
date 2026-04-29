//! MIDI engine — manages hardware MIDI input via midir.

use std::sync::{Arc, Mutex};
use midir::{MidiInput, MidiInputConnection};
use crate::{event::MidiEvent, router::MidiRouter};

/// Manages MIDI hardware connections and routes events.
pub struct MidiEngine {
    router: Arc<Mutex<MidiRouter>>,
    _connections: Vec<MidiInputConnection<()>>,
    sample_counter: Arc<Mutex<u64>>,
}

impl MidiEngine {
    /// Create a new MIDI engine.
    pub fn new() -> Self {
        Self {
            router: Arc::new(Mutex::new(MidiRouter::new())),
            _connections: Vec::new(),
            sample_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// List available MIDI input port names.
    pub fn list_ports() -> Vec<String> {
        let midi_in = match MidiInput::new("aether-midi-list") {
            Ok(m) => m,
            Err(_) => return Vec::new(),
        };
        let ports = midi_in.ports();
        ports.iter()
            .filter_map(|p| midi_in.port_name(p).ok())
            .collect()
    }

    /// Connect to a MIDI input port by index.
    /// Returns Ok(()) if connected, Err with message if failed.
    pub fn connect_port(&mut self, port_index: usize) -> Result<(), String> {
        let midi_in = MidiInput::new("aether-midi-in")
            .map_err(|e| format!("MIDI init error: {e}"))?;
        let ports = midi_in.ports();
        let port = ports.get(port_index)
            .ok_or_else(|| format!("MIDI port {port_index} not found"))?;
        let port_name = midi_in.port_name(port)
            .unwrap_or_else(|_| format!("Port {port_index}"));

        let router = Arc::clone(&self.router);
        let counter = Arc::clone(&self.sample_counter);

        let conn = midi_in.connect(
            port,
            "aether-midi-conn",
            move |_timestamp_us, bytes, _| {
                let ts = *counter.lock().unwrap();
                if let Some(event) = MidiEvent::from_bytes(bytes, ts) {
                    router.lock().unwrap().dispatch(&event);
                }
            },
            (),
        ).map_err(|e| format!("MIDI connect error: {e}"))?;

        println!("MIDI: connected to '{port_name}'");
        self._connections.push(conn);
        Ok(())
    }

    /// Connect to the first available MIDI port.
    pub fn connect_first(&mut self) -> Result<String, String> {
        let ports = Self::list_ports();
        if ports.is_empty() {
            return Err("No MIDI input ports found".into());
        }
        self.connect_port(0)?;
        Ok(ports[0].clone())
    }

    /// Get a reference to the router for registering handlers.
    pub fn router(&self) -> Arc<Mutex<MidiRouter>> {
        Arc::clone(&self.router)
    }

    /// Advance the sample counter (call once per audio buffer).
    pub fn advance_samples(&self, samples: u64) {
        *self.sample_counter.lock().unwrap() += samples;
    }

    /// Inject a MIDI event directly (for testing or virtual keyboard).
    pub fn inject(&self, event: MidiEvent) {
        self.router.lock().unwrap().dispatch(&event);
    }
}

impl Default for MidiEngine {
    fn default() -> Self { Self::new() }
}
