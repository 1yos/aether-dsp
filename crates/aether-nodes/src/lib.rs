pub mod compressor;
pub mod delay;
pub mod envelope;
pub mod filter;
pub mod formant;
pub mod gain;
pub mod granular;
pub mod karplus_strong;
pub mod lfo;
pub mod mixer;
pub mod moog_ladder;
pub mod oscillator;
pub mod record;
pub mod reverb;
pub mod scope;
pub mod waveshaper;
pub mod chorus;

#[cfg(test)]
mod tests {
    mod regression;
}

pub use record::RecordNode;
pub use scope::ScopeNode;
pub use compressor::Compressor;
pub use waveshaper::Waveshaper;
pub use chorus::Chorus;