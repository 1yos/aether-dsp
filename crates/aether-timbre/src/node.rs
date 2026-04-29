//! TimbreTransferNode — integrates timbre transfer into the AetherDSP graph.
//!
//! This node sits in the signal chain and reshapes the incoming audio's
//! spectral envelope to match a target instrument's timbre profile.
//!
//! Signal flow:
//!   [Source Node] → [TimbreTransferNode] → [Output / Mixer]
//!
//! Params:
//!   0: Amount (0.0 = dry, 1.0 = full transfer)

use aether_core::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};
use crate::{analysis::TimbreProfile, transfer::TimbreTransfer};
use std::sync::{Arc, Mutex};

/// A real-time timbre transfer node.
pub struct TimbreTransferNode {
    transfer: TimbreTransfer,
    /// Shared profile — can be swapped from the control thread.
    profile: Arc<Mutex<Option<TimbreProfile>>>,
    /// Current MIDI note for pitch-dependent timbre selection.
    current_note: u8,
    /// Whether the profile has been loaded into the transfer engine.
    profile_loaded: bool,
}

impl TimbreTransferNode {
    pub fn new() -> Self {
        Self {
            transfer: TimbreTransfer::new(2048),
            profile: Arc::new(Mutex::new(None)),
            current_note: 60,
            profile_loaded: false,
        }
    }

    /// Get the profile slot for loading from the control thread.
    pub fn profile_slot(&self) -> Arc<Mutex<Option<TimbreProfile>>> {
        Arc::clone(&self.profile)
    }

    /// Set the current MIDI note (for pitch-dependent timbre).
    pub fn set_note(&mut self, note: u8) {
        if note != self.current_note {
            self.current_note = note;
            self.profile_loaded = false; // Reload envelope for new note
        }
    }

    fn maybe_reload_profile(&mut self) {
        if self.profile_loaded {
            return;
        }
        if let Ok(guard) = self.profile.try_lock() {
            if let Some(profile) = guard.as_ref() {
                if let Some(envelope) = profile.envelope_for_note(self.current_note) {
                    self.transfer.set_target(envelope.clone());
                    self.profile_loaded = true;
                }
            }
        }
    }
}

impl Default for TimbreTransferNode {
    fn default() -> Self {
        Self::new()
    }
}

impl DspNode for TimbreTransferNode {
    fn process(
        &mut self,
        inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        params: &mut ParamBlock,
        _sample_rate: f32,
    ) {
        // Param 0: transfer amount (0.0 = dry pass-through, 1.0 = full transfer).
        // We read `params.params[0].current` — the smoothed running value — not
        // `target`, so the amount ramps sample-accurately when automated.
        // If no params are registered (params.count == 0), default to full transfer (1.0).
        let amount = if params.count > 0 {
            params.params[0].current.clamp(0.0, 1.0)
        } else {
            // No param block registered — default to full transfer.
            1.0
        };
        self.transfer.amount = amount;

        // Try to load profile if not yet loaded (non-blocking try_lock — RT safe).
        self.maybe_reload_profile();

        // No input connected → output silence.
        // Invariant: if inputs[0] is None the node has no upstream source; fill
        // the output buffer with zeros rather than leaving it uninitialised.
        let input = match inputs[0] {
            Some(buf) => buf,
            None => {
                // No input → silence.
                output.fill(0.0);
                return;
            }
        };

        // amount ≈ 0 → dry pass-through.
        // Skip the FFT pipeline entirely and copy input to output unchanged.
        // Threshold of 0.001 avoids audible artefacts from near-zero processing.
        if amount < 0.001 {
            // Dry pass-through: copy input to output, no DSP applied.
            output.copy_from_slice(input);
            return;
        }

        // amount > 0 → full or partial timbre transfer.
        // `TimbreTransfer::process_block` blends dry and wet internally using
        // `self.transfer.amount`, so amount=1.0 gives full spectral replacement.
        let processed = self.transfer.process_block(input);
        for (i, s) in processed.iter().enumerate().take(BUFFER_SIZE) {
            output[i] = *s;
        }
    }

    fn type_name(&self) -> &'static str {
        "TimbreTransferNode"
    }

    // `capture_state` and `restore_state` are no-ops for now — the DspNode
    // trait provides default implementations that return StateBlob::EMPTY and
    // do nothing on restore, respectively.  They compile without any override.
}
