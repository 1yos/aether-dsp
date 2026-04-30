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

/// The DSP graph. Lives on the RT thread after initial construction.
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

    /// Add a node to the graph. Returns its NodeId.
    pub fn add_node(&mut self, processor: Box<dyn DspNode>) -> Option<NodeId> {
        let buf = self.buffers.acquire()?;
        let record = NodeRecord::new(processor, buf);
        let id = self.arena.insert(record)?;
        self.forward_edges.insert(id.index, Vec::new());
        self.index_to_id.insert(id.index, id);
        self.rebuild_execution_order();
        Some(id)
    }

    /// Remove a node, releasing its buffer.
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

    /// Connect src output → dst input[slot].
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

    /// Disconnect dst input[slot].
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
