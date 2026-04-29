import { create } from "zustand";
import {
  Node,
  Edge,
  addEdge,
  applyNodeChanges,
  applyEdgeChanges,
  Connection,
  NodeChange,
  EdgeChange,
} from "reactflow";
import { GraphSnapshot, NODE_PARAM_DEFS } from "../types/graph";

export type WsStatus = "disconnected" | "connecting" | "connected" | "error";

interface GraphStore {
  nodes: Node[];
  edges: Edge[];
  selectedNode: string | null;
  wsStatus: WsStatus;
  audioActive: boolean; // pulses true when output node is processing
  outputNodeId: string | null; // which node is the audio output
  onNodesChange: (changes: NodeChange[]) => void;
  onEdgesChange: (changes: EdgeChange[]) => void;
  onConnect: (connection: Connection) => void;
  setSelectedNode: (id: string | null) => void;
  deleteSelectedNode: () => void;
  addDspNode: (nodeType: string) => void;
  updateParam: (nodeId: string, paramIndex: number, value: number) => void;
  loadSnapshot: (snapshot: GraphSnapshot) => void;
  setWsStatus: (status: WsStatus) => void;
  setAudioActive: (active: boolean) => void;
  setOutputNode: (id: string | null) => void;
}

let nodeCounter = 0;

export const useGraphStore = create<GraphStore>((set, get) => ({
  nodes: [],
  edges: [],
  selectedNode: null,
  wsStatus: "disconnected",
  audioActive: false,
  outputNodeId: null,

  onNodesChange: (changes) =>
    set((s) => ({ nodes: applyNodeChanges(changes, s.nodes) })),

  onEdgesChange: (changes) =>
    set((s) => ({ edges: applyEdgeChanges(changes, s.edges) })),

  onConnect: (connection) =>
    set((s) => ({
      edges: addEdge(
        { ...connection, animated: true, style: { stroke: "#4fc3f7" } },
        s.edges,
      ),
    })),

  setSelectedNode: (id) => set({ selectedNode: id }),

  deleteSelectedNode: () => {
    const { selectedNode, nodes, edges } = get();
    if (!selectedNode) return;
    set({
      nodes: nodes.filter((n) => n.id !== selectedNode),
      edges: edges.filter(
        (e) => e.source !== selectedNode && e.target !== selectedNode,
      ),
      selectedNode: null,
    });
  },

  addDspNode: (nodeType) => {
    const id = `node-${++nodeCounter}`;
    const paramDefs = NODE_PARAM_DEFS[nodeType] ?? [];
    const newNode: Node = {
      id,
      type: "dspNode",
      position: {
        x: 120 + ((nodeCounter - 1) % 4) * 240,
        y: 160 + Math.floor((nodeCounter - 1) / 4) * 280,
      },
      data: {
        label: nodeType,
        nodeType,
        params: paramDefs.map((p) => p.default),
        paramDefs,
        isOutput: false,
      },
    };
    set((s) => ({ nodes: [...s.nodes, newNode] }));
  },

  updateParam: (nodeId, paramIndex, value) => {
    set((s) => ({
      nodes: s.nodes.map((n) =>
        n.id === nodeId
          ? {
              ...n,
              data: {
                ...n.data,
                params: n.data.params.map((v: number, i: number) =>
                  i === paramIndex ? value : v,
                ),
              },
            }
          : n,
      ),
    }));
    window.dispatchEvent(
      new CustomEvent("aether:param", {
        detail: { nodeId, paramIndex, value },
      }),
    );
  },

  loadSnapshot: (snapshot) => {
    const nodes: Node[] = snapshot.nodes.map((n, i) => ({
      id: String(n.id),
      type: "dspNode",
      position: {
        x: 120 + (i % 4) * 240,
        y: 160 + Math.floor(i / 4) * 280,
      },
      data: {
        label: n.node_type,
        nodeType: n.node_type,
        params: n.params,
        paramDefs: NODE_PARAM_DEFS[n.node_type] ?? [],
        isOutput: false,
      },
    }));
    const edges: Edge[] = snapshot.edges.map((e, i) => ({
      id: `e-${i}`,
      source: String(e.src_id),
      target: String(e.dst_id),
      targetHandle: `input-${e.slot}`,
      animated: true,
      label: e.slot > 0 ? `in ${e.slot}` : undefined,
      style: { stroke: "#4fc3f7" },
    }));
    set({ nodes, edges });
  },

  setWsStatus: (status) => set({ wsStatus: status }),
  setAudioActive: (active) => set({ audioActive: active }),
  setOutputNode: (id) => {
    set((s) => ({
      outputNodeId: id,
      nodes: s.nodes.map((n) => ({
        ...n,
        data: { ...n.data, isOutput: n.id === id },
      })),
    }));
  },
}));
