//! Built-in node registry — pre-registers all aether-nodes types.

use crate::node::NodeRegistry;

/// Build a registry pre-populated with all built-in AetherDSP nodes.
pub fn builtin_registry() -> NodeRegistry {
    use aether_nodes::{
        delay::DelayLine,
        envelope::AdsrEnvelope,
        filter::StateVariableFilter,
        gain::Gain,
        mixer::Mixer,
        oscillator::Oscillator,
    };
    #[allow(unused_imports)]
    use aether_core::node::DspNode;
    use crate::node::NodeEntry;
    use crate::ParamDef;

    static OSC_DEFS: &[ParamDef] = &[
        ParamDef { name: "Frequency", min: 20.0, max: 20000.0, default: 440.0 },
        ParamDef { name: "Amplitude", min: 0.0, max: 1.0, default: 0.5 },
        ParamDef { name: "Waveform",  min: 0.0, max: 3.0, default: 0.0 },
    ];
    static FILT_DEFS: &[ParamDef] = &[
        ParamDef { name: "Cutoff",    min: 20.0, max: 20000.0, default: 2000.0 },
        ParamDef { name: "Resonance", min: 0.5,  max: 20.0,    default: 1.0 },
        ParamDef { name: "Mode",      min: 0.0,  max: 2.0,     default: 0.0 },
    ];
    static ENV_DEFS: &[ParamDef] = &[
        ParamDef { name: "Attack",  min: 0.0, max: 10.0, default: 0.01 },
        ParamDef { name: "Decay",   min: 0.0, max: 10.0, default: 0.1 },
        ParamDef { name: "Sustain", min: 0.0, max: 1.0,  default: 0.7 },
        ParamDef { name: "Release", min: 0.0, max: 10.0, default: 0.3 },
        ParamDef { name: "Gate",    min: 0.0, max: 1.0,  default: 0.0 },
    ];
    static DELAY_DEFS: &[ParamDef] = &[
        ParamDef { name: "Time",     min: 0.0, max: 2.0,  default: 0.25 },
        ParamDef { name: "Feedback", min: 0.0, max: 0.99, default: 0.4 },
        ParamDef { name: "Wet",      min: 0.0, max: 1.0,  default: 0.5 },
    ];
    static GAIN_DEFS: &[ParamDef] = &[
        ParamDef { name: "Gain", min: 0.0, max: 4.0, default: 1.0 },
    ];
    static MIXER_DEFS: &[ParamDef] = &[];

    let mut r = NodeRegistry::new();

    r.register(NodeEntry {
        type_name: "Oscillator",
        param_defs: OSC_DEFS,
        factory: || Box::new(Oscillator::new()),
    });
    r.register(NodeEntry {
        type_name: "StateVariableFilter",
        param_defs: FILT_DEFS,
        factory: || Box::new(StateVariableFilter::new()),
    });
    r.register(NodeEntry {
        type_name: "AdsrEnvelope",
        param_defs: ENV_DEFS,
        factory: || Box::new(AdsrEnvelope::new()),
    });
    r.register(NodeEntry {
        type_name: "DelayLine",
        param_defs: DELAY_DEFS,
        factory: || Box::new(DelayLine::new()),
    });
    r.register(NodeEntry {
        type_name: "Gain",
        param_defs: GAIN_DEFS,
        factory: || Box::new(Gain),
    });
    r.register(NodeEntry {
        type_name: "Mixer",
        param_defs: MIXER_DEFS,
        factory: || Box::new(Mixer),
    });

    r
}
