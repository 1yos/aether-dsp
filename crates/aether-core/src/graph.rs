//! Directed Acyclic Graph (DAG) for DSP routing.
//!
//! The graph owns the node arena and buffer pool.
//! Topological sort produces a flat execution order — no recursion in the RT path.

use crate::{
    arena::{Arena, NodeId},
    buffer_pool::BufferPool,
    node::{DspNode, NodeRecord},
    MAX_NODES,
};
use std::collections::HashMap;

/// Directed Acyclic Graph (DAG) for DSP routing.
///
/// The graph owns the node arena and buffer pool. It maintains a topologically
/// sorted execution order and BFS level structure for parallel processing.
///
/// # Structure
///
/// - **Arena**: Generational arena storing node records
/// - **Buffer Pool**: Pre-allocated audio buffers (no RT allocation)
/// - **Execution Order**: Flat topologically sorted node list
/// - **BFS Levels**: Nodes grouped by dependency depth for parallel execution
///
/// # Example
///
/// ```
/// use aether_core::graph::DspGraph;
/// use aether_core::node::DspNode;
/// use aether_core::param::ParamBlock;
/// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
///
/// struct Gain { gain: f32 }
/// impl DspNode for Gain {
///     fn process(&mut self, inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
///                output: &mut [f32; BUFFER_SIZE], _params: &mut ParamBlock, _sr: f32) {
///         if let Some(input) = inputs[0] {
///             for (i, out) in output.iter_mut().enumerate() {
///                 *out = input[i] * self.gain;
///             }
///         }
///     }
///     fn type_name(&self) -> &'static str { "Gain" }
/// }
///
/// let mut graph = DspGraph::new();
/// let gain_id = graph.add_node(Box::new(Gain { gain: 0.5 })).unwrap();
/// graph.set_output_node(gain_id);
/// ```
pub struct DspGraph {
    pub arena: Arena<NodeRecord>,
    pub buffers: BufferPool,
    /// Topologically sorted execution order. Rebuilt on structural mutations.
    pub execution_order: Vec<NodeId>,
    /// BFS wave levels: each inner Vec contains nodes that can execute in parallel.
    /// Level[i] nodes all depend only on nodes in levels 0..i.
    pub levels: Vec<Vec<NodeId>>,
    /// The node whose output buffer is sent to the DAC.
    pub output_node: Option<NodeId>,
    /// Adjacency list: node index → list of (dst_node, slot) it feeds into.
    forward_edges: HashMap<u32, Vec<(NodeId, usize)>>,
    /// Maps slot index → full NodeId (for topo sort without generation scanning).
    index_to_id: HashMap<u32, NodeId>,
}

impl DspGraph {
    /// Creates a new empty DSP graph.
    ///
    /// Initializes the arena, buffer pool, and execution structures with
    /// pre-allocated capacity for `MAX_NODES` nodes.
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::graph::DspGraph;
    ///
    /// let graph = DspGraph::new();
    /// assert_eq!(graph.execution_order.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            arena: Arena::with_capacity(MAX_NODES),
            buffers: BufferPool::default(),
            execution_order: Vec::with_capacity(MAX_NODES),
            levels: Vec::with_capacity(MAX_NODES),
            output_node: None,
            forward_edges: HashMap::new(),
            index_to_id: HashMap::new(),
        }
    }

    /// Adds a node to the graph and returns its ID.
    ///
    /// Acquires a buffer from the pool, inserts the node into the arena,
    /// and rebuilds the topological execution order.
    ///
    /// # Arguments
    ///
    /// * `processor` - Boxed DSP node implementation
    ///
    /// # Returns
    ///
    /// * `Some(NodeId)` - The node's unique identifier
    /// * `None` - If arena is full or buffer pool exhausted
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::graph::DspGraph;
    /// use aether_core::node::DspNode;
    /// use aether_core::param::ParamBlock;
    /// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
    ///
    /// struct Oscillator { frequency: f32, phase: f32 }
    /// impl DspNode for Oscillator {
    ///     fn process(&mut self, _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
    ///                output: &mut [f32; BUFFER_SIZE], _params: &mut ParamBlock, sr: f32) {
    ///         let phase_inc = self.frequency / sr;
    ///         for sample in output.iter_mut() {
    ///             *sample = (self.phase * std::f32::consts::TAU).sin();
    ///             self.phase = (self.phase + phase_inc).fract();
    ///         }
    ///     }
    ///     fn type_name(&self) -> &'static str { "Oscillator" }
    /// }
    ///
    /// let mut graph = DspGraph::new();
    /// let osc = Box::new(Oscillator { frequency: 440.0, phase: 0.0 });
    /// let id = graph.add_node(osc).unwrap();
    /// ```
    ///
    /// # See Also
    ///
    /// * [`remove_node`](Self::remove_node) - Remove a node from the graph
    /// * [`connect`](Self::connect) - Connect two nodes
    pub fn add_node(&mut self, processor: Box<dyn DspNode>) -> Option<NodeId> {
        let buf = self.buffers.acquire()?;
        let record = NodeRecord::new(processor, buf);
        let id = self.arena.insert(record)?;
        self.forward_edges.insert(id.index, Vec::new());
        self.index_to_id.insert(id.index, id);
        self.rebuild_execution_order();
        Some(id)
    }

    /// Removes a node from the graph and releases its buffer.
    ///
    /// Removes the node from the arena, releases its output buffer back to
    /// the pool, and removes all edges connected to this node. Rebuilds the
    /// topological execution order.
    ///
    /// # Arguments
    ///
    /// * `id` - Node ID to remove
    ///
    /// # Returns
    ///
    /// * `true` - Node removed successfully
    /// * `false` - Node doesn't exist (invalid ID or already removed)
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::graph::DspGraph;
    /// use aether_core::node::DspNode;
    /// use aether_core::param::ParamBlock;
    /// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
    ///
    /// struct SimpleNode;
    /// impl DspNode for SimpleNode {
    ///     fn process(&mut self, _: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
    ///                output: &mut [f32; BUFFER_SIZE], _: &mut ParamBlock, _: f32) {
    ///         output.fill(0.0);
    ///     }
    ///     fn type_name(&self) -> &'static str { "SimpleNode" }
    /// }
    ///
    /// let mut graph = DspGraph::new();
    /// let node_id = graph.add_node(Box::new(SimpleNode)).unwrap();
    ///
    /// assert!(graph.remove_node(node_id)); // Returns true
    /// assert!(!graph.remove_node(node_id)); // Returns false (already removed)
    /// ```
    ///
    /// # See Also
    ///
    /// * [`add_node`](Self::add_node) - Add a node to the graph
    pub fn remove_node(&mut self, id: NodeId) -> bool {
        if let Some(record) = self.arena.remove(id) {
            self.buffers.release(record.output_buffer);
            self.forward_edges.remove(&id.index);
            self.index_to_id.remove(&id.index);
            for edges in self.forward_edges.values_mut() {
                edges.retain(|(dst, _)| dst.index != id.index);
            }
            self.rebuild_execution_order();
            true
        } else {
            false
        }
    }

    /// Connects the output of one node to the input of another.
    ///
    /// Creates an edge in the DAG from `src` to `dst`, routing audio from
    /// the source node's output buffer to the destination node's input slot.
    /// Rebuilds the topological execution order to maintain DAG invariants.
    ///
    /// # Arguments
    ///
    /// * `src` - Source node ID (output)
    /// * `dst` - Destination node ID (input)
    /// * `slot` - Input slot index on destination node (0 to MAX_INPUTS-1)
    ///
    /// # Returns
    ///
    /// * `true` - Connection successful
    /// * `false` - One or both nodes don't exist
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::graph::DspGraph;
    /// use aether_core::node::DspNode;
    /// use aether_core::param::ParamBlock;
    /// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
    ///
    /// struct SimpleNode;
    /// impl DspNode for SimpleNode {
    ///     fn process(&mut self, _: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
    ///                output: &mut [f32; BUFFER_SIZE], _: &mut ParamBlock, _: f32) {
    ///         output.fill(0.0);
    ///     }
    ///     fn type_name(&self) -> &'static str { "SimpleNode" }
    /// }
    ///
    /// let mut graph = DspGraph::new();
    /// let node_a = graph.add_node(Box::new(SimpleNode)).unwrap();
    /// let node_b = graph.add_node(Box::new(SimpleNode)).unwrap();
    ///
    /// // Connect node_a output → node_b input slot 0
    /// graph.connect(node_a, node_b, 0);
    /// ```
    ///
    /// # See Also
    ///
    /// * [`disconnect`](Self::disconnect) - Remove a connection
    /// * [`add_node`](Self::add_node) - Add nodes to connect
    pub fn connect(&mut self, src: NodeId, dst: NodeId, slot: usize) -> bool {
        if self.arena.get(src).is_none() || self.arena.get(dst).is_none() {
            return false;
        }
        // Record forward edge for topo sort.
        if let Some(edges) = self.forward_edges.get_mut(&src.index) {
            edges.push((dst, slot));
        }
        // Record backward reference in dst node.
        if let Some(record) = self.arena.get_mut(dst) {
            record.inputs[slot] = Some(src);
        }
        self.rebuild_execution_order();
        true
    }

    /// Disconnects an input slot on a destination node.
    ///
    /// Removes the connection to the specified input slot, clearing the
    /// audio routing. The slot will receive silence until reconnected.
    /// Rebuilds the topological execution order.
    ///
    /// # Arguments
    ///
    /// * `dst` - Destination node ID
    /// * `slot` - Input slot index to disconnect (0 to MAX_INPUTS-1)
    ///
    /// # Returns
    ///
    /// * `true` - Disconnection successful
    /// * `false` - Node doesn't exist or slot was already empty
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::graph::DspGraph;
    /// use aether_core::node::DspNode;
    /// use aether_core::param::ParamBlock;
    /// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
    ///
    /// struct SimpleNode;
    /// impl DspNode for SimpleNode {
    ///     fn process(&mut self, _: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
    ///                output: &mut [f32; BUFFER_SIZE], _: &mut ParamBlock, _: f32) {
    ///         output.fill(0.0);
    ///     }
    ///     fn type_name(&self) -> &'static str { "SimpleNode" }
    /// }
    ///
    /// let mut graph = DspGraph::new();
    /// let node_a = graph.add_node(Box::new(SimpleNode)).unwrap();
    /// let node_b = graph.add_node(Box::new(SimpleNode)).unwrap();
    ///
    /// graph.connect(node_a, node_b, 0);
    /// graph.disconnect(node_b, 0); // Disconnect slot 0
    /// ```
    ///
    /// # See Also
    ///
    /// * [`connect`](Self::connect) - Create a connection
    pub fn disconnect(&mut self, dst: NodeId, slot: usize) -> bool {
        let src_id = self.arena.get(dst).and_then(|r| r.inputs[slot]);
        if let Some(src) = src_id {
            if let Some(edges) = self.forward_edges.get_mut(&src.index) {
                edges.retain(|(d, s)| !(d.index == dst.index && *s == slot));
            }
        }
        if let Some(record) = self.arena.get_mut(dst) {
            record.inputs[slot] = None;
            self.rebuild_execution_order();
            true
        } else {
            false
        }
    }

    /// Kahn's algorithm topological sort. O(V+E), bounded by MAX_NODES.
    fn rebuild_execution_order(&mut self) {
        self.execution_order.clear();
        self.levels.clear();

        // Compute in-degrees from forward edges.
        let mut in_degree: HashMap<u32, usize> = self.index_to_id.keys().map(|&k| (k, 0)).collect();
        for edges in self.forward_edges.values() {
            for (dst, _) in edges {
                *in_degree.entry(dst.index).or_insert(0) += 1;
            }
        }

        // Seed the first wave: all nodes with in-degree 0.
        let mut current_wave: Vec<u32> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&idx, _)| idx)
            .collect();

        while !current_wave.is_empty() {
            let mut level_ids: Vec<NodeId> = Vec::with_capacity(current_wave.len());
            let mut next_wave: Vec<u32> = Vec::new();

            for idx in &current_wave {
                if let Some(&id) = self.index_to_id.get(idx) {
                    level_ids.push(id);
                    self.execution_order.push(id);
                }
                if let Some(edges) = self.forward_edges.get(idx) {
                    for (dst, _) in edges.clone() {
                        let deg = in_degree.entry(dst.index).or_insert(0);
                        if *deg > 0 {
                            *deg -= 1;
                            if *deg == 0 {
                                next_wave.push(dst.index);
                            }
                        }
                    }
                }
            }

            self.levels.push(level_ids);
            current_wave = next_wave;
        }
    }

    /// Sets the output node whose buffer is sent to the DAC.
    ///
    /// Designates which node's output buffer should be copied to the
    /// audio device output. Only one node can be the output node at a time.
    ///
    /// # Arguments
    ///
    /// * `id` - Node ID to use as output
    ///
    /// # Example
    ///
    /// ```
    /// use aether_core::graph::DspGraph;
    /// use aether_core::node::DspNode;
    /// use aether_core::param::ParamBlock;
    /// use aether_core::{BUFFER_SIZE, MAX_INPUTS};
    ///
    /// struct Oscillator { phase: f32 }
    /// impl DspNode for Oscillator {
    ///     fn process(&mut self, _: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
    ///                output: &mut [f32; BUFFER_SIZE], _: &mut ParamBlock, _: f32) {
    ///         for sample in output.iter_mut() {
    ///             *sample = (self.phase * std::f32::consts::TAU).sin();
    ///             self.phase = (self.phase + 0.01).fract();
    ///         }
    ///     }
    ///     fn type_name(&self) -> &'static str { "Oscillator" }
    /// }
    ///
    /// let mut graph = DspGraph::new();
    /// let osc_id = graph.add_node(Box::new(Oscillator { phase: 0.0 })).unwrap();
    /// graph.set_output_node(osc_id);
    /// ```
    pub fn set_output_node(&mut self, id: NodeId) {
        self.output_node = Some(id);
    }
}

impl Default for DspGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{node::DspNode, param::ParamBlock, BUFFER_SIZE, MAX_INPUTS};
    use proptest::prelude::*;

    /// Minimal test node for graph topology testing.
    struct TestNode;

    impl DspNode for TestNode {
        fn process(
            &mut self,
            _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
            output: &mut [f32; BUFFER_SIZE],
            _params: &mut ParamBlock,
            _sample_rate: f32,
        ) {
            output.fill(0.0);
        }

        fn type_name(&self) -> &'static str {
            "TestNode"
        }
    }

    // Property 2
    proptest! {
        /// **Validates: Requirements 1.2, 1.9**
        ///
        /// Property 2: Topological level assignments satisfy the dependency ordering invariant.
        ///
        /// For any DAG after `rebuild_execution_order`, every node at level L SHALL have all
        /// its input-connected nodes at levels strictly less than L. Equivalently, no node at
        /// level L depends on any other node at level L.
        #[test]
        fn prop_topological_level_ordering_invariant(
            num_nodes in 1usize..=20,
            edges in prop::collection::vec((0usize..20, 0usize..20, 0usize..MAX_INPUTS), 0..50)
        ) {
            let mut graph = DspGraph::new();
            let mut node_ids = Vec::new();

            // Add nodes
            for _ in 0..num_nodes {
                if let Some(id) = graph.add_node(Box::new(TestNode)) {
                    node_ids.push(id);
                }
            }

            // Add edges, filtering to maintain DAG invariant (src < dst to prevent cycles)
            for &(src_idx, dst_idx, slot) in &edges {
                if src_idx < num_nodes && dst_idx < num_nodes && src_idx < dst_idx {
                    let src = node_ids[src_idx];
                    let dst = node_ids[dst_idx];
                    graph.connect(src, dst, slot);
                }
            }

            // Build a map from NodeId to level index
            let mut node_to_level: HashMap<u32, usize> = HashMap::new();
            for (level_idx, level_nodes) in graph.levels.iter().enumerate() {
                for &node_id in level_nodes {
                    node_to_level.insert(node_id.index, level_idx);
                }
            }

            // Verify the invariant: for every edge (src → dst), level[src] < level[dst]
            for &(src_idx, dst_idx, slot) in &edges {
                if src_idx < num_nodes && dst_idx < num_nodes && src_idx < dst_idx {
                    let src = node_ids[src_idx];
                    let dst = node_ids[dst_idx];

                    // Check if the edge was actually added (connect may fail if slot already occupied)
                    if let Some(record) = graph.arena.get(dst) {
                        if record.inputs[slot] == Some(src) {
                            // Edge exists, verify level ordering
                            let src_level = node_to_level.get(&src.index).copied();
                            let dst_level = node_to_level.get(&dst.index).copied();

                            if let (Some(src_lvl), Some(dst_lvl)) = (src_level, dst_level) {
                                prop_assert!(
                                    src_lvl < dst_lvl,
                                    "Level ordering violated: node {} at level {} → node {} at level {}",
                                    src.index, src_lvl, dst.index, dst_lvl
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
