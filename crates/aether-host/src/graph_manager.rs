//! Graph manager — runs on the control thread.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use aether_core::{
    arena::NodeId,
    command::{EdgeSnapshot, NodeSnapshot},
    scheduler::Scheduler,
};
use aether_midi::event::MidiEvent;
use aether_nodes::{
    delay::DelayLine, envelope::AdsrEnvelope, filter::StateVariableFilter, gain::Gain,
    mixer::Mixer, oscillator::Oscillator, record::RecordNode, scope::ScopeNode,
};
use aether_sampler::node::SamplerNode;
use aether_sampler::instrument::{LoadedInstrument, SamplerInstrument};
use aether_timbre::node::TimbreTransferNode;
use ringbuf::{traits::Split, HeapCons, HeapRb};
use serde::{Deserialize, Serialize};

use crate::undo_stack::{StructuralIntent, UndoEntry, UndoStack};
use crate::wav_writer::WavWriterThread;

// ── Intent ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Intent {
    AddNode { node_type: String },
    RemoveNode { node_id: u32, generation: u32 },
    Connect { src_id: u32, src_gen: u32, dst_id: u32, dst_gen: u32, slot: usize },
    Disconnect { dst_id: u32, dst_gen: u32, slot: usize },
    UpdateParam { node_id: u32, generation: u32, param_index: usize, value: f32, ramp_ms: f32 },
    SetOutputNode { node_id: u32, generation: u32 },
    SetMute { muted: bool },
    ClearGraph,
    LoadPatch { patch: PatchDef },
    GetSnapshot,
    MidiListPorts,
    MidiConnect { port_index: usize },
    InjectMidi { channel: u8, note: u8, velocity: u8, is_note_on: bool },
    LoadInstrument { node_id: u32, generation: u32, instrument_json: String },
    #[allow(dead_code)] AnalyzeTimbre { instrument_json: String },
    #[allow(dead_code)] ApplyTimbreTransfer { timbre_transfer_node_id: u32, profile_name: String },
    ExportClap { #[allow(dead_code)] node_id: u32, #[allow(dead_code)] generation: u32, output_path: String },
    StartRecording { output_path: String },
    StopRecording,
    Undo,
    Redo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchDef {
    pub nodes: Vec<PatchNode>,
    pub connections: Vec<PatchConnection>,
    pub output_node: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default)]
    pub params: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchConnection {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub slot: usize,
}

// ── Response ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Snapshot { nodes: Vec<NodeSnapshot>, edges: Vec<EdgeSnapshot>, muted: bool, output_node_id: Option<u32>, can_undo: bool, can_redo: bool },
    Ack { command: String },
    Error { message: String },
    MidiPorts { ports: Vec<String> },
    ClapExported { path: String, size_bytes: u64 },
    RecordingStarted,
    RecordingStopped { duration_secs: f32 },
}

// ── NodeExtra ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub enum NodeExtra {
    None,
    MidiQueue(Arc<Mutex<Vec<MidiEvent>>>),
    ScopeConsumer(HeapCons<f32>),
    RecordConsumer(HeapCons<f32>),
    /// Instrument slot for a SamplerNode — shared Arc for loading instruments.
    SamplerSlot {
        midi_queue: Arc<Mutex<Vec<MidiEvent>>>,
        instrument_slot: Arc<Mutex<Option<LoadedInstrument>>>,
    },
}

// ── RecordingState ────────────────────────────────────────────────────────────

struct RecordingState {
    writer: WavWriterThread,
    start_instant: Instant,
    record_node_id: NodeId,
}

// ── GraphManager ──────────────────────────────────────────────────────────────

pub struct GraphManager {
    pub id_map: HashMap<String, NodeId>,
    pub node_labels: HashMap<u32, String>,
    pub output_node: Option<NodeId>,
    pub muted: bool,
    pub sample_rate: f32,
    pub midi_queues: Arc<Mutex<HashMap<NodeId, Arc<Mutex<Vec<MidiEvent>>>>>>,
    pub undo_stack: UndoStack,
    pub redo_stack: Vec<UndoEntry>,
    recording_state: Option<RecordingState>,
    /// Pending scope consumers produced by AddNode for ScopeNode.
    /// ws_server drains this after each handle() call.
    pub scope_consumers: HashMap<NodeId, HeapCons<f32>>,
    /// Instrument slots for SamplerNodes — used by LoadInstrument intent.
    pub instrument_slots: HashMap<NodeId, Arc<Mutex<Option<LoadedInstrument>>>>,
}

impl GraphManager {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            id_map: HashMap::new(),
            node_labels: HashMap::new(),
            output_node: None,
            muted: false,
            sample_rate,
            midi_queues: Arc::new(Mutex::new(HashMap::new())),
            undo_stack: UndoStack::new(),
            redo_stack: Vec::new(),
            recording_state: None,
            scope_consumers: HashMap::new(),
            instrument_slots: HashMap::new(),
        }
    }

    pub fn handle(&mut self, intent: Intent, scheduler: &mut Scheduler) -> Response {
        match intent {
            Intent::AddNode { node_type } => {
                let (node, extra) = match make_node(&node_type, scheduler.sample_rate) {
                    Some(p) => p,
                    None => return Response::Error { message: format!("Unknown node type: {node_type}") },
                };
                let node_id = match scheduler.graph.add_node(node) {
                    Some(id) => id,
                    None => return Response::Error { message: "Graph full".into() },
                };
                init_default_params(scheduler, node_id, &node_type);
                match extra {
                    NodeExtra::MidiQueue(q) => { self.midi_queues.lock().unwrap().insert(node_id, q); }
                    NodeExtra::ScopeConsumer(consumer) => { self.scope_consumers.insert(node_id, consumer); }
                    NodeExtra::SamplerSlot { midi_queue, instrument_slot } => {
                        self.midi_queues.lock().unwrap().insert(node_id, midi_queue);
                        self.instrument_slots.insert(node_id, instrument_slot);
                    }
                    _ => {}
                }
                let label = format!("{}-{}", node_type, node_id.index);
                self.id_map.insert(label.clone(), node_id);
                self.node_labels.insert(node_id.index, label);
                // Push undo entry
                let entry = UndoEntry {
                    forward: StructuralIntent::AddNode { node_type: node_type.clone(), created_id: node_id },
                    inverse: StructuralIntent::RemoveNode { node_id: node_id.index, generation: node_id.generation, node_type },
                };
                self.undo_stack.push(entry);
                self.redo_stack.clear();
                self.snapshot(scheduler)
            }

            Intent::RemoveNode { node_id, generation } => {
                let id = NodeId { index: node_id, generation };
                let node_type = self.node_labels.get(&node_id).cloned().unwrap_or_default();
                scheduler.graph.remove_node(id);
                if self.output_node == Some(id) { self.output_node = None; scheduler.graph.output_node = None; }
                self.node_labels.remove(&node_id);
                self.id_map.retain(|_, v| v.index != node_id);
                self.midi_queues.lock().unwrap().remove(&id);
                self.scope_consumers.remove(&id);
                self.instrument_slots.remove(&id);
                // Push undo entry
                let entry = UndoEntry {
                    forward: StructuralIntent::RemoveNode { node_id, generation, node_type: node_type.clone() },
                    inverse: StructuralIntent::AddNode { node_type, created_id: id },
                };
                self.undo_stack.push(entry);
                self.redo_stack.clear();
                self.snapshot(scheduler)
            }

            Intent::Connect { src_id, src_gen, dst_id, dst_gen, slot } => {
                // Capture previous source for the inverse (Disconnect needs prev_src)
                let dst = NodeId { index: dst_id, generation: dst_gen };
                let prev_src = scheduler.graph.arena.get(dst)
                    .and_then(|r| r.inputs.get(slot).copied().flatten());
                scheduler.graph.connect(NodeId { index: src_id, generation: src_gen }, dst, slot);
                // Push undo entry
                let inverse = if let Some(ps) = prev_src {
                    StructuralIntent::Connect { src_id: ps.index, src_gen: ps.generation, dst_id, dst_gen, slot }
                } else {
                    StructuralIntent::Disconnect { dst_id, dst_gen, slot, prev_src_id: src_id, prev_src_gen: src_gen }
                };
                let entry = UndoEntry {
                    forward: StructuralIntent::Connect { src_id, src_gen, dst_id, dst_gen, slot },
                    inverse,
                };
                self.undo_stack.push(entry);
                self.redo_stack.clear();
                self.snapshot(scheduler)
            }

            Intent::Disconnect { dst_id, dst_gen, slot } => {
                let dst = NodeId { index: dst_id, generation: dst_gen };
                // Capture the current source before disconnecting
                let prev_src = scheduler.graph.arena.get(dst)
                    .and_then(|r| r.inputs.get(slot).copied().flatten());
                scheduler.graph.disconnect(dst, slot);
                // Push undo entry
                if let Some(ps) = prev_src {
                    let entry = UndoEntry {
                        forward: StructuralIntent::Disconnect { dst_id, dst_gen, slot, prev_src_id: ps.index, prev_src_gen: ps.generation },
                        inverse: StructuralIntent::Connect { src_id: ps.index, src_gen: ps.generation, dst_id, dst_gen, slot },
                    };
                    self.undo_stack.push(entry);
                    self.redo_stack.clear();
                }
                self.snapshot(scheduler)
            }

            Intent::UpdateParam { node_id, generation, param_index, value, ramp_ms } => {
                let node = NodeId { index: node_id, generation };
                let ramp_samples = ((ramp_ms / 1000.0) * self.sample_rate) as u32;
                if let Some(record) = scheduler.graph.arena.get_mut(node) {
                    if param_index < record.params.count {
                        record.params.params[param_index].set_target(value, ramp_samples);
                    }
                }
                Response::Ack { command: "update_param".into() }
            }

            Intent::SetOutputNode { node_id, generation } => {
                let id = NodeId { index: node_id, generation };
                scheduler.graph.set_output_node(id);
                self.output_node = Some(id);
                self.snapshot(scheduler)
            }

            Intent::SetMute { muted } => {
                self.muted = muted; scheduler.muted = muted;
                self.snapshot(scheduler)
            }

            Intent::ClearGraph => {
                // Capture snapshot BEFORE clearing for the undo inverse
                let snapshot_before = self.build_patch_def(scheduler);
                let ids: Vec<_> = scheduler.graph.execution_order.clone();
                for id in ids { scheduler.graph.remove_node(id); }
                scheduler.graph.output_node = None; scheduler.muted = false;
                self.output_node = None; self.muted = false;
                self.id_map.clear(); self.node_labels.clear();
                self.midi_queues.lock().unwrap().clear();
                self.scope_consumers.clear();
                self.instrument_slots.clear();
                // Push undo entry
                let entry = UndoEntry {
                    forward: StructuralIntent::ClearGraph { snapshot_before: snapshot_before.clone() },
                    inverse: StructuralIntent::LoadPatch { patch: snapshot_before },
                };
                self.undo_stack.push(entry);
                self.redo_stack.clear();
                self.snapshot(scheduler)
            }

            Intent::LoadPatch { patch } => {
                // Capture snapshot BEFORE loading for the undo inverse
                let snapshot_before = self.build_patch_def(scheduler);
                let ids: Vec<_> = scheduler.graph.execution_order.clone();
                for id in ids { scheduler.graph.remove_node(id); }
                scheduler.graph.output_node = None;
                self.output_node = None; self.id_map.clear(); self.node_labels.clear();
                self.midi_queues.lock().unwrap().clear();

                let mut local_id_map: HashMap<String, NodeId> = HashMap::new();
                for pnode in &patch.nodes {
                    let (node, extra) = match make_node(&pnode.node_type, scheduler.sample_rate) {
                        Some(p) => p,
                        None => return Response::Error { message: format!("Unknown node type: {}", pnode.node_type) },
                    };
                    let node_id = match scheduler.graph.add_node(node) {
                        Some(id) => id,
                        None => return Response::Error { message: "Graph full".into() },
                    };
                    init_default_params(scheduler, node_id, &pnode.node_type);
                    apply_patch_params(scheduler, node_id, &pnode.node_type, &pnode.params);
                    if let NodeExtra::MidiQueue(q) = extra {
                        self.midi_queues.lock().unwrap().insert(node_id, q);
                    }
                    local_id_map.insert(pnode.id.clone(), node_id);
                    self.node_labels.insert(node_id.index, pnode.id.clone());
                }
                for conn in &patch.connections {
                    let src = match local_id_map.get(&conn.from) { Some(id) => *id, None => return Response::Error { message: format!("Unknown node id: {}", conn.from) } };
                    let dst = match local_id_map.get(&conn.to) { Some(id) => *id, None => return Response::Error { message: format!("Unknown node id: {}", conn.to) } };
                    scheduler.graph.connect(src, dst, conn.slot);
                }
                if let Some(out_id) = local_id_map.get(&patch.output_node) {
                    scheduler.graph.set_output_node(*out_id);
                    self.output_node = Some(*out_id);
                }
                self.id_map = local_id_map;
                // Push undo entry
                let entry = UndoEntry {
                    forward: StructuralIntent::LoadPatch { patch: patch.clone() },
                    inverse: StructuralIntent::LoadPatch { patch: snapshot_before },
                };
                self.undo_stack.push(entry);
                self.redo_stack.clear();
                self.snapshot(scheduler)
            }

            Intent::GetSnapshot => self.snapshot(scheduler),

            Intent::MidiListPorts => Response::Error { message: "MidiListPorts must be handled in ws_server".into() },
            Intent::MidiConnect { .. } => Response::Error { message: "MidiConnect must be handled in ws_server".into() },

            Intent::InjectMidi { channel, note, velocity, is_note_on } => {
                let kind = if is_note_on {
                    aether_midi::event::MidiEventKind::NoteOn { note, velocity }
                } else {
                    aether_midi::event::MidiEventKind::NoteOff { note, velocity }
                };
                let event = MidiEvent { timestamp: 0, channel, kind };
                let queues = self.midi_queues.lock().unwrap();
                for q in queues.values() { q.lock().unwrap().push(event.clone()); }
                Response::Ack { command: "inject_midi".into() }
            }

            Intent::LoadInstrument { node_id, generation, instrument_json } => {
                let id = NodeId { index: node_id, generation };

                // Find the instrument slot for this SamplerNode
                let slot = match self.instrument_slots.get(&id) {
                    Some(s) => Arc::clone(s),
                    None => return Response::Error {
                        message: format!("No SamplerNode found with id {node_id} — add a SamplerNode first"),
                    },
                };

                // Deserialize the instrument JSON
                let instrument: SamplerInstrument = match serde_json::from_str(&instrument_json) {
                    Ok(i) => i,
                    Err(e) => return Response::Error {
                        message: format!("Failed to parse instrument JSON: {e}"),
                    },
                };

                // Load audio buffers from disk.
                // Zones with embedded audio data (audioDataUrl) are browser-only and skipped here.
                // Zones with file_path are loaded from disk relative to the current directory.
                let mut buffers = std::collections::HashMap::new();
                let mut release_buffers = std::collections::HashMap::new();
                let base_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

                for zone in &instrument.zones {
                    let path = if std::path::Path::new(&zone.file_path).is_absolute() {
                        std::path::PathBuf::from(&zone.file_path)
                    } else {
                        base_dir.join(&zone.file_path)
                    };

                    if path.exists() {
                        match aether_sampler::buffer::SampleBuffer::load_wav(&path) {
                            Ok(buf) => { buffers.insert(zone.id.clone(), buf); }
                            Err(e) => eprintln!("LoadInstrument: failed to load '{}': {e}", path.display()),
                        }
                    } else {
                        eprintln!("LoadInstrument: zone '{}' file not found: {}", zone.id, path.display());
                    }

                    if let Some(ref rel_path) = zone.release_file {
                        let rpath = if std::path::Path::new(rel_path).is_absolute() {
                            std::path::PathBuf::from(rel_path)
                        } else {
                            base_dir.join(rel_path)
                        };
                        if rpath.exists() {
                            if let Ok(rbuf) = aether_sampler::buffer::SampleBuffer::load_wav(&rpath) {
                                release_buffers.insert(zone.id.clone(), rbuf);
                            }
                        }
                    }
                }

                let loaded = LoadedInstrument {
                    instrument,
                    buffers,
                    release_buffers,
                };

                // Push into the SamplerNode's instrument slot
                *slot.lock().unwrap() = Some(loaded);

                Response::Ack { command: "load_instrument".into() }
            }            Intent::AnalyzeTimbre { .. } => Response::Ack { command: "analyze_timbre".into() },
            Intent::ApplyTimbreTransfer { .. } => Response::Ack { command: "apply_timbre_transfer".into() },
            Intent::ExportClap { node_id: _, generation: _, output_path } => export_clap(output_path),

            Intent::StartRecording { output_path } => {
                if self.recording_state.is_some() {
                    return Response::Error { message: "Already recording".into() };
                }
                let rb = HeapRb::<f32>::new(96_000);
                let (producer, consumer) = rb.split();
                let record_node = RecordNode::new(producer);
                let record_node_id = match scheduler.graph.add_node(Box::new(record_node)) {
                    Some(id) => id,
                    None => return Response::Error { message: "Graph full".into() },
                };
                let writer = WavWriterThread::spawn(consumer, output_path, self.sample_rate as u32);
                self.recording_state = Some(RecordingState { writer, start_instant: Instant::now(), record_node_id });
                Response::RecordingStarted
            }

            Intent::StopRecording => {
                match self.recording_state.take() {
                    None => Response::Ack { command: "stop_noop".into() },
                    Some(state) => {
                        let duration_secs = state.start_instant.elapsed().as_secs_f32();
                        state.writer.stop();
                        scheduler.graph.remove_node(state.record_node_id);
                        Response::RecordingStopped { duration_secs }
                    }
                }
            }

            Intent::Undo => {
                match self.undo_stack.pop() {
                    None => Response::Ack { command: "undo_noop".into() },
                    Some(entry) => {
                        self.apply_structural_intent(&entry.inverse, scheduler);
                        self.redo_stack.push(entry);
                        self.snapshot(scheduler)
                    }
                }
            }

            Intent::Redo => {
                match self.redo_stack.pop() {
                    None => Response::Ack { command: "redo_noop".into() },
                    Some(entry) => {
                        self.apply_structural_intent(&entry.forward, scheduler);
                        self.undo_stack.push(entry);
                        self.snapshot(scheduler)
                    }
                }
            }
        }
    }

    pub fn snapshot(&self, scheduler: &Scheduler) -> Response {
        let nodes: Vec<NodeSnapshot> = scheduler.graph.execution_order.iter()
            .filter_map(|id| scheduler.graph.arena.get(*id).map(|r| NodeSnapshot {
                id: id.index, generation: id.generation,
                node_type: r.processor.type_name().to_string(),
                params: r.params.params[..r.params.count].iter().map(|p| p.current).collect(),
            }))
            .collect();

        let mut edges: Vec<EdgeSnapshot> = Vec::new();
        for id in &scheduler.graph.execution_order {
            if let Some(record) = scheduler.graph.arena.get(*id) {
                for (slot, maybe_src) in record.inputs.iter().enumerate() {
                    if let Some(src_id) = maybe_src {
                        edges.push(EdgeSnapshot { src_id: src_id.index, dst_id: id.index, slot });
                    }
                }
            }
        }
        Response::Snapshot {
            nodes,
            edges,
            muted: self.muted,
            output_node_id: self.output_node.map(|id| id.index),
            can_undo: !self.undo_stack.is_empty(),
            can_redo: !self.redo_stack.is_empty(),
        }
    }

    /// Build a `PatchDef` from the current graph state (used for undo snapshots).
    pub fn build_patch_def(&self, scheduler: &Scheduler) -> PatchDef {
        let param_names_for = |node_type: &str| -> &[&str] {
            match node_type {
                "Oscillator" => &["Frequency", "Amplitude", "Waveform"],
                "StateVariableFilter" => &["Cutoff", "Resonance", "Mode"],
                "AdsrEnvelope" => &["Attack", "Decay", "Sustain", "Release", "Gate"],
                "DelayLine" => &["Time", "Feedback", "Wet"],
                "Gain" => &["Gain"],
                "TimbreTransferNode" => &["Amount"],
                _ => &[],
            }
        };

        let mut nodes = Vec::new();
        for id in &scheduler.graph.execution_order {
            if let Some(record) = scheduler.graph.arena.get(*id) {
                let node_type = record.processor.type_name().to_string();
                let label = self.node_labels.get(&id.index).cloned()
                    .unwrap_or_else(|| format!("{}-{}", node_type, id.index));
                let names = param_names_for(&node_type);
                let mut params = HashMap::new();
                for (i, name) in names.iter().enumerate() {
                    if i < record.params.count {
                        params.insert(name.to_string(), record.params.params[i].current);
                    }
                }
                nodes.push(PatchNode { id: label, node_type, params });
            }
        }

        let mut connections = Vec::new();
        for id in &scheduler.graph.execution_order {
            if let Some(record) = scheduler.graph.arena.get(*id) {
                let dst_label = self.node_labels.get(&id.index).cloned()
                    .unwrap_or_else(|| format!("node-{}", id.index));
                for (slot, maybe_src) in record.inputs.iter().enumerate() {
                    if let Some(src_id) = maybe_src {
                        let src_label = self.node_labels.get(&src_id.index).cloned()
                            .unwrap_or_else(|| format!("node-{}", src_id.index));
                        connections.push(PatchConnection { from: src_label, to: dst_label.clone(), slot });
                    }
                }
            }
        }

        let output_node = self.output_node
            .and_then(|id| self.node_labels.get(&id.index).cloned())
            .unwrap_or_default();

        PatchDef { nodes, connections, output_node }
    }

    /// Apply a `StructuralIntent` directly to the scheduler graph (used by undo/redo).
    fn apply_structural_intent(&mut self, intent: &StructuralIntent, scheduler: &mut Scheduler) {
        match intent {
            StructuralIntent::AddNode { node_type, created_id } => {
                let (node, extra) = match make_node(node_type, scheduler.sample_rate) {
                    Some(p) => p,
                    None => return,
                };
                // Try to add the node; we can't guarantee the same id but we do our best
                let node_id = match scheduler.graph.add_node(node) {
                    Some(id) => id,
                    None => return,
                };
                let _ = created_id; // id may differ after undo/redo cycles
                init_default_params(scheduler, node_id, node_type);
                match extra {
                    NodeExtra::MidiQueue(q) => { self.midi_queues.lock().unwrap().insert(node_id, q); }
                    NodeExtra::SamplerSlot { midi_queue, instrument_slot } => {
                        self.midi_queues.lock().unwrap().insert(node_id, midi_queue);
                        self.instrument_slots.insert(node_id, instrument_slot);
                    }
                    _ => {}
                }
                let label = format!("{}-{}", node_type, node_id.index);
                self.id_map.insert(label.clone(), node_id);
                self.node_labels.insert(node_id.index, label);
            }
            StructuralIntent::RemoveNode { node_id, generation, node_type: _ } => {
                let id = NodeId { index: *node_id, generation: *generation };
                scheduler.graph.remove_node(id);
                if self.output_node == Some(id) { self.output_node = None; scheduler.graph.output_node = None; }
                self.node_labels.remove(node_id);
                self.id_map.retain(|_, v| v.index != *node_id);
                self.midi_queues.lock().unwrap().remove(&id);
            }
            StructuralIntent::Connect { src_id, src_gen, dst_id, dst_gen, slot } => {
                scheduler.graph.connect(
                    NodeId { index: *src_id, generation: *src_gen },
                    NodeId { index: *dst_id, generation: *dst_gen },
                    *slot,
                );
            }
            StructuralIntent::Disconnect { dst_id, dst_gen, slot, .. } => {
                scheduler.graph.disconnect(NodeId { index: *dst_id, generation: *dst_gen }, *slot);
            }
            StructuralIntent::LoadPatch { patch } => {
                // Clear current graph
                let ids: Vec<_> = scheduler.graph.execution_order.clone();
                for id in ids { scheduler.graph.remove_node(id); }
                scheduler.graph.output_node = None;
                self.output_node = None; self.id_map.clear(); self.node_labels.clear();
                self.midi_queues.lock().unwrap().clear();

                let mut local_id_map: HashMap<String, NodeId> = HashMap::new();
                for pnode in &patch.nodes {
                    let (node, extra) = match make_node(&pnode.node_type, scheduler.sample_rate) {
                        Some(p) => p,
                        None => continue,
                    };
                    let node_id = match scheduler.graph.add_node(node) {
                        Some(id) => id,
                        None => continue,
                    };
                    init_default_params(scheduler, node_id, &pnode.node_type);
                    apply_patch_params(scheduler, node_id, &pnode.node_type, &pnode.params);
                    match extra {
                        NodeExtra::MidiQueue(q) => { self.midi_queues.lock().unwrap().insert(node_id, q); }
                        NodeExtra::SamplerSlot { midi_queue, instrument_slot } => {
                            self.midi_queues.lock().unwrap().insert(node_id, midi_queue);
                            self.instrument_slots.insert(node_id, instrument_slot);
                        }
                        _ => {}
                    }
                    local_id_map.insert(pnode.id.clone(), node_id);
                    self.node_labels.insert(node_id.index, pnode.id.clone());
                }
                for conn in &patch.connections {
                    let src = match local_id_map.get(&conn.from) { Some(id) => *id, None => continue };
                    let dst = match local_id_map.get(&conn.to) { Some(id) => *id, None => continue };
                    scheduler.graph.connect(src, dst, conn.slot);
                }
                if let Some(out_id) = local_id_map.get(&patch.output_node) {
                    scheduler.graph.set_output_node(*out_id);
                    self.output_node = Some(*out_id);
                }
                self.id_map = local_id_map;
            }
            StructuralIntent::ClearGraph { .. } => {
                let ids: Vec<_> = scheduler.graph.execution_order.clone();
                for id in ids { scheduler.graph.remove_node(id); }
                scheduler.graph.output_node = None; scheduler.muted = false;
                self.output_node = None; self.muted = false;
                self.id_map.clear(); self.node_labels.clear();
                self.midi_queues.lock().unwrap().clear();
                self.instrument_slots.clear();
            }
        }
    }
}

// ── Node factory ──────────────────────────────────────────────────────────────

fn make_node(node_type: &str, _sample_rate: f32) -> Option<(Box<dyn aether_core::node::DspNode>, NodeExtra)> {
    match node_type {
        "Oscillator" => Some((Box::new(Oscillator::new()), NodeExtra::None)),
        "StateVariableFilter" => Some((Box::new(StateVariableFilter::new()), NodeExtra::None)),
        "AdsrEnvelope" => Some((Box::new(AdsrEnvelope::new()), NodeExtra::None)),
        "DelayLine" => Some((Box::new(DelayLine::new()), NodeExtra::None)),
        "Gain" => Some((Box::new(Gain), NodeExtra::None)),
        "Mixer" => Some((Box::new(Mixer), NodeExtra::None)),
        "SamplerNode" => {
            let node = SamplerNode::new(_sample_rate);
            let queue = node.midi_queue();
            let slot = node.instrument_slot();
            Some((Box::new(node), NodeExtra::SamplerSlot { midi_queue: queue, instrument_slot: slot }))
        }
        "TimbreTransferNode" => Some((Box::new(TimbreTransferNode::new()), NodeExtra::None)),
        "RecordNode" => {
            let (producer, consumer) = HeapRb::<f32>::new(96_000).split();
            Some((Box::new(RecordNode::new(producer)), NodeExtra::RecordConsumer(consumer)))
        }
        "ScopeNode" => {
            let (producer, consumer) = HeapRb::<f32>::new(512).split();
            Some((Box::new(ScopeNode::new(producer)), NodeExtra::ScopeConsumer(consumer)))
        }
        _ => None,
    }
}

fn init_default_params(scheduler: &mut Scheduler, id: NodeId, node_type: &str) {
    let defaults: &[f32] = match node_type {
        "Oscillator" => &[440.0, 0.5, 0.0],
        "StateVariableFilter" => &[2000.0, 1.0, 0.0],
        "AdsrEnvelope" => &[0.01, 0.1, 0.7, 0.3, 1.0],
        "DelayLine" => &[0.25, 0.4, 0.5],
        "Gain" => &[0.8],
        "Mixer" | "SamplerNode" | "RecordNode" | "ScopeNode" => &[],
        "TimbreTransferNode" => &[1.0],
        _ => &[],
    };
    if let Some(record) = scheduler.graph.arena.get_mut(id) {
        for &v in defaults { record.params.add(v); }
    }
}

fn apply_patch_params(scheduler: &mut Scheduler, id: NodeId, node_type: &str, overrides: &HashMap<String, f32>) {
    let param_names: &[&str] = match node_type {
        "Oscillator" => &["Frequency", "Amplitude", "Waveform"],
        "StateVariableFilter" => &["Cutoff", "Resonance", "Mode"],
        "AdsrEnvelope" => &["Attack", "Decay", "Sustain", "Release", "Gate"],
        "DelayLine" => &["Time", "Feedback", "Wet"],
        "Gain" => &["Gain"],
        "TimbreTransferNode" => &["Amount"],
        _ => &[],
    };
    if let Some(record) = scheduler.graph.arena.get_mut(id) {
        for (i, name) in param_names.iter().enumerate() {
            if let Some(&v) = overrides.get(*name) {
                if i < record.params.count {
                    record.params.params[i].current = v;
                    record.params.params[i].target = v;
                    record.params.params[i].step = 0.0;
                }
            }
        }
    }
}

// ── CLAP export ───────────────────────────────────────────────────────────────

fn export_clap(output_path: String) -> Response {
    use std::process::Command;
    let instrument_json = r#"{"name":"Exported Instrument","origin":"","description":"","author":"","tuning":{"name":"12-TET","frequencies":[8.18,8.66,9.18,9.72,10.30,10.91,11.56,12.25,12.98,13.75,14.57,15.43,16.35,17.32,18.35,19.45,20.60,21.83,23.12,24.50,25.96,27.50,29.14,30.87,32.70,34.65,36.71,38.89,41.20,43.65,46.25,49.00,51.91,55.00,58.27,61.74,65.41,69.30,73.42,77.78,82.41,87.31,92.50,98.00,103.83,110.00,116.54,123.47,130.81,138.59,146.83,155.56,164.81,174.61,185.00,196.00,207.65,220.00,233.08,246.94,261.63,277.18,293.66,311.13,329.63,349.23,369.99,392.00,415.30,440.00,466.16,493.88,523.25,554.37,587.33,622.25,659.25,698.46,739.99,783.99,830.61,880.00,932.33,987.77,1046.50,1108.73,1174.66,1244.51,1318.51,1396.91,1479.98,1567.98,1661.22,1760.00,1864.66,1975.53,2093.00,2217.46,2349.32,2489.02,2637.02,2793.83,2959.96,3135.96,3322.44,3520.00,3729.31,3951.07,4186.01,4434.92,4698.63,4978.03,5274.04,5587.65,5919.91,6271.93,6644.88,7040.00,7458.62,7902.13,8372.02,8869.84,9397.27,9956.06,10548.08,11175.30,11839.82,12543.85,13289.75,14080.00,14917.24,15804.27]},"zones":[],"attack":0.005,"decay":0.1,"sustain":0.8,"release":0.3,"max_voices":16}"#;
    let workspace_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let status = Command::new("cargo").args(["build", "-p", "aether-plugin", "--release"]).env("AETHER_INSTRUMENT_JSON", instrument_json).current_dir(&workspace_root).status();
    let status = match status { Ok(s) => s, Err(e) => return Response::Error { message: format!("Failed to invoke cargo build: {e}") } };
    if !status.success() { return Response::Error { message: format!("cargo build failed: {:?}", status.code()) }; }
    let clap_src = workspace_root.join("target").join("release").join("aether_plugin.clap");
    if !clap_src.exists() { return Response::Error { message: format!("Build succeeded but .clap not found at: {}", clap_src.display()) }; }
    let dest = std::path::Path::new(&output_path);
    if let Some(parent) = dest.parent() { if let Err(e) = std::fs::create_dir_all(parent) { return Response::Error { message: format!("Failed to create output dir: {e}") }; } }
    if let Err(e) = std::fs::copy(&clap_src, dest) { return Response::Error { message: format!("Failed to copy .clap: {e}") }; }
    let size_bytes = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    Response::ClapExported { path: output_path, size_bytes }
}
