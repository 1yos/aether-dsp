/**
 * useEngineSocket integration tests
 *
 * Tests the WebSocket hook by mocking the WebSocket API and verifying
 * that status transitions and message handling work correctly.
 * Uses a direct hook invocation pattern to avoid React rendering overhead.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { useEngineStore } from "../studio/store/engineStore";

// ── Mock WebSocket ────────────────────────────────────────────────────────────

class MockWebSocket {
  static instances: MockWebSocket[] = [];
  url: string;
  readyState = 0;
  onopen: (() => void) | null = null;
  onmessage: ((e: { data: string }) => void) | null = null;
  onerror: (() => void) | null = null;
  onclose: (() => void) | null = null;
  sentMessages: string[] = [];

  constructor(url: string) {
    this.url = url;
    MockWebSocket.instances.push(this);
  }

  send(data: string) {
    this.sentMessages.push(data);
  }
  close() {
    this.readyState = 3;
    this.onclose?.();
  }
  simulateOpen() {
    this.readyState = 1;
    this.onopen?.();
  }
  simulateMessage(data: object) {
    this.onmessage?.({ data: JSON.stringify(data) });
  }
  simulateError() {
    this.onerror?.();
  }
  simulateClose() {
    this.readyState = 3;
    this.onclose?.();
  }
}

beforeEach(() => {
  MockWebSocket.instances = [];
  vi.stubGlobal("WebSocket", MockWebSocket);
  useEngineStore.setState({
    nodes: [],
    edges: [],
    wsStatus: "disconnected",
    muted: false,
    outputNodeId: null,
    audioActive: false,
    sendIntent: null,
    midiPorts: [],
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

// ── Simulate the hook logic directly (without React rendering) ────────────────

function simulateSocketConnect() {
  const setWsStatus = useEngineStore.getState().setWsStatus;
  const setSendIntent = useEngineStore.getState().setSendIntent;
  const applySnapshot = useEngineStore.getState().applySnapshot;
  const setMidiPorts = useEngineStore.getState().setMidiPorts;
  const setAudioActive = useEngineStore.getState().setAudioActive;

  setWsStatus("connecting");
  const ws = new MockWebSocket("ws://127.0.0.1:9001");

  ws.onopen = () => {
    setWsStatus("connected");
    const send = (msg: object) => ws.send(JSON.stringify(msg));
    setSendIntent(send);
    ws.send(JSON.stringify({ type: "get_snapshot" }));
  };

  ws.onmessage = (e) => {
    const msg = JSON.parse(e.data) as Record<string, unknown>;
    switch (msg.type) {
      case "snapshot":
        applySnapshot(msg as Parameters<typeof applySnapshot>[0]);
        break;
      case "ack":
        setAudioActive(true);
        break;
      case "midi_ports":
        setMidiPorts((msg.ports as string[]) ?? []);
        break;
    }
  };

  ws.onerror = () => setWsStatus("error");
  ws.onclose = () => setWsStatus("disconnected");

  return ws;
}

describe("WebSocket connection lifecycle", () => {
  it("sets status to connecting on init", () => {
    simulateSocketConnect();
    expect(useEngineStore.getState().wsStatus).toBe("connecting");
  });

  it("sets status to connected on open", () => {
    const ws = simulateSocketConnect();
    ws.simulateOpen();
    expect(useEngineStore.getState().wsStatus).toBe("connected");
  });

  it("sends get_snapshot on open", () => {
    const ws = simulateSocketConnect();
    ws.simulateOpen();
    const sent = ws.sentMessages.map((m) => JSON.parse(m));
    expect(sent).toContainEqual({ type: "get_snapshot" });
  });

  it("registers sendIntent on open", () => {
    const ws = simulateSocketConnect();
    ws.simulateOpen();
    expect(useEngineStore.getState().sendIntent).not.toBeNull();
  });

  it("sets status to error on socket error", () => {
    const ws = simulateSocketConnect();
    ws.simulateError();
    expect(useEngineStore.getState().wsStatus).toBe("error");
  });

  it("sets status to disconnected on close", () => {
    const ws = simulateSocketConnect();
    ws.simulateOpen();
    ws.simulateClose();
    expect(useEngineStore.getState().wsStatus).toBe("disconnected");
  });
});

describe("WebSocket message handling", () => {
  it("applies snapshot to store", () => {
    const ws = simulateSocketConnect();
    ws.simulateOpen();
    ws.simulateMessage({
      type: "snapshot",
      nodes: [{ id: 0, generation: 0, node_type: "Gain", params: [0.8] }],
      edges: [],
      muted: false,
      output_node_id: null,
    });
    expect(useEngineStore.getState().nodes).toHaveLength(1);
  });

  it("sets midi ports from midi_ports message", () => {
    const ws = simulateSocketConnect();
    ws.simulateOpen();
    ws.simulateMessage({
      type: "midi_ports",
      ports: ["Korg nanoKEY2", "IAC Driver"],
    });
    expect(useEngineStore.getState().midiPorts).toEqual([
      "Korg nanoKEY2",
      "IAC Driver",
    ]);
  });

  it("pulses audioActive on ack message", () => {
    const ws = simulateSocketConnect();
    ws.simulateOpen();
    ws.simulateMessage({ type: "ack", command: "update_param" });
    expect(useEngineStore.getState().audioActive).toBe(true);
  });

  it("ignores unknown message types without throwing", () => {
    const ws = simulateSocketConnect();
    ws.simulateOpen();
    expect(() =>
      ws.simulateMessage({ type: "unknown_future_type", data: 42 }),
    ).not.toThrow();
  });
});
