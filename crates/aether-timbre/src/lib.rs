//! Aether Timbre Transfer Engine
//!
//! Makes one instrument sound like another using spectral envelope matching.
//!
//! # How it works
//!
//! 1. **Analysis** — Extract the spectral envelope (timbre fingerprint) of the
//!    target instrument from a reference recording using LPC or cepstral analysis.
//!
//! 2. **Transfer** — When processing a source signal (e.g. guitar), apply the
//!    target's spectral envelope while preserving the source's pitch and dynamics.
//!
//! 3. **Result** — The output sounds like the target instrument played with the
//!    expressiveness of the source.
//!
//! # Use cases
//!
//! - You have a guitar but want it to sound like a Krar (Ethiopian lyre)
//! - You have a piano but want it to sound like a Sitar
//! - You want to create entirely new hybrid instruments
//! - You want to generate synthetic samples for the instrument maker
//!   without recording the actual instrument

pub mod analysis;
pub mod transfer;
pub mod node;
pub mod synthesizer;

pub use analysis::{SpectralEnvelope, TimbreProfile};
pub use transfer::TimbreTransfer;
pub use node::TimbreTransferNode;
pub use synthesizer::InstrumentSynthesizer;
