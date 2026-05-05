/**
 * usePresetLoader — loads synthesis presets for world instruments.
 *
 * When an instrument is opened (keyboard player or add to canvas),
 * this hook fetches the instrument's .aether-instrument preset file
 * and sends the patch to the engine via load_patch intent.
 *
 * Preset format: a PatchDef JSON with nodes, connections, output_node.
 * The engine loads the patch and the instrument is immediately playable.
 *
 * For instruments with real samples (hasSamples: true), the preset
 * includes a SamplerNode that the sample library populates with WAV files.
 * For synthesized instruments, the preset uses DSP nodes directly.
 */

import { useCallback, useRef } from "react";
import { useEngineStore } from "../studio/store/engineStore";

export interface AetherInstrumentPreset {
  name: string;
  description: string;
  instrument_id: string;
  source: "synthesis" | "samples" | "hybrid";
  /** PatchDef nodes */
  nodes: Array<{
    id: string;
    type: string;
    params: Record<string, number>;
  }>;
  connections: Array<{
    from: string;
    to: string;
    slot: number;
  }>;
  output_node: string;
  /** Optional tuning system name */
  tuning?: string;
}

// Cache loaded presets to avoid re-fetching
const presetCache = new Map<string, AetherInstrumentPreset>();

export function usePresetLoader() {
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const loadPatch = useEngineStore((s) => s.loadPatch);
  const loadingRef = useRef<Set<string>>(new Set());

  /**
   * Load a preset from the public/instruments/ directory.
   * Returns the preset or null if not found.
   */
  const fetchPreset = useCallback(
    async (
      instrumentId: string,
      filePath: string,
    ): Promise<AetherInstrumentPreset | null> => {
      if (presetCache.has(instrumentId)) {
        return presetCache.get(instrumentId)!;
      }

      // filePath is like "instruments/krar.aether-instrument"
      // Served from /public/
      const url = `/${filePath}`;
      try {
        const res = await fetch(url);
        if (!res.ok) return null;
        const preset: AetherInstrumentPreset = await res.json();
        presetCache.set(instrumentId, preset);
        return preset;
      } catch {
        return null;
      }
    },
    [],
  );

  /**
   * Load an instrument preset into the engine.
   * Sends a load_patch intent with the preset's node graph.
   */
  const loadInstrumentPreset = useCallback(
    async (instrumentId: string, filePath: string): Promise<boolean> => {
      if (!sendIntent || loadingRef.current.has(instrumentId)) return false;
      loadingRef.current.add(instrumentId);

      try {
        const preset = await fetchPreset(instrumentId, filePath);
        if (!preset) return false;

        // Convert preset to PatchDef format expected by the engine
        const patch = {
          nodes: preset.nodes.map((n) => ({
            id: n.id,
            type: n.type,
            params: n.params,
          })),
          connections: preset.connections,
          output_node: preset.output_node,
        };

        loadPatch(patch);
        return true;
      } finally {
        loadingRef.current.delete(instrumentId);
      }
    },
    [sendIntent, fetchPreset, loadPatch],
  );

  /**
   * Preload a preset without loading it into the engine.
   * Call this when hovering over an instrument to reduce latency.
   */
  const preloadPreset = useCallback(
    async (instrumentId: string, filePath: string): Promise<void> => {
      await fetchPreset(instrumentId, filePath);
    },
    [fetchPreset],
  );

  return {
    loadInstrumentPreset,
    preloadPreset,
    isPresetCached: (id: string) => presetCache.has(id),
  };
}
