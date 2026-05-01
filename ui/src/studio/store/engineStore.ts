/**
 * Engine store — single source of truth.
 * The host owns the graph; this store mirrors what the host tells us.
 * Every mutation goes to the host first, then we update from the response.
 */
import { create } from "zustand";
import {
  Node,
  Edge,
  applyNodeChanges,
  applyEdgeChanges,
  Connection,
  NodeChange,
  EdgeChange,
} from "reactflow";

export type WsStatus = "disconnected" | "connecting" | "connected" | "error";

export interface ParamDef {
  name: string;
  min: number;
  max: number;
  default: number;
}

export interface NodeDef {
  type: string;
  inputs: number;
  params: ParamDef[];
  color: string;
}

export const NODE_DEFS: Record<string, NodeDef> = {
  Oscillator: {
    type: "Oscillator",
    inputs: 0,
    color: "#4fc3f7",
    params: [
      { name: "Frequency", min: 20, max: 20000, default: 440 },
      { name: "Amplitude", min: 0, max: 1, default: 0.5 },
      { name: "Waveform", min: 0, max: 3, default: 0 },
      { name: "MIDI Note", min: -1, max: 127, default: -1 },
    ],
  },
  StateVariableFilter: {
    type: "StateVariableFilter",
    inputs: 1,
    color: "#ce93d8",
    params: [
      { name: "Cutoff", min: 20, max: 20000, default: 2000 },
      { name: "Resonance", min: 0.5, max: 20, default: 1 },
      { name: "Mode", min: 0, max: 2, default: 0 },
    ],
  },
  AdsrEnvelope: {
    type: "AdsrEnvelope",
    inputs: 1,
    color: "#ffb74d",
    params: [
      { name: "Attack", min: 0, max: 10, default: 0.01 },
      { name: "Decay", min: 0, max: 10, default: 0.1 },
      { name: "Sustain", min: 0, max: 1, default: 0.7 },
      { name: "Release", min: 0, max: 10, default: 0.3 },
      { name: "Gate", min: 0, max: 1, default: 1.0 },
    ],
  },
  DelayLine: {
    type: "DelayLine",
    inputs: 1,
    color: "#80cbc4",
    params: [
      { name: "Time", min: 0, max: 2, default: 0.25 },
      { name: "Feedback", min: 0, max: 0.99, default: 0.4 },
      { name: "Wet", min: 0, max: 1, default: 0.5 },
    ],
  },
  Gain: {
    type: "Gain",
    inputs: 1,
    color: "#a5d6a7",
    params: [{ name: "Gain", min: 0, max: 4, default: 0.8 }],
  },
  Mixer: {
    type: "Mixer",
    inputs: 4,
    color: "#ef9a9a",
    params: [],
  },
  SamplerNode: {
    type: "SamplerNode",
    inputs: 0,
    color: "#fff176",
    params: [],
  },
  TimbreTransferNode: {
    type: "TimbreTransferNode",
    inputs: 1,
    color: "#b39ddb",
    params: [{ name: "Amount", min: 0, max: 1, default: 1.0 }],
  },
  ScopeNode: {
    type: "ScopeNode",
    inputs: 1,
    color: "#38bdf8",
    params: [],
  },
  RecordNode: {
    type: "RecordNode",
    inputs: 1,
    color: "#f87171",
    params: [],
  },
  Lfo: {
    type: "Lfo",
    inputs: 0,
    color: "#a78bfa",
    params: [
      { name: "Rate", min: 0.01, max: 20, default: 1.0 },
      { name: "Depth", min: 0, max: 1, default: 0.5 },
      { name: "Waveform", min: 0, max: 4, default: 0 },
      { name: "Phase", min: 0, max: 1, default: 0 },
    ],
  },
  Reverb: {
    type: "Reverb",
    inputs: 1,
    color: "#818cf8",
    params: [
      { name: "Room Size", min: 0, max: 1, default: 0.5 },
      { name: "Damping", min: 0, max: 1, default: 0.5 },
      { name: "Wet", min: 0, max: 1, default: 0.3 },
      { name: "Width", min: 0, max: 1, default: 1.0 },
    ],
  },
  KarplusStrong: {
    type: "KarplusStrong",
    inputs: 0,
    color: "#d4a017",
    params: [
      { name: "Frequency", min: 20, max: 4000, default: 440 },
      { name: "Decay", min: 0.9, max: 0.9999, default: 0.995 },
      { name: "Brightness", min: 0, max: 1, default: 0.7 },
      { name: "Trigger", min: 0, max: 1, default: 0 },
    ],
  },
  FormantFilter: {
    type: "FormantFilter",
    inputs: 1,
    color: "#e91e8c",
    params: [
      { name: "Vowel", min: 0, max: 4, default: 0 },
      { name: "Shift", min: -12, max: 12, default: 0 },
      { name: "Wet", min: 0, max: 1, default: 0.5 },
    ],
  },
  MoogLadder: {
    type: "MoogLadder",
    inputs: 1,
    color: "#ff8c42",
    params: [
      { name: "Cutoff", min: 20, max: 20000, default: 2000 },
      { name: "Resonance", min: 0, max: 4, default: 0.5 },
      { name: "Drive", min: 0, max: 1, default: 0 },
    ],
  },
  Granular: {
    type: "Granular",
    inputs: 1,
    color: "#00897b",
    params: [
      { name: "Grain Size", min: 10, max: 500, default: 80 },
      { name: "Density", min: 1, max: 50, default: 8 },
      { name: "Pitch Scatter", min: 0, max: 2, default: 0.3 },
      { name: "Position", min: 0, max: 1, default: 0.5 },
      { name: "Pos Scatter", min: 0, max: 1, default: 0.2 },
      { name: "Wet", min: 0, max: 1, default: 0.8 },
    ],
  },
};

interface EngineStore {
  // Graph visual state
  nodes: Node[];
  edges: Edge[];
  selectedNodeId: string | null;

  // Engine state (mirrored from host)
  wsStatus: WsStatus;
  muted: boolean;
  outputNodeId: string | null;
  audioActive: boolean;

  // MIDI state
  midiPorts: string[];
  connectedMidiPort: string | null;

  // Scope state
  scopeFrame: Float32Array | null;

  // Recording state
  isRecording: boolean;
  recordingDuration: number;

  // Undo/Redo state
  canUndo: boolean;
  canRedo: boolean;

  // Web Audio API context (for sample-accurate scheduling)
  audioContext: AudioContext | null;

  // WebSocket send function (set by the hook)
  sendIntent: ((intent: object) => void) | null;

  // Actions
  setSendIntent: (fn: (intent: object) => void) => void;
  setWsStatus: (s: WsStatus) => void;
  setMuted: (m: boolean) => void;
  setAudioActive: (a: boolean) => void;
  setSelectedNode: (id: string | null) => void;
  initAudioContext: () => AudioContext;

  // Graph visual actions (local only — for drag/layout)
  onNodesChange: (changes: NodeChange[]) => void;
  onEdgesChange: (changes: EdgeChange[]) => void;

  // Engine intents (go to host, then host responds with new snapshot)
  addNode: (nodeType: string) => void;
  removeNode: (nodeId: string) => void;
  connectNodes: (connection: Connection) => void;
  disconnectEdge: (edgeId: string) => void;
  updateParam: (
    nodeId: string,
    generation: number,
    paramIndex: number,
    value: number,
  ) => void;
  setOutputNode: (nodeId: string, generation: number) => void;
  toggleMute: () => void;
  clearGraph: () => void;
  loadPatch: (patch: object) => void;

  // MIDI actions
  listMidiPorts: () => void;
  connectMidiPort: (index: number) => void;
  setMidiPorts: (ports: string[]) => void;
  setConnectedMidiPort: (port: string | null) => void;

  // Scope actions
  setScopeFrame: (frame: Float32Array | null) => void;

  // Recording actions
  setIsRecording: (v: boolean) => void;
  setRecordingDuration: (v: number) => void;
  startRecording: (outputPath: string) => void;
  stopRecording: () => void;

  // Undo/Redo actions
  setCanUndo: (v: boolean) => void;
  setCanRedo: (v: boolean) => void;
  undo: () => void;
  redo: () => void;

  // Called when host sends a snapshot
  applySnapshot: (data: {
    nodes: Array<{
      id: number;
      generation: number;
      node_type: string;
      params: number[];
    }>;
    edges: Array<{ src_id: number; dst_id: number; slot: number }>;
    muted: boolean;
    output_node_id: number | null;
    can_undo?: boolean;
    can_redo?: boolean;
  }) => void;
}

// position counter unused — positions assigned in applySnapshot

export const useEngineStore = create<EngineStore>((set, get) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,
  wsStatus: "disconnected",
  muted: false,
  outputNodeId: null,
  audioActive: false,
  midiPorts: [],
  connectedMidiPort: null,
  isRecording: false,
  recordingDuration: 0,
  scopeFrame: null,
  canUndo: false,
  canRedo: false,
  audioContext: null,
  sendIntent: null,

  setSendIntent: (fn) => set({ sendIntent: fn }),
  setWsStatus: (wsStatus) => set({ wsStatus }),
  setMuted: (muted) => set({ muted }),
  setAudioActive: (audioActive) => set({ audioActive }),
  setSelectedNode: (selectedNodeId) => set({ selectedNodeId }),

  initAudioContext: () => {
    const existing = get().audioContext;
    if (existing) return existing;
    const ctx = new AudioContext();
    set({ audioContext: ctx });
    return ctx;
  },

  onNodesChange: (changes) =>
    set((s) => ({ nodes: applyNodeChanges(changes, s.nodes) })),

  onEdgesChange: (changes) =>
    set((s) => ({ edges: applyEdgeChanges(changes, s.edges) })),

  addNode: (nodeType) => {
    get().sendIntent?.({ type: "add_node", node_type: nodeType });
  },

  removeNode: (nodeId) => {
    const node = get().nodes.find((n) => n.id === nodeId);
    if (!node) return;
    get().sendIntent?.({
      type: "remove_node",
      node_id: parseInt(nodeId, 10),
      generation: node.data.generation ?? 0,
    });
  },

  connectNodes: (connection) => {
    const srcNode = get().nodes.find((n) => n.id === connection.source);
    const dstNode = get().nodes.find((n) => n.id === connection.target);
    if (!srcNode || !dstNode) return;
    const slot = parseInt(
      connection.targetHandle?.replace("input-", "") ?? "0",
      10,
    );
    get().sendIntent?.({
      type: "connect",
      src_id: parseInt(connection.source!, 10),
      src_gen: srcNode.data.generation ?? 0,
      dst_id: parseInt(connection.target!, 10),
      dst_gen: dstNode.data.generation ?? 0,
      slot,
    });
  },

  disconnectEdge: (edgeId) => {
    const edge = get().edges.find((e) => e.id === edgeId);
    if (!edge) return;
    const dstNode = get().nodes.find((n) => n.id === edge.target);
    if (!dstNode) return;
    const slot = parseInt(edge.targetHandle?.replace("input-", "") ?? "0", 10);
    get().sendIntent?.({
      type: "disconnect",
      dst_id: parseInt(edge.target, 10),
      dst_gen: dstNode.data.generation ?? 0,
      slot,
    });
  },

  updateParam: (nodeId, generation, paramIndex, value) => {
    get().sendIntent?.({
      type: "update_param",
      node_id: parseInt(nodeId, 10),
      generation,
      param_index: paramIndex,
      value,
      ramp_ms: 20,
    });
  },

  setOutputNode: (nodeId, generation) => {
    get().sendIntent?.({
      type: "set_output_node",
      node_id: parseInt(nodeId, 10),
      generation,
    });
  },

  toggleMute: () => {
    const muted = !get().muted;
    get().sendIntent?.({ type: "set_mute", muted });
  },

  clearGraph: () => {
    get().sendIntent?.({ type: "clear_graph" });
  },

  loadPatch: (patch) => {
    get().sendIntent?.({ type: "load_patch", patch });
  },

  listMidiPorts: () => {
    get().sendIntent?.({ type: "midi_list_ports" });
  },

  connectMidiPort: (index) => {
    get().sendIntent?.({ type: "midi_connect", port_index: index });
  },

  setMidiPorts: (ports) => set({ midiPorts: ports }),

  setConnectedMidiPort: (port) => set({ connectedMidiPort: port }),

  setScopeFrame: (scopeFrame) => set({ scopeFrame }),

  setIsRecording: (isRecording) => set({ isRecording }),

  setRecordingDuration: (recordingDuration) => set({ recordingDuration }),

  startRecording: (outputPath) => {
    get().sendIntent?.({ type: "start_recording", output_path: outputPath });
  },

  stopRecording: () => {
    get().sendIntent?.({ type: "stop_recording" });
  },

  setCanUndo: (canUndo) => set({ canUndo }),
  setCanRedo: (canRedo) => set({ canRedo }),

  undo: () => {
    get().sendIntent?.({ type: "undo" });
  },

  redo: () => {
    get().sendIntent?.({ type: "redo" });
  },

  applySnapshot: (data) => {
    const cols = 4;
    const nodes: Node[] = data.nodes.map((n, i) => {
      const def = NODE_DEFS[n.node_type];
      const col = i % cols;
      const row = Math.floor(i / cols);
      return {
        id: String(n.id),
        type: "studioNode",
        position: { x: 60 + col * 260, y: 80 + row * 300 },
        data: {
          nodeType: n.node_type,
          generation: n.generation,
          params: n.params,
          paramDefs: def?.params ?? [],
          inputCount: def?.inputs ?? 1,
          color: def?.color ?? "#4fc3f7",
          isOutput: data.output_node_id === n.id,
        },
      };
    });

    const edges: Edge[] = data.edges.map((e, i) => ({
      id: `e-${i}-${e.src_id}-${e.dst_id}-${e.slot}`,
      source: String(e.src_id),
      target: String(e.dst_id),
      targetHandle: `input-${e.slot}`,
      animated: !data.muted,
      style: { stroke: "#4fc3f7", strokeWidth: 1.5 },
    }));

    set({
      nodes,
      edges,
      muted: data.muted,
      outputNodeId:
        data.output_node_id !== null ? String(data.output_node_id) : null,
      canUndo: data.can_undo ?? false,
      canRedo: data.can_redo ?? false,
    });

    // Pulse audio indicator
    set({ audioActive: true });
    setTimeout(() => set({ audioActive: false }), 400);
  },
}));
