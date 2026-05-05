/**
 * useInstrumentEngine — bridges the catalog to the Rust audio engine.
 *
 * When an instrument is opened, it loads the .aether-instrument preset
 * (synthesis patch) via load_patch. For instruments with real samples,
 * the preset includes a SamplerNode that the sample library populates.
 */

import { useCallback, useRef } from "react";
import { useEngineStore } from "../studio/store/engineStore";
import type { CatalogInstrument } from "./types";

// Preset cache — avoid re-fetching on every open
const presetCache = new Map<string, object>();

async function fetchPreset(filePath: string): Promise<object | null> {
  if (presetCache.has(filePath)) return presetCache.get(filePath)!;
  try {
    const res = await fetch(`/${filePath}`);
    if (!res.ok) return null;
    const preset = await res.json();
    presetCache.set(filePath, preset);
    return preset;
  } catch {
    return null;
  }
}

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
  const loadPatch = useEngineStore((s) => s.loadPatch);

  // Track catalog instrument ID → node ID mapping
  const instrumentNodeMap = useRef<Map<string, string>>(new Map());

  /**
   * Load an instrument's synthesis preset into the engine.
   * Called when opening the keyboard player or adding to canvas.
   */
  const loadInstrumentPreset = useCallback(
    async (instrument: CatalogInstrument & { file?: string }) => {
      if (!sendIntent) return false;

      const filePath =
        instrument.file ?? `instruments/${instrument.id}.aether-instrument`;
      const preset = await fetchPreset(filePath);
      if (!preset) return false;

      // The preset is a PatchDef — send it directly to the engine
      loadPatch(preset);
      return true;
    },
    [sendIntent, loadPatch],
  );

  /**
   * Add an instrument to the canvas as a SamplerNode.
   * The host will respond with a snapshot; the snapshot handler in
   * useWebSocket picks up pendingLoad and sends load_instrument.
   */
  const addToCanvas = useCallback(
    async (instrument: CatalogInstrument & { file?: string }) => {
      if (!sendIntent) return;

      // Try to load the synthesis preset first
      const loaded = await loadInstrumentPreset(instrument);
      if (loaded) return;

      // Fallback: add a SamplerNode and load instrument JSON
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

      pendingLoad = {
        instrumentId: instrument.id,
        instrumentJson,
        autoConnect: true,
      };

      addNode("SamplerNode");
    },
    [sendIntent, addNode, loadInstrumentPreset],
  );

  /**
   * Called by useWebSocket when a snapshot arrives after add_node.
   */
  const handleSnapshotAfterAdd = useCallback(
    (newNodeId: string, generation: number) => {
      if (!pendingLoad || !sendIntent) return;
      const { instrumentId, instrumentJson, autoConnect } = pendingLoad;
      pendingLoad = null;

      sendIntent({
        type: "load_instrument",
        node_id: parseInt(newNodeId, 10),
        generation,
        instrument_json: instrumentJson,
      });

      instrumentNodeMap.current.set(instrumentId, newNodeId);

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
    loadInstrumentPreset,
    handleSnapshotAfterAdd,
    playNote,
    stopNote,
    isEngineConnected: !!sendIntent,
    nodeCount: nodes.length,
    getPendingLoad: () => pendingLoad,
  };
}
