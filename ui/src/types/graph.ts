export interface NodeSnapshot {
  id: number;
  generation: number;
  node_type: string;
  params: number[];
}

export interface EdgeSnapshot {
  src_id: number;
  dst_id: number;
  slot: number;
}

export interface GraphSnapshot {
  nodes: NodeSnapshot[];
  edges: EdgeSnapshot[];
}

export interface ParamDef {
  name: string;
  min: number;
  max: number;
  default: number;
}

/** How many audio inputs each node type accepts. */
export const NODE_INPUT_COUNT: Record<string, number> = {
  Oscillator: 0, // source — no audio inputs
  StateVariableFilter: 1,
  AdsrEnvelope: 1, // optional audio input (AM mode)
  DelayLine: 1,
  Gain: 1,
  Mixer: 4, // up to 4 inputs
  SamplerNode: 0, // MIDI-driven source — no audio inputs
  TimbreTransferNode: 1,
};

export const NODE_PARAM_DEFS: Record<string, ParamDef[]> = {
  Oscillator: [
    { name: "Frequency", min: 20, max: 20000, default: 440 },
    { name: "Amplitude", min: 0, max: 1, default: 0.5 },
    { name: "Waveform", min: 0, max: 3, default: 0 },
  ],
  StateVariableFilter: [
    { name: "Cutoff", min: 20, max: 20000, default: 2000 },
    { name: "Q", min: 0.5, max: 20, default: 1 },
    { name: "Mode", min: 0, max: 2, default: 0 },
  ],
  AdsrEnvelope: [
    { name: "Attack", min: 0, max: 10, default: 0.01 },
    { name: "Decay", min: 0, max: 10, default: 0.1 },
    { name: "Sustain", min: 0, max: 1, default: 0.7 },
    { name: "Release", min: 0, max: 10, default: 0.3 },
    { name: "Gate", min: 0, max: 1, default: 0 },
  ],
  DelayLine: [
    { name: "Time", min: 0, max: 2, default: 0.25 },
    { name: "Feedback", min: 0, max: 0.99, default: 0.4 },
    { name: "Wet", min: 0, max: 1, default: 0.5 },
  ],
  Gain: [{ name: "Gain", min: 0, max: 4, default: 1 }],
  Mixer: [],
  SamplerNode: [],
  TimbreTransferNode: [{ name: "Amount", min: 0, max: 1, default: 1.0 }],
};
