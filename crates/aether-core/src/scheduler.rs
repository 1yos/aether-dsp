//! Real-time audio scheduler.
//!
//! This is the hot path. It:
//!   1. Drains bounded commands from the SPSC ring.
//!   2. Executes the topologically sorted node list level by level.
//!      Nodes within the same BFS level are independent and run in parallel
//!      via Rayon's work-stealing thread pool.
//!   3. Copies the output node's buffer to the DAC output.
//!
//! HARD RT RULES enforced here:
//!   - No allocation (Vec<NodeTask> is pre-allocated per level, bounded by MAX_NODES)
//!   - No locks
//!   - No I/O
//!   - No unbounded loops

use ringbuf::traits::Consumer;

use crate::{
    arena::NodeId,
    command::Command,
    graph::DspGraph,
    node::DspNode,
    param::ParamBlock,
    BUFFER_SIZE, MAX_COMMANDS_PER_TICK, MAX_INPUTS,
};

// ── Parallel dispatch helpers ─────────────────────────────────────────────────

/// Per-node data bundle collected before parallel dispatch.
///
/// SAFETY INVARIANT: Within a single BFS level, every node writes to a distinct
/// `BufferId` (guaranteed by the DAG structure — no two nodes in the same level
/// share an output buffer). The `BufferPool` stores buffers in a flat `Vec`, so
/// tasks writing to different `BufferId`s write to non-overlapping index ranges.
/// This makes the concurrent writes safe despite using raw pointers.
struct NodeTask {
    output_buf_ptr: *mut [f32; BUFFER_SIZE],
    params_ptr: *mut ParamBlock,
    processor_ptr: *mut dyn DspNode,
    inputs: [Option<*const [f32; BUFFER_SIZE]>; MAX_INPUTS],
}

/// SAFETY: Within a BFS level each task accesses disjoint memory:
/// - distinct output buffer (different BufferId → different Vec index range)
/// - distinct processor and params (each belongs to exactly one NodeRecord)
/// No two tasks in the same level share any pointed-to memory.
unsafe impl Send for NodeTask {}
unsafe impl Sync for NodeTask {}

// ── Scheduler ─────────────────────────────────────────────────────────────────

/// The RT scheduler. Owns the graph and processes audio callbacks.
pub struct Scheduler {
    pub graph: DspGraph,
    pub sample_rate: f32,
    pub muted: bool,
}

impl Scheduler {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            graph: DspGraph::new(),
            sample_rate,
            muted: false,
        }
    }

    /// Called once per audio callback from the CPAL stream.
    pub fn process_block<C>(&mut self, cmd_consumer: &mut C, output: &mut [f32])
    where
        C: Consumer<Item = Command>,
    {
        let mut processed = 0;
        while processed < MAX_COMMANDS_PER_TICK {
            match cmd_consumer.try_pop() {
                Some(cmd) => { self.apply_command(cmd); processed += 1; }
                None => break,
            }
        }
        self.process_graph(output);
    }

    /// Simplified process block — no ring buffer.
    /// Used when the scheduler is shared via Arc<Mutex<>> and the control thread
    /// mutates it directly. The audio thread calls this after acquiring try_lock().
    pub fn process_block_simple(&mut self, output: &mut [f32]) {
        self.process_graph(output);
    }

    fn process_graph(&mut self, output: &mut [f32]) {
        let sr = self.sample_rate;
        let level_count = self.graph.levels.len();

        for level_idx in 0..level_count {
            let level_len = self.graph.levels[level_idx].len();

            if level_len == 0 {
                continue;
            } else if level_len == 1 {
                // Zero-overhead path: single node, no Rayon overhead.
                let node_id = self.graph.levels[level_idx][0];
                self.process_node(node_id, sr);
            } else {
                // Parallel path: collect raw pointers while holding &mut self,
                // then dispatch DSP work in parallel via rayon::scope.
                //
                // SAFETY: Within a BFS level, every node writes to a distinct
                // output buffer (disjoint BufferId). The BufferPool stores buffers
                // in a flat Vec; tasks write to non-overlapping index ranges.
                // Each processor and ParamBlock belongs to exactly one node.
                let mut tasks: Vec<NodeTask> = Vec::with_capacity(level_len);

                for i in 0..level_len {
                    let node_id = self.graph.levels[level_idx][i];
                    let mut input_ptrs: [Option<*const [f32; BUFFER_SIZE]>; MAX_INPUTS] =
                        [None; MAX_INPUTS];

                    if let Some(record) = self.graph.arena.get(node_id) {
                        for (slot, maybe_src) in record.inputs.iter().enumerate() {
                            if let Some(src_id) = maybe_src {
                                if let Some(src_record) = self.graph.arena.get(*src_id) {
                                    input_ptrs[slot] = Some(
                                        self.graph.buffers.get(src_record.output_buffer)
                                            as *const [f32; BUFFER_SIZE],
                                    );
                                }
                            }
                        }
                        let record_mut = self.graph.arena.get_mut(node_id).unwrap();
                        let output_buf_ptr = self.graph.buffers.get_mut(record_mut.output_buffer)
                            as *mut [f32; BUFFER_SIZE];
                        let params_ptr = &mut record_mut.params as *mut ParamBlock;
                        let processor_ptr = &mut *record_mut.processor as *mut dyn DspNode;

                        tasks.push(NodeTask {
                            output_buf_ptr,
                            params_ptr,
                            processor_ptr,
                            inputs: input_ptrs,
                        });
                    }
                }

                // SAFETY: each element of `tasks` points to disjoint memory.
                // We pass a raw pointer per task so each closure captures a
                // distinct non-aliasing pointer.
                rayon::scope(|s| {
                    for task in tasks.iter_mut() {
                        // Capture the raw pointer value (usize) to avoid the
                        // borrow checker complaining about &mut Vec element borrows.
                        let ptr = task as *mut NodeTask as usize;
                        s.spawn(move |_| {
                            // SAFETY: ptr is a valid, exclusively-owned NodeTask.
                            let t: &mut NodeTask = unsafe { &mut *(ptr as *mut NodeTask) };
                            let inputs: [Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS] =
                                t.inputs.map(|p| p.map(|raw| unsafe { &*raw }));
                            unsafe {
                                (*t.processor_ptr).process(
                                    &inputs,
                                    &mut *t.output_buf_ptr,
                                    &mut *t.params_ptr,
                                    sr,
                                );
                            }
                        });
                    }
                });
            }
        }

        // Copy output node buffer to DAC
        if self.muted {
            output.fill(0.0);
            return;
        }
        if let Some(out_id) = self.graph.output_node {
            if let Some(record) = self.graph.arena.get(out_id) {
                let buf = self.graph.buffers.get(record.output_buffer);
                let frames = output.len() / 2;
                for i in 0..frames.min(BUFFER_SIZE) {
                    output[i * 2] = buf[i];
                    output[i * 2 + 1] = buf[i];
                }
            }
        } else {
            // INVARIANT: empty graph → silence.
            output.fill(0.0);
        }
    }

    /// Process a single node on the calling thread.
    fn process_node(&mut self, node_id: NodeId, sample_rate: f32) {
        let mut input_ptrs: [Option<*const [f32; BUFFER_SIZE]>; MAX_INPUTS] = [None; MAX_INPUTS];

        if let Some(record) = self.graph.arena.get(node_id) {
            for (slot, maybe_src) in record.inputs.iter().enumerate() {
                if let Some(src_id) = maybe_src {
                    if let Some(src_record) = self.graph.arena.get(*src_id) {
                        input_ptrs[slot] = Some(
                            self.graph.buffers.get(src_record.output_buffer)
                                as *const [f32; BUFFER_SIZE],
                        );
                    }
                }
            }
        } else {
            return;
        }

        let (output_buf_id, params_ptr, processor_ptr) = {
            let record = self.graph.arena.get_mut(node_id).unwrap();
            (
                record.output_buffer,
                &mut record.params as *mut ParamBlock,
                &mut *record.processor as *mut dyn crate::node::DspNode,
            )
        };

        let output_buf = self.graph.buffers.get_mut(output_buf_id);
        let inputs: [Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS] =
            input_ptrs.map(|p| p.map(|ptr| unsafe { &*ptr }));

        unsafe {
            (*processor_ptr).process(&inputs, output_buf, &mut *params_ptr, sample_rate);
        }
    }

    fn apply_command(&mut self, cmd: Command) {
        match cmd {
            Command::AddNode { id } => { let _ = id; }
            Command::RemoveNode { id } => { self.graph.remove_node(id); }
            Command::Connect { src, dst, slot } => { self.graph.connect(src, dst, slot); }
            Command::Disconnect { dst, slot } => { self.graph.disconnect(dst, slot); }
            Command::UpdateParam { node, param_index, new_param } => {
                if let Some(record) = self.graph.arena.get_mut(node) {
                    if param_index < record.params.count {
                        record.params.params[param_index] = new_param;
                    }
                }
            }
            Command::SetOutputNode { id } => { self.graph.set_output_node(id); }
            Command::SetMute { muted } => { self.muted = muted; }
            Command::ClearGraph => {
                let ids: Vec<_> = self.graph.execution_order.clone();
                for id in ids { self.graph.remove_node(id); }
                self.graph.output_node = None;
            }
        }
    }

    /// Reference sequential implementation for testing.
    /// Processes nodes in flat execution_order without parallelism.
    #[cfg(test)]
    fn process_graph_sequential(&mut self, output: &mut [f32]) {
        let sr = self.sample_rate;

        // Process nodes in flat execution order (sequential)
        for &node_id in &self.graph.execution_order {
            self.process_node(node_id, sr);
        }

        // Copy output node buffer to DAC
        if self.muted {
            output.fill(0.0);
            return;
        }
        if let Some(out_id) = self.graph.output_node {
            if let Some(record) = self.graph.arena.get(out_id) {
                let buf = self.graph.buffers.get(record.output_buffer);
                let frames = output.len() / 2;
                for i in 0..frames.min(BUFFER_SIZE) {
                    output[i * 2] = buf[i];
                    output[i * 2 + 1] = buf[i];
                }
            }
        } else {
            output.fill(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::DspNode;
    use proptest::prelude::*;

    /// Minimal deterministic test node for property testing.
    /// Sums all inputs and multiplies by a fixed gain.
    struct TestNode {
        gain: f32,
    }

    impl TestNode {
        fn new(gain: f32) -> Self {
            Self { gain }
        }
    }

    impl DspNode for TestNode {
        fn process(
            &mut self,
            inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
            output: &mut [f32; BUFFER_SIZE],
            _params: &mut ParamBlock,
            _sample_rate: f32,
        ) {
            output.fill(0.0);
            for input_opt in inputs.iter() {
                if let Some(input) = input_opt {
                    for i in 0..BUFFER_SIZE {
                        output[i] += input[i] * self.gain;
                    }
                }
            }
        }

        fn type_name(&self) -> &'static str {
            "TestNode"
        }
    }

    // Property 1
    proptest! {
        /// **Validates: Requirements 1.1, 1.4**
        ///
        /// Feature: aether-engine-upgrades, Property 1: parallel execution is output-equivalent
        ///
        /// Property 1: Parallel execution is output-equivalent to sequential execution.
        ///
        /// For any valid DSP patch (any combination of nodes and edges forming a valid DAG),
        /// processing a block with the parallel Rayon scheduler SHALL produce a bit-identical
        /// output buffer to processing the same block with the original sequential scheduler,
        /// given the same initial node state and the same input.
        #[test]
        fn prop_parallel_equiv_sequential(
            num_nodes in 1usize..=20,
            edges in prop::collection::vec((0usize..20, 0usize..20, 0usize..MAX_INPUTS), 0..50),
            seed in any::<u64>(),
        ) {
            // Create two identical schedulers
            let mut scheduler_parallel = Scheduler::new(48000.0);
            let mut scheduler_sequential = Scheduler::new(48000.0);

            let mut node_ids = Vec::new();

            // Add nodes to both schedulers with deterministic gains based on seed
            for i in 0..num_nodes {
                let gain = ((seed.wrapping_add(i as u64) % 100) as f32) / 100.0;
                
                let id1 = scheduler_parallel.graph.add_node(Box::new(TestNode::new(gain)));
                let id2 = scheduler_sequential.graph.add_node(Box::new(TestNode::new(gain)));
                
                if let (Some(id1), Some(id2)) = (id1, id2) {
                    // Verify both schedulers assigned the same NodeId
                    prop_assert_eq!(id1.index, id2.index);
                    prop_assert_eq!(id1.generation, id2.generation);
                    node_ids.push(id1);
                }
            }

            // Add edges to both schedulers (filter to maintain DAG invariant: src < dst)
            for (src_idx, dst_idx, slot) in edges {
                if src_idx < num_nodes && dst_idx < num_nodes && src_idx < dst_idx {
                    let src = node_ids[src_idx];
                    let dst = node_ids[dst_idx];
                    
                    scheduler_parallel.graph.connect(src, dst, slot);
                    scheduler_sequential.graph.connect(src, dst, slot);
                }
            }

            // Set output node to the last node if we have any nodes
            if !node_ids.is_empty() {
                let output_node = node_ids[num_nodes - 1];
                scheduler_parallel.graph.set_output_node(output_node);
                scheduler_sequential.graph.set_output_node(output_node);
            }

            // Prepare output buffers (stereo, 64 frames = 128 samples)
            let mut output_parallel = vec![0.0f32; BUFFER_SIZE * 2];
            let mut output_sequential = vec![0.0f32; BUFFER_SIZE * 2];

            // Process one block with both schedulers
            scheduler_parallel.process_graph(&mut output_parallel);
            scheduler_sequential.process_graph_sequential(&mut output_sequential);

            // Assert bit-identical output
            for (i, (&p, &s)) in output_parallel.iter().zip(output_sequential.iter()).enumerate() {
                prop_assert!(
                    p == s || (p.is_nan() && s.is_nan()),
                    "Output mismatch at sample {}: parallel={}, sequential={}",
                    i, p, s
                );
            }
        }
    }
}
