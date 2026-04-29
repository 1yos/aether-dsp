//! MIDI router — dispatches events to registered instrument handlers.

use std::collections::HashMap;
use crate::event::MidiEvent;

/// A function that handles a MIDI event.
pub type MidiHandler = Box<dyn Fn(&MidiEvent) + Send + Sync>;

/// Routes MIDI events to registered handlers by channel.
pub struct MidiRouter {
    /// Per-channel handlers. Channel 255 = all channels.
    handlers: HashMap<u8, Vec<MidiHandler>>,
}

impl MidiRouter {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    /// Register a handler for a specific MIDI channel (0–15).
    /// Use channel 255 to receive events from all channels.
    pub fn register(&mut self, channel: u8, handler: MidiHandler) {
        self.handlers.entry(channel).or_default().push(handler);
    }

    /// Dispatch an event to all matching handlers.
    pub fn dispatch(&self, event: &MidiEvent) {
        // Channel-specific handlers
        if let Some(handlers) = self.handlers.get(&event.channel) {
            for h in handlers { h(event); }
        }
        // All-channel handlers
        if let Some(handlers) = self.handlers.get(&255) {
            for h in handlers { h(event); }
        }
    }

    /// Remove all handlers for a channel.
    pub fn clear_channel(&mut self, channel: u8) {
        self.handlers.remove(&channel);
    }
}

impl Default for MidiRouter {
    fn default() -> Self { Self::new() }
}
