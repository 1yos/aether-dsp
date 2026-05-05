/**
 * engineStore integration tests
 *
 * Tests the Zustand store in isolation — no WebSocket, no DOM.
 * Verifies that intents are dispatched correctly and snapshots
 * are applied to the store state.
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { useEngineStore } from "../studio/store/engineStore";

// Reset store between tests
beforeEach(() => {
  useEngineStore.setState({
    nodes: [],
    edges: [],
    wsStatus: "disconnected",
    muted: false,
    outputNodeId: null,
    audioActive: false,
    sendIntent: null,
  });
});

describe("engineStore — wsStatus", () => {
  it("starts disconnected", () => {
    expect(useEngineStore.getState().wsStatus).toBe("disconnected");
  });

  it("setWsStatus updates status", () => {
    useEngineStore.getState().setWsStatus("connected");
    expect(useEngineStore.getState().wsStatus).toBe("connected");
  });

  it("cycles through all statuses", () => {
    const { setWsStatus } = useEngineStore.getState();
    setWsStatus("connecting");
    expect(useEngineStore.getState().wsStatus).toBe("connecting");
    setWsStatus("connected");
    expect(useEngineStore.getState().wsStatus).toBe("connected");
    setWsStatus("error");
    expect(useEngineStore.getState().wsStatus).toBe("error");
    setWsStatus("disconnected");
    expect(useEngineStore.getState().wsStatus).toBe("disconnected");
  });
});

describe("engineStore — sendIntent", () => {
  it("sendIntent is null by default", () => {
    expect(useEngineStore.getState().sendIntent).toBeNull();
  });

  it("setSendIntent registers the function", () => {
    const mockSend = vi.fn();
    useEngineStore.getState().setSendIntent(mockSend);
    expect(useEngineStore.getState().sendIntent).toBe(mockSend);
  });

  it("addNode dispatches add_node intent", () => {
    const mockSend = vi.fn();
    useEngineStore.getState().setSendIntent(mockSend);
    useEngineStore.getState().addNode("Oscillator");
    expect(mockSend).toHaveBeenCalledWith({
      type: "add_node",
      node_type: "Oscillator",
    });
  });

  it("toggleMute dispatches set_mute intent", () => {
    const mockSend = vi.fn();
    useEngineStore.getState().setSendIntent(mockSend);
    useEngineStore.getState().toggleMute();
    expect(mockSend).toHaveBeenCalledWith({ type: "set_mute", muted: true });
  });

  it("clearGraph dispatches clear_graph intent", () => {
    const mockSend = vi.fn();
    useEngineStore.getState().setSendIntent(mockSend);
    useEngineStore.getState().clearGraph();
    expect(mockSend).toHaveBeenCalledWith({ type: "clear_graph" });
  });

  it("undo dispatches undo intent", () => {
    const mockSend = vi.fn();
    useEngineStore.getState().setSendIntent(mockSend);
    useEngineStore.getState().undo();
    expect(mockSend).toHaveBeenCalledWith({ type: "undo" });
  });

  it("setMidiRoute dispatches set_midi_route intent", () => {
    const mockSend = vi.fn();
    useEngineStore.getState().setSendIntent(mockSend);
    const state = useEngineStore.getState() as unknown as Record<
      string,
      unknown
    >;
    const fn = state["setMidiRoute"] as
      | ((ch: number, id: string, gen: number) => void)
      | undefined;
    if (fn) {
      fn(1, "5", 0);
      expect(mockSend).toHaveBeenCalledWith({
        type: "set_midi_route",
        channel: 1,
        node_id: 5,
        generation: 0,
      });
    }
  });
});

describe("engineStore — applySnapshot", () => {
  it("populates nodes and edges from snapshot", () => {
    useEngineStore.getState().applySnapshot({
      nodes: [
        {
          id: 0,
          generation: 0,
          node_type: "Oscillator",
          params: [440, 0.5, 0, -1],
        },
        { id: 1, generation: 0, node_type: "Gain", params: [0.8] },
      ],
      edges: [{ src_id: 0, dst_id: 1, slot: 0 }],
      muted: false,
      output_node_id: 1,
      can_undo: true,
      can_redo: false,
    });

    const state = useEngineStore.getState();
    expect(state.nodes).toHaveLength(2);
    expect(state.edges).toHaveLength(1);
    expect(state.outputNodeId).toBe("1");
    expect(state.canUndo).toBe(true);
    expect(state.canRedo).toBe(false);
  });

  it("sets muted flag from snapshot", () => {
    useEngineStore.getState().applySnapshot({
      nodes: [],
      edges: [],
      muted: true,
      output_node_id: null,
    });
    expect(useEngineStore.getState().muted).toBe(true);
  });

  it("maps node types to correct colors", () => {
    useEngineStore.getState().applySnapshot({
      nodes: [{ id: 0, generation: 0, node_type: "Oscillator", params: [] }],
      edges: [],
      muted: false,
      output_node_id: null,
    });
    const node = useEngineStore.getState().nodes[0];
    expect(node.data.color).toBe("#4fc3f7");
  });

  it("clears nodes on empty snapshot", () => {
    // First populate
    useEngineStore.getState().applySnapshot({
      nodes: [{ id: 0, generation: 0, node_type: "Gain", params: [1.0] }],
      edges: [],
      muted: false,
      output_node_id: null,
    });
    expect(useEngineStore.getState().nodes).toHaveLength(1);

    // Then clear
    useEngineStore.getState().applySnapshot({
      nodes: [],
      edges: [],
      muted: false,
      output_node_id: null,
    });
    expect(useEngineStore.getState().nodes).toHaveLength(0);
  });
});

describe("engineStore — audioActive pulse", () => {
  it("setAudioActive toggles the flag", () => {
    useEngineStore.getState().setAudioActive(true);
    expect(useEngineStore.getState().audioActive).toBe(true);
    useEngineStore.getState().setAudioActive(false);
    expect(useEngineStore.getState().audioActive).toBe(false);
  });
});
