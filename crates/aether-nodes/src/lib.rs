#[cfg(feature = "compressor")]
pub mod compressor;
#[cfg(feature = "delay")]
pub mod delay;
#[cfg(feature = "envelope")]
pub mod envelope;
#[cfg(feature = "filter")]
pub mod filter;
#[cfg(feature = "formant")]
pub mod formant;
#[cfg(feature = "gain")]
pub mod gain;
#[cfg(feature = "granular")]
pub mod granular;
#[cfg(feature = "karplus-strong")]
pub mod karplus_strong;
#[cfg(feature = "lfo")]
pub mod lfo;
#[cfg(feature = "mixer")]
pub mod mixer;
#[cfg(feature = "moog-ladder")]
pub mod moog_ladder;
#[cfg(feature = "oscillator")]
pub mod oscillator;
#[cfg(feature = "record")]
pub mod record;
#[cfg(feature = "reverb")]
pub mod reverb;
#[cfg(feature = "scope")]
pub mod scope;
#[cfg(feature = "waveshaper")]
pub mod waveshaper;
#[cfg(feature = "chorus")]
pub mod chorus;

#[cfg(test)]
mod tests {
    mod regression;
}

#[cfg(feature = "record")]
pub use record::RecordNode;
#[cfg(feature = "scope")]
pub use scope::ScopeNode;
#[cfg(feature = "compressor")]
pub use compressor::Compressor;
#[cfg(feature = "waveshaper")]
pub use waveshaper::Waveshaper;
#[cfg(feature = "chorus")]
pub use chorus::Chorus;