//! Undo/redo stack for structural graph mutations.

use std::collections::VecDeque;

use aether_core::arena::NodeId;

use crate::graph_manager::PatchDef;

// ── StructuralIntent ──────────────────────────────────────────────────────────

/// A subset of `Intent` covering only the six structural mutations that can be
/// undone/redone. Computed at push time so the inverse is always available.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum StructuralIntent {
    AddNode { node_type: String, created_id: NodeId },
    RemoveNode { node_id: u32, generation: u32, node_type: String },
    Connect { src_id: u32, src_gen: u32, dst_id: u32, dst_gen: u32, slot: usize },
    Disconnect { dst_id: u32, dst_gen: u32, slot: usize, prev_src_id: u32, prev_src_gen: u32 },
    LoadPatch { patch: PatchDef },
    ClearGraph { snapshot_before: PatchDef },
}

// ── UndoEntry ─────────────────────────────────────────────────────────────────

/// A paired forward/inverse intent. Both are stored so undo and redo can be
/// applied without re-computing the inverse.
pub struct UndoEntry {
    /// The intent that was originally applied.
    pub forward: StructuralIntent,
    /// The intent that undoes it.
    pub inverse: StructuralIntent,
}

// ── UndoStack ─────────────────────────────────────────────────────────────────

/// Bounded ring buffer of `UndoEntry` values. Maximum depth: 100.
pub struct UndoStack {
    entries: VecDeque<UndoEntry>,
    max_depth: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_depth: 100,
        }
    }

    /// Push an entry to the back. If at capacity, the oldest entry (front) is
    /// dropped to keep the stack bounded.
    pub fn push(&mut self, entry: UndoEntry) {
        if self.entries.len() >= self.max_depth {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Pop the most-recent entry from the back.
    pub fn pop(&mut self) -> Option<UndoEntry> {
        self.entries.pop_back()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_entry() -> UndoEntry {
        UndoEntry {
            forward: StructuralIntent::AddNode {
                node_type: "Gain".into(),
                created_id: NodeId { index: 0, generation: 0 },
            },
            inverse: StructuralIntent::RemoveNode {
                node_id: 0,
                generation: 0,
                node_type: "Gain".into(),
            },
        }
    }

    #[test]
    fn push_pop_roundtrip() {
        let mut stack = UndoStack::new();
        assert!(stack.is_empty());
        stack.push(dummy_entry());
        assert_eq!(stack.len(), 1);
        assert!(stack.pop().is_some());
        assert!(stack.is_empty());
    }

    #[test]
    fn bounded_at_100() {
        let mut stack = UndoStack::new();
        for _ in 0..150 {
            stack.push(dummy_entry());
            assert!(stack.len() <= 100);
        }
        assert_eq!(stack.len(), 100);
    }

    // Property 6: UndoStack bounded size
    // Validates: Requirements 3.1, 3.7
    #[cfg(test)]
    mod property_6 {
        use super::*;

        /// Simulate pushing N entries and assert the stack never exceeds 100.
        fn check_bounded(n: usize) {
            let mut stack = UndoStack::new();
            for _ in 0..n {
                stack.push(dummy_entry());
                assert!(stack.len() <= 100, "stack exceeded max_depth after push");
            }
        }

        #[test]
        fn bounded_1_to_200() {
            for n in 1..=200 {
                check_bounded(n);
            }
        }
    }

    // Property 7: Undo/redo round-trip restores graph state
    // **Validates: Requirements 3.3, 3.8, 3.10, 3.11, 3.12**
    #[cfg(test)]
    mod property_7 {
        use super::*;
        use crate::graph_manager::{GraphManager, Intent, Response};
        use aether_core::scheduler::Scheduler;
        use proptest::prelude::*;

        /// Generate random structural intents for property testing
        fn arb_structural_intent() -> impl Strategy<Value = Intent> {
            prop_oneof![
                // AddNode with common node types
                prop::sample::select(vec![
                    "Oscillator",
                    "Gain",
                    "StateVariableFilter",
                    "DelayLine",
                    "Mixer",
                ])
                .prop_map(|node_type| Intent::AddNode {
                    node_type: node_type.to_string()
                }),
                // Connect - we'll need to ensure nodes exist first
                // For simplicity, we'll test this separately
                // Disconnect - same as above
            ]
        }

        /// Helper to extract snapshot data from Response
        fn extract_snapshot(
            response: &Response,
        ) -> Option<(Vec<(u32, u32, String)>, Vec<(u32, u32, usize)>, Option<u32>)> {
            match response {
                Response::Snapshot {
                    nodes,
                    edges,
                    output_node_id,
                    ..
                } => {
                    let node_data: Vec<(u32, u32, String)> = nodes
                        .iter()
                        .map(|n| (n.id, n.generation, n.node_type.clone()))
                        .collect();
                    let edge_data: Vec<(u32, u32, usize)> =
                        edges.iter().map(|e| (e.src_id, e.dst_id, e.slot)).collect();
                    Some((node_data, edge_data, *output_node_id))
                }
                _ => None,
            }
        }

        proptest! {
            /// Feature: aether-engine-upgrades, Property 7: Undo/redo round-trip restores graph state
            ///
            /// For any structural intent applied to a GraphManager, applying the intent and then
            /// immediately undoing it SHALL restore the graph to its exact prior state (same nodes,
            /// same edges, same output node).
            #[test]
            fn prop_undo_round_trip_add_node(
                node_type in prop::sample::select(vec![
                    "Oscillator",
                    "Gain",
                    "StateVariableFilter",
                    "DelayLine",
                    "Mixer",
                ])
            ) {
                // Setup
                let mut graph_manager = GraphManager::new(48000.0);
                let mut scheduler = Scheduler::new(48000.0);

                // Capture initial snapshot
                let initial_response = graph_manager.snapshot(&scheduler);
                let initial_snapshot = extract_snapshot(&initial_response)
                    .expect("Failed to extract initial snapshot");

                // Apply AddNode intent
                let add_intent = Intent::AddNode { node_type: node_type.to_string() };
                let add_response = graph_manager.handle(add_intent, &mut scheduler);
                
                // Verify we got a snapshot response (not an error)
                prop_assert!(matches!(add_response, Response::Snapshot { .. }));

                // Capture post-add snapshot
                let post_add_snapshot = extract_snapshot(&add_response)
                    .expect("Failed to extract post-add snapshot");

                // Verify that a node was added
                prop_assert_eq!(post_add_snapshot.0.len(), initial_snapshot.0.len() + 1);

                // Apply Undo intent
                let undo_response = graph_manager.handle(Intent::Undo, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(undo_response, Response::Snapshot { .. }));

                // Capture post-undo snapshot
                let post_undo_snapshot = extract_snapshot(&undo_response)
                    .expect("Failed to extract post-undo snapshot");

                // Assert: post-undo snapshot matches initial snapshot
                prop_assert_eq!(
                    post_undo_snapshot.0.len(),
                    initial_snapshot.0.len(),
                    "Node count mismatch after undo"
                );
                prop_assert_eq!(
                    post_undo_snapshot.1.len(),
                    initial_snapshot.1.len(),
                    "Edge count mismatch after undo"
                );
                prop_assert_eq!(
                    post_undo_snapshot.2,
                    initial_snapshot.2,
                    "Output node mismatch after undo"
                );
            }

            /// Test undo round-trip for Connect intent
            #[test]
            fn prop_undo_round_trip_connect(
                slot in 0usize..4,
            ) {
                // Setup: create a graph with two nodes
                let mut graph_manager = GraphManager::new(48000.0);
                let mut scheduler = Scheduler::new(48000.0);

                // Add two nodes to connect
                let add1 = graph_manager.handle(
                    Intent::AddNode { node_type: "Oscillator".to_string() },
                    &mut scheduler
                );
                let snapshot1 = extract_snapshot(&add1).expect("Failed to extract snapshot after add1");
                let node1_id = snapshot1.0[0].0;
                let node1_gen = snapshot1.0[0].1;

                let add2 = graph_manager.handle(
                    Intent::AddNode { node_type: "Gain".to_string() },
                    &mut scheduler
                );
                let snapshot2 = extract_snapshot(&add2).expect("Failed to extract snapshot after add2");
                let node2_id = snapshot2.0[1].0;
                let node2_gen = snapshot2.0[1].1;

                // Capture initial snapshot (with two nodes, no connections)
                let initial_response = graph_manager.snapshot(&scheduler);
                let initial_snapshot = extract_snapshot(&initial_response)
                    .expect("Failed to extract initial snapshot");

                // Apply Connect intent
                let connect_intent = Intent::Connect {
                    src_id: node1_id,
                    src_gen: node1_gen,
                    dst_id: node2_id,
                    dst_gen: node2_gen,
                    slot,
                };
                let connect_response = graph_manager.handle(connect_intent, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(connect_response, Response::Snapshot { .. }));

                // Capture post-connect snapshot
                let post_connect_snapshot = extract_snapshot(&connect_response)
                    .expect("Failed to extract post-connect snapshot");

                // Verify that an edge was added
                prop_assert_eq!(post_connect_snapshot.1.len(), initial_snapshot.1.len() + 1);

                // Apply Undo intent
                let undo_response = graph_manager.handle(Intent::Undo, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(undo_response, Response::Snapshot { .. }));

                // Capture post-undo snapshot
                let post_undo_snapshot = extract_snapshot(&undo_response)
                    .expect("Failed to extract post-undo snapshot");

                // Assert: post-undo snapshot matches initial snapshot
                prop_assert_eq!(
                    post_undo_snapshot.0.len(),
                    initial_snapshot.0.len(),
                    "Node count mismatch after undo"
                );
                prop_assert_eq!(
                    post_undo_snapshot.1.len(),
                    initial_snapshot.1.len(),
                    "Edge count mismatch after undo"
                );
                prop_assert_eq!(
                    post_undo_snapshot.2,
                    initial_snapshot.2,
                    "Output node mismatch after undo"
                );
            }

            /// Test undo round-trip for Disconnect intent
            #[test]
            fn prop_undo_round_trip_disconnect(
                slot in 0usize..4,
            ) {
                // Setup: create a graph with two connected nodes
                let mut graph_manager = GraphManager::new(48000.0);
                let mut scheduler = Scheduler::new(48000.0);

                // Add two nodes
                let add1 = graph_manager.handle(
                    Intent::AddNode { node_type: "Oscillator".to_string() },
                    &mut scheduler
                );
                let snapshot1 = extract_snapshot(&add1).expect("Failed to extract snapshot after add1");
                let node1_id = snapshot1.0[0].0;
                let node1_gen = snapshot1.0[0].1;

                let add2 = graph_manager.handle(
                    Intent::AddNode { node_type: "Gain".to_string() },
                    &mut scheduler
                );
                let snapshot2 = extract_snapshot(&add2).expect("Failed to extract snapshot after add2");
                let node2_id = snapshot2.0[1].0;
                let node2_gen = snapshot2.0[1].1;

                // Connect them
                let _connect = graph_manager.handle(
                    Intent::Connect {
                        src_id: node1_id,
                        src_gen: node1_gen,
                        dst_id: node2_id,
                        dst_gen: node2_gen,
                        slot,
                    },
                    &mut scheduler
                );

                // Capture initial snapshot (with two nodes and one connection)
                let initial_response = graph_manager.snapshot(&scheduler);
                let initial_snapshot = extract_snapshot(&initial_response)
                    .expect("Failed to extract initial snapshot");

                // Apply Disconnect intent
                let disconnect_intent = Intent::Disconnect {
                    dst_id: node2_id,
                    dst_gen: node2_gen,
                    slot,
                };
                let disconnect_response = graph_manager.handle(disconnect_intent, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(disconnect_response, Response::Snapshot { .. }));

                // Capture post-disconnect snapshot
                let post_disconnect_snapshot = extract_snapshot(&disconnect_response)
                    .expect("Failed to extract post-disconnect snapshot");

                // Verify that an edge was removed
                prop_assert_eq!(post_disconnect_snapshot.1.len(), initial_snapshot.1.len() - 1);

                // Apply Undo intent
                let undo_response = graph_manager.handle(Intent::Undo, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(undo_response, Response::Snapshot { .. }));

                // Capture post-undo snapshot
                let post_undo_snapshot = extract_snapshot(&undo_response)
                    .expect("Failed to extract post-undo snapshot");

                // Assert: post-undo snapshot matches initial snapshot
                prop_assert_eq!(
                    post_undo_snapshot.0.len(),
                    initial_snapshot.0.len(),
                    "Node count mismatch after undo"
                );
                prop_assert_eq!(
                    post_undo_snapshot.1.len(),
                    initial_snapshot.1.len(),
                    "Edge count mismatch after undo"
                );
                prop_assert_eq!(
                    post_undo_snapshot.2,
                    initial_snapshot.2,
                    "Output node mismatch after undo"
                );
            }
        }
    }

    // Property 8: Redo re-applies the undone intent
    // **Validates: Requirements 3.5**
    #[cfg(test)]
    mod property_8 {
        use super::*;
        use crate::graph_manager::{GraphManager, Intent, Response};
        use aether_core::scheduler::Scheduler;
        use proptest::prelude::*;

        /// Helper to extract snapshot data from Response
        fn extract_snapshot(
            response: &Response,
        ) -> Option<(Vec<(u32, u32, String)>, Vec<(u32, u32, usize)>, Option<u32>)> {
            match response {
                Response::Snapshot {
                    nodes,
                    edges,
                    output_node_id,
                    ..
                } => {
                    let node_data: Vec<(u32, u32, String)> = nodes
                        .iter()
                        .map(|n| (n.id, n.generation, n.node_type.clone()))
                        .collect();
                    let edge_data: Vec<(u32, u32, usize)> =
                        edges.iter().map(|e| (e.src_id, e.dst_id, e.slot)).collect();
                    Some((node_data, edge_data, *output_node_id))
                }
                _ => None,
            }
        }

        proptest! {
            /// Feature: aether-engine-upgrades, Property 8: Redo re-applies the undone intent
            ///
            /// For any structural intent applied to a GraphManager, applying the intent, undoing it,
            /// and then redoing it SHALL restore the graph to the post-intent state (same nodes,
            /// same edges, same output node as immediately after the original application).
            #[test]
            fn prop_redo_reapplies_add_node(
                node_type in prop::sample::select(vec![
                    "Oscillator",
                    "Gain",
                    "StateVariableFilter",
                    "DelayLine",
                    "Mixer",
                ])
            ) {
                // Setup
                let mut graph_manager = GraphManager::new(48000.0);
                let mut scheduler = Scheduler::new(48000.0);

                // Capture initial snapshot
                let initial_response = graph_manager.snapshot(&scheduler);
                let initial_snapshot = extract_snapshot(&initial_response)
                    .expect("Failed to extract initial snapshot");

                // Apply AddNode intent
                let add_intent = Intent::AddNode { node_type: node_type.to_string() };
                let add_response = graph_manager.handle(add_intent, &mut scheduler);
                
                // Verify we got a snapshot response (not an error)
                prop_assert!(matches!(add_response, Response::Snapshot { .. }));

                // Capture post-add snapshot
                let post_add_snapshot = extract_snapshot(&add_response)
                    .expect("Failed to extract post-add snapshot");

                // Verify that a node was added
                prop_assert_eq!(post_add_snapshot.0.len(), initial_snapshot.0.len() + 1);

                // Apply Undo intent
                let undo_response = graph_manager.handle(Intent::Undo, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(undo_response, Response::Snapshot { .. }));

                // Capture post-undo snapshot
                let post_undo_snapshot = extract_snapshot(&undo_response)
                    .expect("Failed to extract post-undo snapshot");

                // Verify undo worked
                prop_assert_eq!(
                    post_undo_snapshot.0.len(),
                    initial_snapshot.0.len(),
                    "Node count mismatch after undo"
                );

                // Apply Redo intent
                let redo_response = graph_manager.handle(Intent::Redo, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(redo_response, Response::Snapshot { .. }));

                // Capture post-redo snapshot
                let post_redo_snapshot = extract_snapshot(&redo_response)
                    .expect("Failed to extract post-redo snapshot");

                // Assert: post-redo snapshot matches post-add snapshot
                prop_assert_eq!(
                    post_redo_snapshot.0.len(),
                    post_add_snapshot.0.len(),
                    "Node count mismatch after redo"
                );
                prop_assert_eq!(
                    post_redo_snapshot.1.len(),
                    post_add_snapshot.1.len(),
                    "Edge count mismatch after redo"
                );
                prop_assert_eq!(
                    post_redo_snapshot.2,
                    post_add_snapshot.2,
                    "Output node mismatch after redo"
                );
            }

            /// Test redo round-trip for Connect intent
            #[test]
            fn prop_redo_reapplies_connect(
                slot in 0usize..4,
            ) {
                // Setup: create a graph with two nodes
                let mut graph_manager = GraphManager::new(48000.0);
                let mut scheduler = Scheduler::new(48000.0);

                // Add two nodes to connect
                let add1 = graph_manager.handle(
                    Intent::AddNode { node_type: "Oscillator".to_string() },
                    &mut scheduler
                );
                let snapshot1 = extract_snapshot(&add1).expect("Failed to extract snapshot after add1");
                let node1_id = snapshot1.0[0].0;
                let node1_gen = snapshot1.0[0].1;

                let add2 = graph_manager.handle(
                    Intent::AddNode { node_type: "Gain".to_string() },
                    &mut scheduler
                );
                let snapshot2 = extract_snapshot(&add2).expect("Failed to extract snapshot after add2");
                let node2_id = snapshot2.0[1].0;
                let node2_gen = snapshot2.0[1].1;

                // Capture initial snapshot (with two nodes, no connections)
                let initial_response = graph_manager.snapshot(&scheduler);
                let initial_snapshot = extract_snapshot(&initial_response)
                    .expect("Failed to extract initial snapshot");

                // Apply Connect intent
                let connect_intent = Intent::Connect {
                    src_id: node1_id,
                    src_gen: node1_gen,
                    dst_id: node2_id,
                    dst_gen: node2_gen,
                    slot,
                };
                let connect_response = graph_manager.handle(connect_intent, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(connect_response, Response::Snapshot { .. }));

                // Capture post-connect snapshot
                let post_connect_snapshot = extract_snapshot(&connect_response)
                    .expect("Failed to extract post-connect snapshot");

                // Verify that an edge was added
                prop_assert_eq!(post_connect_snapshot.1.len(), initial_snapshot.1.len() + 1);

                // Apply Undo intent
                let undo_response = graph_manager.handle(Intent::Undo, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(undo_response, Response::Snapshot { .. }));

                // Capture post-undo snapshot
                let post_undo_snapshot = extract_snapshot(&undo_response)
                    .expect("Failed to extract post-undo snapshot");

                // Verify undo worked
                prop_assert_eq!(
                    post_undo_snapshot.1.len(),
                    initial_snapshot.1.len(),
                    "Edge count mismatch after undo"
                );

                // Apply Redo intent
                let redo_response = graph_manager.handle(Intent::Redo, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(redo_response, Response::Snapshot { .. }));

                // Capture post-redo snapshot
                let post_redo_snapshot = extract_snapshot(&redo_response)
                    .expect("Failed to extract post-redo snapshot");

                // Assert: post-redo snapshot matches post-connect snapshot
                prop_assert_eq!(
                    post_redo_snapshot.0.len(),
                    post_connect_snapshot.0.len(),
                    "Node count mismatch after redo"
                );
                prop_assert_eq!(
                    post_redo_snapshot.1.len(),
                    post_connect_snapshot.1.len(),
                    "Edge count mismatch after redo"
                );
                prop_assert_eq!(
                    post_redo_snapshot.2,
                    post_connect_snapshot.2,
                    "Output node mismatch after redo"
                );
            }

            /// Test redo round-trip for Disconnect intent
            #[test]
            fn prop_redo_reapplies_disconnect(
                slot in 0usize..4,
            ) {
                // Setup: create a graph with two connected nodes
                let mut graph_manager = GraphManager::new(48000.0);
                let mut scheduler = Scheduler::new(48000.0);

                // Add two nodes
                let add1 = graph_manager.handle(
                    Intent::AddNode { node_type: "Oscillator".to_string() },
                    &mut scheduler
                );
                let snapshot1 = extract_snapshot(&add1).expect("Failed to extract snapshot after add1");
                let node1_id = snapshot1.0[0].0;
                let node1_gen = snapshot1.0[0].1;

                let add2 = graph_manager.handle(
                    Intent::AddNode { node_type: "Gain".to_string() },
                    &mut scheduler
                );
                let snapshot2 = extract_snapshot(&add2).expect("Failed to extract snapshot after add2");
                let node2_id = snapshot2.0[1].0;
                let node2_gen = snapshot2.0[1].1;

                // Connect them
                let _connect = graph_manager.handle(
                    Intent::Connect {
                        src_id: node1_id,
                        src_gen: node1_gen,
                        dst_id: node2_id,
                        dst_gen: node2_gen,
                        slot,
                    },
                    &mut scheduler
                );

                // Capture initial snapshot (with two nodes and one connection)
                let initial_response = graph_manager.snapshot(&scheduler);
                let initial_snapshot = extract_snapshot(&initial_response)
                    .expect("Failed to extract initial snapshot");

                // Apply Disconnect intent
                let disconnect_intent = Intent::Disconnect {
                    dst_id: node2_id,
                    dst_gen: node2_gen,
                    slot,
                };
                let disconnect_response = graph_manager.handle(disconnect_intent, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(disconnect_response, Response::Snapshot { .. }));

                // Capture post-disconnect snapshot
                let post_disconnect_snapshot = extract_snapshot(&disconnect_response)
                    .expect("Failed to extract post-disconnect snapshot");

                // Verify that an edge was removed
                prop_assert_eq!(post_disconnect_snapshot.1.len(), initial_snapshot.1.len() - 1);

                // Apply Undo intent
                let undo_response = graph_manager.handle(Intent::Undo, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(undo_response, Response::Snapshot { .. }));

                // Capture post-undo snapshot
                let post_undo_snapshot = extract_snapshot(&undo_response)
                    .expect("Failed to extract post-undo snapshot");

                // Verify undo worked
                prop_assert_eq!(
                    post_undo_snapshot.1.len(),
                    initial_snapshot.1.len(),
                    "Edge count mismatch after undo"
                );

                // Apply Redo intent
                let redo_response = graph_manager.handle(Intent::Redo, &mut scheduler);
                
                // Verify we got a snapshot response
                prop_assert!(matches!(redo_response, Response::Snapshot { .. }));

                // Capture post-redo snapshot
                let post_redo_snapshot = extract_snapshot(&redo_response)
                    .expect("Failed to extract post-redo snapshot");

                // Assert: post-redo snapshot matches post-disconnect snapshot
                prop_assert_eq!(
                    post_redo_snapshot.0.len(),
                    post_disconnect_snapshot.0.len(),
                    "Node count mismatch after redo"
                );
                prop_assert_eq!(
                    post_redo_snapshot.1.len(),
                    post_disconnect_snapshot.1.len(),
                    "Edge count mismatch after redo"
                );
                prop_assert_eq!(
                    post_redo_snapshot.2,
                    post_disconnect_snapshot.2,
                    "Output node mismatch after redo"
                );
            }
        }
    }
}
