/**
 * useSampleLibrary — React hook for the sample library system.
 *
 * Provides:
 *   - manifest: all available packs from GitHub Releases
 *   - packStatuses: install status of each pack
 *   - diskUsage: total bytes used by installed samples
 *   - downloadPack(id): start a download
 *   - uninstallPack(id): remove a pack
 *   - instrumentHasSamples(id): check if an instrument has real samples
 *   - resolveSamplePath(instrumentId, relativePath): get absolute path
 *
 * Works in Tauri app only. In browser dev mode, all operations are no-ops
 * and instrumentHasSamples always returns false (synthesis fallback).
 */

import { useState, useEffect, useCallback, useRef } from "react";

// ── Types ─────────────────────────────────────────────────────────────────────

export type PackStatus =
  | { type: "not_installed" }
  | { type: "downloading"; progress_pct: number }
  | { type: "installed"; version: string }
  | {
      type: "update_available";
      installed_version: string;
      latest_version: string;
    };

export interface SamplePack {
  id: string;
  name: string;
  description: string;
  category: string;
  version: string;
  filename: string;
  compressed_bytes: number;
  uncompressed_bytes: number;
  instrument_ids: string[];
  quality: "lite" | "full";
  full_pack_id: string | null;
  license: string;
  attribution: string;
}

export interface SampleManifest {
  version: string;
  packs: SamplePack[];
}

// ── Tauri bridge ──────────────────────────────────────────────────────────────

const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const tauri = isTauri ? (window as any).__TAURI__ : null;

async function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!tauri) throw new Error("Not in Tauri");
  return tauri.tauri.invoke(cmd, args);
}

// ── Hook ──────────────────────────────────────────────────────────────────────

export interface SampleLibraryState {
  manifest: SampleManifest | null;
  packStatuses: Record<string, PackStatus>;
  diskUsage: number;
  loading: boolean;
  error: string | null;
  isTauri: boolean;
  refreshManifest: () => Promise<void>;
  downloadPack: (packId: string) => Promise<void>;
  uninstallPack: (packId: string) => Promise<void>;
  instrumentHasSamples: (instrumentId: string) => boolean;
  resolveSamplePath: (
    instrumentId: string,
    relativePath: string,
  ) => Promise<string | null>;
}

export function useSampleLibrary(): SampleLibraryState {
  const [manifest, setManifest] = useState<SampleManifest | null>(null);
  const [packStatuses, setPackStatuses] = useState<Record<string, PackStatus>>(
    {},
  );
  const [diskUsage, setDiskUsage] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  const refreshStatuses = useCallback(async () => {
    if (!isTauri) return;
    try {
      const json = await invoke<string>("get_pack_statuses");
      const pairs: [string, PackStatus][] = JSON.parse(json);
      const map: Record<string, PackStatus> = {};
      for (const [id, status] of pairs) map[id] = status;
      setPackStatuses(map);
    } catch (e) {
      console.error("useSampleLibrary: failed to get pack statuses", e);
    }
  }, []);

  const refreshDiskUsage = useCallback(async () => {
    if (!isTauri) return;
    try {
      const bytes = await invoke<number>("get_sample_disk_usage");
      setDiskUsage(bytes);
    } catch (e) {
      console.error("useSampleLibrary: failed to get disk usage", e);
    }
  }, []);

  const refreshManifest = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      if (isTauri) {
        const json = await invoke<string>("fetch_sample_manifest");
        setManifest(JSON.parse(json));
      } else {
        // Dev mode: try to load from public assets
        try {
          const res = await fetch("/assets/samples-manifest.json");
          if (res.ok) setManifest(await res.json());
        } catch {
          // Silently ignore in dev mode
        }
      }
      await refreshStatuses();
      await refreshDiskUsage();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [refreshStatuses, refreshDiskUsage]);

  // Initial load
  useEffect(() => {
    refreshManifest();

    // Listen for download progress events
    if (isTauri && tauri?.event?.listen) {
      tauri.event
        .listen(
          "sample-download-progress",
          (e: {
            payload: {
              pack_id: string;
              bytes_downloaded: number;
              bytes_total: number;
              phase: string;
            };
          }) => {
            const { pack_id, bytes_downloaded, bytes_total, phase } = e.payload;
            setPackStatuses((prev) => ({
              ...prev,
              [pack_id]:
                phase === "complete"
                  ? { type: "installed", version: "1.0.0" }
                  : phase === "failed"
                    ? { type: "not_installed" }
                    : {
                        type: "downloading",
                        progress_pct:
                          bytes_total > 0
                            ? Math.round((bytes_downloaded / bytes_total) * 100)
                            : 0,
                      },
            }));
            if (phase === "complete") {
              refreshStatuses();
              refreshDiskUsage();
            }
          },
        )
        .then((fn: () => void) => {
          unlistenRef.current = fn;
        });
    }

    return () => {
      unlistenRef.current?.();
    };
  }, [refreshManifest, refreshStatuses, refreshDiskUsage]);

  const downloadPack = useCallback(async (packId: string) => {
    if (!isTauri) return;
    setPackStatuses((prev) => ({
      ...prev,
      [packId]: { type: "downloading", progress_pct: 0 },
    }));
    try {
      await invoke("download_sample_pack", { packId });
    } catch (e) {
      setPackStatuses((prev) => ({
        ...prev,
        [packId]: { type: "not_installed" },
      }));
      setError(`Download failed: ${e}`);
    }
  }, []);

  const uninstallPack = useCallback(
    async (packId: string) => {
      if (!isTauri) return;
      try {
        await invoke("uninstall_sample_pack", { packId });
        await refreshStatuses();
        await refreshDiskUsage();
      } catch (e) {
        setError(`Uninstall failed: ${e}`);
      }
    },
    [refreshStatuses, refreshDiskUsage],
  );

  const instrumentHasSamples = useCallback(
    (instrumentId: string): boolean => {
      if (!isTauri) return false;
      // Check if any installed pack covers this instrument
      return Object.values(packStatuses).some(
        (s) =>
          (s.type === "installed" || s.type === "update_available") &&
          manifest?.packs.some((p) => p.instrument_ids.includes(instrumentId)),
      );
    },
    [packStatuses, manifest],
  );

  const resolveSamplePath = useCallback(
    async (
      instrumentId: string,
      relativePath: string,
    ): Promise<string | null> => {
      if (!isTauri) return null;
      try {
        return await invoke<string | null>("resolve_sample_path", {
          instrumentId,
          relativePath,
        });
      } catch {
        return null;
      }
    },
    [],
  );

  return {
    manifest,
    packStatuses,
    diskUsage,
    loading,
    error,
    isTauri,
    refreshManifest,
    downloadPack,
    uninstallPack,
    instrumentHasSamples,
    resolveSamplePath,
  };
}
