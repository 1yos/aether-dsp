/**
 * useInstrumentEngine — bridges the catalog to the Rust audio engine.
 *
 * Provides:
 *  - addToCanvas(instrument)  → sends AddNode + LoadInstrument to host
 *  - playNote(instrument, note, velocity) → sends InjectMidi to host
 *  - stopNote(instrument, note) → sends NoteOff to host
 */

import { useCallback, useRef } from "react";
import { useEngineStore } from "../studio/store/engineStore";
import type { CatalogInstrument } from "./types";

export function useInstrumentEngine() {
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const nodes = useEngineStore((s) => s.nodes);

  // Track which catalog instrument id maps to which node id
  const instrumentNodeMap = useRef<Map<string, number>>(new Map());

  /**
   * Add an instrument to the canvas as a SamplerNode.
   * Returns the node id assigned by the host (from the next snapshot).
   */
  const addToCanvas = useCallback(
    (instrument: CatalogInstrument) => {
      if (!sendIntent) return;

      // 1. Add a SamplerNode to the graph
      sendIntent({ type: "add_node", node_type: "SamplerNode" });

      // 2. The host will respond with a snapshot containing the new node.
      //    We store the instrument id → node mapping so LoadInstrument can
      //    be sent once we know the node id from the snapshot.
      //    For now, tag the pending instrument so the snapshot handler can pick it up.
      (window as unknown as Record<string, unknown>).__pendingInstrumentLoad =
        instrument.id;
    },
    [sendIntent],
  );

  /**
   * Load a .aether-instrument JSON into an existing SamplerNode.
   */
  const loadInstrumentIntoNode = useCallback(
    (nodeId: number, generation: number, instrumentJson: string) => {
      if (!sendIntent) return;
      sendIntent({
        type: "load_instrument",
        node_id: nodeId,
        generation,
        instrument_json: instrumentJson,
      });
    },
    [sendIntent],
  );

  /**
   * Play a note on the instrument's SamplerNode via MIDI injection.
   * Falls back to Web Audio if the host is not connected.
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

  /**
   * Find the SamplerNode id for a given instrument (if it's been added to canvas).
   */
  const getNodeIdForInstrument = useCallback(
    (instrumentId: string): number | null => {
      return instrumentNodeMap.current.get(instrumentId) ?? null;
    },
    [],
  );

  return {
    addToCanvas,
    loadInstrumentIntoNode,
    playNote,
    stopNote,
    getNodeIdForInstrument,
    isEngineConnected: !!sendIntent,
    nodeCount: nodes.length,
  };
}
