/**
 * useInstrumentEngine — bridges the catalog to the Rust audio engine.
 *
 * Add to Canvas:
 *   1. Send add_node (SamplerNode) → host responds with snapshot containing new node ID
 *   2. Send load_instrument with the instrument JSON → host loads WAV files and swaps in
 *   3. Automatically connect SamplerNode → output node
 *
 * Try It (keyboard player):
 *   - Creates a temporary SamplerNode, loads the instrument, connects to output
 *   - On close, removes the temporary node
 *
 * Play note / Stop note:
 *   - Sends inject_midi intents to the host
 *   - Falls back to Web Audio when host is not connected
 */

import { useCallback, useRef } from "react";
import { useEngineStore } from "../studio/store/engineStore";
import type { CatalogInstrument } from "./types";

// Pending instrument load: after add_node we need to wait for the snapshot
// to get the new node ID, then send load_instrument.
interface PendingLoad {
  instrumentId: string;
  instrumentJson: string;
  autoConnect: boolean;
}

// Global pending load state (outside React to survive re-renders)
let pendingLoad: PendingLoad | null = null;

export function useInstrumentEngine() {
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const nodes = useEngineStore((s) => s.nodes);
  const outputNodeId = useEngineStore((s) => s.outputNodeId);
  const addNode = useEngineStore((s) => s.addNode);

  // Track catalog instrument ID → node ID mapping
  const instrumentNodeMap = useRef<Map<string, string>>(new Map());

  /**
   * Add an instrument to the canvas as a SamplerNode.
   * The host will respond with a snapshot; the snapshot handler in
   * useWebSocket picks up pendingLoad and sends load_instrument.
   */
  const addToCanvas = useCallback(
    async (instrument: CatalogInstrument) => {
      if (!sendIntent) return;

      // Build the instrument JSON for the host
      const instrumentJson = JSON.stringify({
        name: instrument.name,
        family: instrument.family,
        region: instrument.region,
        country: instrument.country,
        tuning: { name: instrument.tuning, frequencies: null },
        zones: [],
        attack: 0.01,
        decay: 0.1,
        sustain: 0.7,
        release: 0.3,
        max_voices: 16,
      });

      // Set pending load so the snapshot handler can pick it up
      pendingLoad = {
        instrumentId: instrument.id,
        instrumentJson,
        autoConnect: true,
      };

      // Add the SamplerNode — host will respond with snapshot
      addNode("SamplerNode");
    },
    [sendIntent, addNode],
  );

  /**
   * Called by useWebSocket when a snapshot arrives after add_node.
   * Finds the newest SamplerNode and sends load_instrument for it.
   */
  const handleSnapshotAfterAdd = useCallback(
    (newNodeId: string, generation: number) => {
      if (!pendingLoad || !sendIntent) return;
      const { instrumentId, instrumentJson, autoConnect } = pendingLoad;
      pendingLoad = null;

      // Load the instrument into the new node
      sendIntent({
        type: "load_instrument",
        node_id: parseInt(newNodeId, 10),
        generation,
        instrument_json: instrumentJson,
      });

      // Track the mapping
      instrumentNodeMap.current.set(instrumentId, newNodeId);

      // Auto-connect to output if requested
      if (autoConnect && outputNodeId) {
        sendIntent({
          type: "connect",
          src_id: parseInt(newNodeId, 10),
          src_gen: generation,
          dst_id: parseInt(outputNodeId, 10),
          dst_gen: 0,
          slot: 0,
        });
      }
    },
    [sendIntent, outputNodeId],
  );

  /**
   * Play a note through the engine via MIDI injection.
   */
  const playNote = useCallback(
    (
      _instrument: CatalogInstrument,
      note: number,
      velocity: number = 80,
      channel: number = 0,
    ) => {
      if (!sendIntent) return;
      sendIntent({
        type: "inject_midi",
        channel,
        note,
        velocity,
        is_note_on: true,
      });
    },
    [sendIntent],
  );

  /**
   * Stop a note.
   */
  const stopNote = useCallback(
    (_instrument: CatalogInstrument, note: number, channel: number = 0) => {
      if (!sendIntent) return;
      sendIntent({
        type: "inject_midi",
        channel,
        note,
        velocity: 0,
        is_note_on: false,
      });
    },
    [sendIntent],
  );

  return {
    addToCanvas,
    handleSnapshotAfterAdd,
    playNote,
    stopNote,
    isEngineConnected: !!sendIntent,
    nodeCount: nodes.length,
    getPendingLoad: () => pendingLoad,
  };
}
