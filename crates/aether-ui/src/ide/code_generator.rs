// Code generator - generates actual Rust code for DSP nodes

use crate::dsp_graph::NodeType;

/// Generate Rust code for a DSP node
pub fn generate_node_code(node_type: &NodeType, node_name: &str) -> String {
    match node_type {
        NodeType::Oscillator { waveform } => generate_oscillator_code(node_name, *waveform),
        NodeType::Gain => generate_gain_code(node_name),
        NodeType::LowPass => generate_lowpass_filter_code(node_name),
        NodeType::HighPass => generate_highpass_filter_code(node_name),
        NodeType::Delay => generate_delay_code(node_name),
        NodeType::Compressor => generate_compressor_code(node_name),
        NodeType::Reverb => generate_reverb_code(node_name),
        NodeType::LFO => generate_lfo_code(node_name),
        NodeType::Envelope => generate_envelope_code(node_name),
        _ => generate_generic_node_code(node_name, node_type.name()),
    }
}

fn generate_oscillator_code(name: &str, waveform: crate::dsp_graph::Waveform) -> String {
    let waveform_str = match waveform {
        crate::dsp_graph::Waveform::Sine => "sine",
        crate::dsp_graph::Waveform::Saw => "saw",
        crate::dsp_graph::Waveform::Square => "square",
        crate::dsp_graph::Waveform::Triangle => "triangle",
        crate::dsp_graph::Waveform::Noise => "noise",
    };

    format!(
        r#"use aetherdsp_core::{{node::DspNode, BUFFER_SIZE}};

/// {} Oscillator - Generates {} waveform
pub struct {} {{
    frequency: f32,
    amplitude: f32,
    phase: f32,
    sample_rate: f32,
}}

impl {} {{
    pub fn new(sample_rate: f32) -> Self {{
        Self {{
            frequency: 440.0,
            amplitude: 0.5,
            phase: 0.0,
            sample_rate,
        }}
    }}

    pub fn set_frequency(&mut self, freq: f32) {{
        self.frequency = freq.clamp(20.0, 20000.0);
    }}

    pub fn set_amplitude(&mut self, amp: f32) {{
        self.amplitude = amp.clamp(0.0, 1.0);
    }}
}}

impl DspNode for {} {{
    fn process(
        &mut self,
        _inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {{
        if outputs.is_empty() {{
            return;
        }}

        let output = outputs[0];
        let phase_increment = self.frequency / self.sample_rate;

        for i in 0..BUFFER_SIZE {{
            // Generate {} waveform
            let sample = match "{}" {{
                "sine" => (self.phase * 2.0 * std::f32::consts::PI).sin(),
                "saw" => 2.0 * self.phase - 1.0,
                "square" => if self.phase < 0.5 {{ 1.0 }} else {{ -1.0 }},
                "triangle" => 4.0 * (self.phase - 0.5).abs() - 1.0,
                _ => 0.0,
            }};

            output[i] = sample * self.amplitude;

            self.phase += phase_increment;
            if self.phase >= 1.0 {{
                self.phase -= 1.0;
            }}
        }}
    }}

    fn num_inputs(&self) -> usize {{ 0 }}
    fn num_outputs(&self) -> usize {{ 1 }}
}}
"#,
        waveform_str, waveform_str, name, name, name, waveform_str, waveform_str
    )
}

fn generate_gain_code(name: &str) -> String {
    format!(
        r#"use aetherdsp_core::{{node::DspNode, BUFFER_SIZE}};

/// {} - Simple gain/volume control
pub struct {} {{
    gain: f32,
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{ gain: 1.0 }}
    }}

    pub fn set_gain(&mut self, gain: f32) {{
        self.gain = gain.clamp(0.0, 2.0);
    }}
}}

impl DspNode for {} {{
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {{
        if inputs.is_empty() || outputs.is_empty() {{
            return;
        }}

        let input = inputs[0];
        let output = outputs[0];

        for i in 0..BUFFER_SIZE {{
            output[i] = input[i] * self.gain;
        }}
    }}

    fn num_inputs(&self) -> usize {{ 1 }}
    fn num_outputs(&self) -> usize {{ 1 }}
}}
"#,
        name, name, name, name
    )
}

fn generate_lowpass_filter_code(name: &str) -> String {
    format!(
        r#"use aetherdsp_core::{{node::DspNode, BUFFER_SIZE}};

/// {} - One-pole lowpass filter
pub struct {} {{
    cutoff: f32,
    resonance: f32,
    z1: f32,
    sample_rate: f32,
}}

impl {} {{
    pub fn new(sample_rate: f32) -> Self {{
        Self {{
            cutoff: 1000.0,
            resonance: 0.5,
            z1: 0.0,
            sample_rate,
        }}
    }}

    pub fn set_cutoff(&mut self, freq: f32) {{
        self.cutoff = freq.clamp(20.0, 20000.0);
    }}

    pub fn set_resonance(&mut self, res: f32) {{
        self.resonance = res.clamp(0.0, 1.0);
    }}
}}

impl DspNode for {} {{
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {{
        if inputs.is_empty() || outputs.is_empty() {{
            return;
        }}

        let input = inputs[0];
        let output = outputs[0];

        // Calculate filter coefficient
        let omega = 2.0 * std::f32::consts::PI * self.cutoff / self.sample_rate;
        let alpha = omega / (omega + 1.0);

        for i in 0..BUFFER_SIZE {{
            self.z1 = alpha * input[i] + (1.0 - alpha) * self.z1;
            output[i] = self.z1;
        }}
    }}

    fn num_inputs(&self) -> usize {{ 1 }}
    fn num_outputs(&self) -> usize {{ 1 }}
}}
"#,
        name, name, name, name
    )
}

fn generate_highpass_filter_code(name: &str) -> String {
    format!(
        r#"use aetherdsp_core::{{node::DspNode, BUFFER_SIZE}};

/// {} - One-pole highpass filter
pub struct {} {{
    cutoff: f32,
    resonance: f32,
    z1: f32,
    sample_rate: f32,
}}

impl {} {{
    pub fn new(sample_rate: f32) -> Self {{
        Self {{
            cutoff: 1000.0,
            resonance: 0.5,
            z1: 0.0,
            sample_rate,
        }}
    }}

    pub fn set_cutoff(&mut self, freq: f32) {{
        self.cutoff = freq.clamp(20.0, 20000.0);
    }}

    pub fn set_resonance(&mut self, res: f32) {{
        self.resonance = res.clamp(0.0, 1.0);
    }}
}}

impl DspNode for {} {{
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {{
        if inputs.is_empty() || outputs.is_empty() {{
            return;
        }}

        let input = inputs[0];
        let output = outputs[0];

        // Calculate filter coefficient
        let omega = 2.0 * std::f32::consts::PI * self.cutoff / self.sample_rate;
        let alpha = 1.0 / (omega + 1.0);

        for i in 0..BUFFER_SIZE {{
            let hp = alpha * (self.z1 + input[i] - self.z1);
            self.z1 = input[i];
            output[i] = hp;
        }}
    }}

    fn num_inputs(&self) -> usize {{ 1 }}
    fn num_outputs(&self) -> usize {{ 1 }}
}}
"#,
        name, name, name, name
    )
}

fn generate_delay_code(name: &str) -> String {
    format!(
        r#"use aetherdsp_core::{{node::DspNode, BUFFER_SIZE}};

/// {} - Simple delay line with feedback
pub struct {} {{
    delay_time: f32,
    feedback: f32,
    mix: f32,
    buffer: Vec<f32>,
    write_pos: usize,
    sample_rate: f32,
}}

impl {} {{
    pub fn new(sample_rate: f32) -> Self {{
        let max_delay_samples = (sample_rate * 2.0) as usize;
        Self {{
            delay_time: 0.5,
            feedback: 0.3,
            mix: 0.5,
            buffer: vec![0.0; max_delay_samples],
            write_pos: 0,
            sample_rate,
        }}
    }}

    pub fn set_delay_time(&mut self, time: f32) {{
        self.delay_time = time.clamp(0.0, 2.0);
    }}

    pub fn set_feedback(&mut self, fb: f32) {{
        self.feedback = fb.clamp(0.0, 1.0);
    }}

    pub fn set_mix(&mut self, mix: f32) {{
        self.mix = mix.clamp(0.0, 1.0);
    }}
}}

impl DspNode for {} {{
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {{
        if inputs.is_empty() || outputs.is_empty() {{
            return;
        }}

        let input = inputs[0];
        let output = outputs[0];

        let delay_samples = (self.delay_time * self.sample_rate) as usize;
        let buffer_len = self.buffer.len();

        for i in 0..BUFFER_SIZE {{
            let read_pos = (self.write_pos + buffer_len - delay_samples) % buffer_len;
            let delayed = self.buffer[read_pos];

            self.buffer[self.write_pos] = input[i] + delayed * self.feedback;
            output[i] = input[i] * (1.0 - self.mix) + delayed * self.mix;

            self.write_pos = (self.write_pos + 1) % buffer_len;
        }}
    }}

    fn num_inputs(&self) -> usize {{ 1 }}
    fn num_outputs(&self) -> usize {{ 1 }}
}}
"#,
        name, name, name, name
    )
}

fn generate_compressor_code(name: &str) -> String {
    format!(
        r#"use aetherdsp_core::{{node::DspNode, BUFFER_SIZE}};

/// {} - Dynamic range compressor
pub struct {} {{
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    envelope: f32,
    sample_rate: f32,
}}

impl {} {{
    pub fn new(sample_rate: f32) -> Self {{
        Self {{
            threshold: -20.0,
            ratio: 4.0,
            attack: 10.0,
            release: 100.0,
            envelope: 0.0,
            sample_rate,
        }}
    }}

    pub fn set_threshold(&mut self, thresh: f32) {{
        self.threshold = thresh.clamp(-60.0, 0.0);
    }}

    pub fn set_ratio(&mut self, ratio: f32) {{
        self.ratio = ratio.clamp(1.0, 20.0);
    }}

    pub fn set_attack(&mut self, attack: f32) {{
        self.attack = attack.clamp(0.0, 100.0);
    }}

    pub fn set_release(&mut self, release: f32) {{
        self.release = release.clamp(0.0, 1000.0);
    }}
}}

impl DspNode for {} {{
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {{
        if inputs.is_empty() || outputs.is_empty() {{
            return;
        }}

        let input = inputs[0];
        let output = outputs[0];

        let attack_coef = (-1.0 / (self.attack * 0.001 * self.sample_rate)).exp();
        let release_coef = (-1.0 / (self.release * 0.001 * self.sample_rate)).exp();

        for i in 0..BUFFER_SIZE {{
            let input_db = 20.0 * input[i].abs().max(1e-6).log10();

            // Envelope follower
            let coef = if input_db > self.envelope {{
                attack_coef
            }} else {{
                release_coef
            }};
            self.envelope = input_db + coef * (self.envelope - input_db);

            // Calculate gain reduction
            let over_threshold = (self.envelope - self.threshold).max(0.0);
            let gain_reduction = over_threshold * (1.0 - 1.0 / self.ratio);
            let gain_linear = 10.0_f32.powf(-gain_reduction / 20.0);

            output[i] = input[i] * gain_linear;
        }}
    }}

    fn num_inputs(&self) -> usize {{ 1 }}
    fn num_outputs(&self) -> usize {{ 1 }}
}}
"#,
        name, name, name, name
    )
}

fn generate_reverb_code(name: &str) -> String {
    format!(
        r#"use aetherdsp_core::{{node::DspNode, BUFFER_SIZE}};

/// {} - Simple reverb (placeholder implementation)
pub struct {} {{
    room_size: f32,
    damping: f32,
    mix: f32,
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{
            room_size: 0.5,
            damping: 0.5,
            mix: 0.3,
        }}
    }}

    pub fn set_room_size(&mut self, size: f32) {{
        self.room_size = size.clamp(0.0, 1.0);
    }}

    pub fn set_damping(&mut self, damp: f32) {{
        self.damping = damp.clamp(0.0, 1.0);
    }}

    pub fn set_mix(&mut self, mix: f32) {{
        self.mix = mix.clamp(0.0, 1.0);
    }}
}}

impl DspNode for {} {{
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {{
        if inputs.is_empty() || outputs.is_empty() {{
            return;
        }}

        let input = inputs[0];
        let output = outputs[0];

        // TODO: Implement proper reverb algorithm (Freeverb, etc.)
        for i in 0..BUFFER_SIZE {{
            output[i] = input[i]; // Passthrough for now
        }}
    }}

    fn num_inputs(&self) -> usize {{ 1 }}
    fn num_outputs(&self) -> usize {{ 1 }}
}}
"#,
        name, name, name, name
    )
}

fn generate_lfo_code(name: &str) -> String {
    format!(
        r#"use aetherdsp_core::{{node::DspNode, BUFFER_SIZE}};

/// {} - Low Frequency Oscillator
pub struct {} {{
    rate: f32,
    depth: f32,
    phase: f32,
    sample_rate: f32,
}}

impl {} {{
    pub fn new(sample_rate: f32) -> Self {{
        Self {{
            rate: 1.0,
            depth: 0.5,
            phase: 0.0,
            sample_rate,
        }}
    }}

    pub fn set_rate(&mut self, rate: f32) {{
        self.rate = rate.clamp(0.01, 20.0);
    }}

    pub fn set_depth(&mut self, depth: f32) {{
        self.depth = depth.clamp(0.0, 1.0);
    }}
}}

impl DspNode for {} {{
    fn process(
        &mut self,
        _inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {{
        if outputs.is_empty() {{
            return;
        }}

        let output = outputs[0];
        let phase_increment = self.rate / self.sample_rate;

        for i in 0..BUFFER_SIZE {{
            let lfo_value = (self.phase * 2.0 * std::f32::consts::PI).sin();
            output[i] = lfo_value * self.depth;

            self.phase += phase_increment;
            if self.phase >= 1.0 {{
                self.phase -= 1.0;
            }}
        }}
    }}

    fn num_inputs(&self) -> usize {{ 0 }}
    fn num_outputs(&self) -> usize {{ 1 }}
}}
"#,
        name, name, name, name
    )
}

fn generate_envelope_code(name: &str) -> String {
    format!(
        r#"use aetherdsp_core::{{node::DspNode, BUFFER_SIZE}};

/// {} - ADSR Envelope Generator
pub struct {} {{
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    stage: EnvelopeStage,
    level: f32,
    sample_rate: f32,
}}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EnvelopeStage {{
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}}

impl {} {{
    pub fn new(sample_rate: f32) -> Self {{
        Self {{
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
            stage: EnvelopeStage::Idle,
            level: 0.0,
            sample_rate,
        }}
    }}

    pub fn set_attack(&mut self, attack: f32) {{
        self.attack = attack.clamp(0.0, 2.0);
    }}

    pub fn set_decay(&mut self, decay: f32) {{
        self.decay = decay.clamp(0.0, 2.0);
    }}

    pub fn set_sustain(&mut self, sustain: f32) {{
        self.sustain = sustain.clamp(0.0, 1.0);
    }}

    pub fn set_release(&mut self, release: f32) {{
        self.release = release.clamp(0.0, 5.0);
    }}

    pub fn trigger(&mut self) {{
        self.stage = EnvelopeStage::Attack;
    }}

    pub fn release_trigger(&mut self) {{
        self.stage = EnvelopeStage::Release;
    }}
}}

impl DspNode for {} {{
    fn process(
        &mut self,
        _inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {{
        if outputs.is_empty() {{
            return;
        }}

        let output = outputs[0];

        for i in 0..BUFFER_SIZE {{
            match self.stage {{
                EnvelopeStage::Idle => {{
                    self.level = 0.0;
                }}
                EnvelopeStage::Attack => {{
                    let rate = 1.0 / (self.attack * self.sample_rate);
                    self.level += rate;
                    if self.level >= 1.0 {{
                        self.level = 1.0;
                        self.stage = EnvelopeStage::Decay;
                    }}
                }}
                EnvelopeStage::Decay => {{
                    let rate = (1.0 - self.sustain) / (self.decay * self.sample_rate);
                    self.level -= rate;
                    if self.level <= self.sustain {{
                        self.level = self.sustain;
                        self.stage = EnvelopeStage::Sustain;
                    }}
                }}
                EnvelopeStage::Sustain => {{
                    self.level = self.sustain;
                }}
                EnvelopeStage::Release => {{
                    let rate = self.level / (self.release * self.sample_rate);
                    self.level -= rate;
                    if self.level <= 0.0 {{
                        self.level = 0.0;
                        self.stage = EnvelopeStage::Idle;
                    }}
                }}
            }}

            output[i] = self.level;
        }}
    }}

    fn num_inputs(&self) -> usize {{ 0 }}
    fn num_outputs(&self) -> usize {{ 1 }}
}}
"#,
        name, name, name, name
    )
}

fn generate_generic_node_code(name: &str, node_type: &str) -> String {
    format!(
        r#"use aetherdsp_core::{{node::DspNode, BUFFER_SIZE}};

/// {} - {}
pub struct {} {{
    // TODO: Add node-specific state
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{
            // TODO: Initialize state
        }}
    }}
}}

impl DspNode for {} {{
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        _sample_rate: f32,
    ) {{
        if inputs.is_empty() || outputs.is_empty() {{
            return;
        }}

        let input = inputs[0];
        let output = outputs[0];

        // TODO: Implement processing
        for i in 0..BUFFER_SIZE {{
            output[i] = input[i]; // Passthrough for now
        }}
    }}

    fn num_inputs(&self) -> usize {{ 1 }}
    fn num_outputs(&self) -> usize {{ 1 }}
}}
"#,
        name, node_type, name, name, name
    )
}

/// Generate a complete graph.rs file with all nodes
pub fn generate_graph_code(nodes: &[(String, NodeType)]) -> String {
    let mut code = String::from(
        r#"use aetherdsp_core::{graph::AudioGraph, node::DspNode};

// Import all node modules
"#,
    );

    // Add imports for each node
    for (node_name, _) in nodes {
        code.push_str(&format!("mod {};\n", node_name.to_lowercase()));
    }

    code.push_str(
        r#"
/// Build the DSP graph
pub fn build_graph(sample_rate: f32) -> AudioGraph {
    let mut graph = AudioGraph::new(sample_rate);

"#,
    );

    // Add node instantiation
    for (node_name, _node_type) in nodes {
        let struct_name = to_pascal_case(node_name);
        code.push_str(&format!(
            "    let {} = Box::new({}::{}::new(sample_rate));\n",
            node_name.to_lowercase(),
            node_name.to_lowercase(),
            struct_name
        ));
    }

    code.push_str(
        r#"
    // TODO: Add nodes to graph and create connections

    graph
}
"#,
    );

    code
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
