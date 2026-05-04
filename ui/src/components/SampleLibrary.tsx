/**
 * Sample Library Panel
 *
 * Manages on-demand download of sample packs from GitHub Releases.
 * Shows all available packs, their install status, size, and license.
 * Provides download/uninstall controls with real-time progress.
 *
 * Works in two modes:
 *   - Tauri app: uses invoke() to call Rust commands
 *   - Browser dev: shows mock data with disabled download buttons
 */

import { useState, useEffect, useCallback } from "react";

// ── Types (mirror aether-samples Rust types) ──────────────────────────────────

type PackCategory =
  | "drums"
  | "piano"
  | "strings"
  | "brass"
  | "woodwinds"
  | "guitar"
  | "bass"
  | "choir"
  | "world_instruments"
  | "impulse_responses";

type PackQuality = "lite" | "full";

type PackStatus =
  | { type: "not_installed" }
  | { type: "downloading"; progress_pct: number }
  | { type: "installed"; version: string }
  | {
      type: "update_available";
      installed_version: string;
      latest_version: string;
    };

interface SamplePack {
  id: string;
  name: string;
  description: string;
  category: PackCategory;
  version: string;
  filename: string;
  sha256: string;
  compressed_bytes: number;
  uncompressed_bytes: number;
  instrument_ids: string[];
  quality: PackQuality;
  full_pack_id: string | null;
  license: string;
  attribution: string;
}

interface SampleManifest {
  version: string;
  packs: SamplePack[];
}

interface DownloadProgress {
  pack_id: string;
  bytes_downloaded: number;
  bytes_total: number;
  phase:
    | "downloading"
    | "verifying"
    | "extracting"
    | "complete"
    | { failed: { message: string } };
}

// ── Tauri bridge ──────────────────────────────────────────────────────────────

const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

async function tauriInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauri) throw new Error("Not running in Tauri");
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const { invoke } = (window as any).__TAURI__.tauri;
  return invoke(cmd, args);
}

async function tauriListen(
  event: string,
  handler: (payload: unknown) => void,
): Promise<() => void> {
  if (!isTauri) return () => {};
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const { listen } = (window as any).__TAURI__.event;
  return listen(event, (e: { payload: unknown }) => handler(e.payload));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function formatBytes(bytes: number): string {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

const CATEGORY_LABELS: Record<PackCategory, string> = {
  drums: "Drums",
  piano: "Piano",
  strings: "Strings",
  brass: "Brass",
  woodwinds: "Woodwinds",
  guitar: "Guitar",
  bass: "Bass",
  choir: "Choir",
  world_instruments: "World",
  impulse_responses: "Reverb IRs",
};

const CATEGORY_ICONS: Record<PackCategory, string> = {
  drums: "🥁",
  piano: "🎹",
  strings: "🎻",
  brass: "🎺",
  woodwinds: "🎷",
  guitar: "🎸",
  bass: "🎸",
  choir: "🎤",
  world_instruments: "🌍",
  impulse_responses: "🏛️",
};

const CATEGORY_ORDER: PackCategory[] = [
  "drums",
  "piano",
  "strings",
  "brass",
  "woodwinds",
  "guitar",
  "bass",
  "choir",
  "world_instruments",
  "impulse_responses",
];

// ── Component ─────────────────────────────────────────────────────────────────

interface SampleLibraryProps {
  onClose?: () => void;
}

export function SampleLibrary({ onClose }: SampleLibraryProps) {
  const [manifest, setManifest] = useState<SampleManifest | null>(null);
  const [statuses, setStatuses] = useState<Record<string, PackStatus>>({});
  const [diskUsage, setDiskUsage] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<
    PackCategory | "all"
  >("all");
  const [showLiteOnly, setShowLiteOnly] = useState(true);

  // Load manifest and statuses on mount
  useEffect(() => {
    loadManifest();
    loadStatuses();
    loadDiskUsage();

    // Listen for download progress events from Tauri
    let unlisten: (() => void) | null = null;
    tauriListen("sample-download-progress", (payload) => {
      const progress = payload as DownloadProgress;
      setStatuses((prev) => ({
        ...prev,
        [progress.pack_id]:
          progress.phase === "complete"
            ? { type: "installed", version: "1.0.0" }
            : typeof progress.phase === "object" && "failed" in progress.phase
              ? { type: "not_installed" }
              : {
                  type: "downloading",
                  progress_pct:
                    progress.bytes_total > 0
                      ? Math.round(
                          (progress.bytes_downloaded / progress.bytes_total) *
                            100,
                        )
                      : 0,
                },
      }));
      if (progress.phase === "complete") {
        loadStatuses();
        loadDiskUsage();
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const loadManifest = async () => {
    setLoading(true);
    setError(null);
    try {
      if (isTauri) {
        const json = await tauriInvoke<string>("fetch_sample_manifest");
        setManifest(JSON.parse(json));
      } else {
        // Dev mode: load from local assets
        const res = await fetch("/assets/samples-manifest.json");
        if (res.ok) {
          setManifest(await res.json());
        } else {
          setError(
            "Could not load sample manifest. Run in Tauri app for full functionality.",
          );
        }
      }
    } catch (e) {
      setError(`Failed to load manifest: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const loadStatuses = async () => {
    if (!isTauri) return;
    try {
      const json = await tauriInvoke<string>("get_pack_statuses");
      const pairs: [string, PackStatus][] = JSON.parse(json);
      const map: Record<string, PackStatus> = {};
      for (const [id, status] of pairs) {
        map[id] = status;
      }
      setStatuses(map);
    } catch (e) {
      console.error("Failed to load pack statuses:", e);
    }
  };

  const loadDiskUsage = async () => {
    if (!isTauri) return;
    try {
      const bytes = await tauriInvoke<number>("get_sample_disk_usage");
      setDiskUsage(bytes);
    } catch (e) {
      console.error("Failed to get disk usage:", e);
    }
  };

  const handleDownload = useCallback(async (packId: string) => {
    if (!isTauri) return;
    setStatuses((prev) => ({
      ...prev,
      [packId]: { type: "downloading", progress_pct: 0 },
    }));
    try {
      await tauriInvoke("download_sample_pack", { packId });
    } catch (e) {
      setStatuses((prev) => ({ ...prev, [packId]: { type: "not_installed" } }));
      setError(`Download failed: ${e}`);
    }
  }, []);

  const handleUninstall = useCallback(async (packId: string) => {
    if (!isTauri) return;
    if (
      !confirm(`Uninstall this sample pack? The audio files will be deleted.`)
    )
      return;
    try {
      await tauriInvoke("uninstall_sample_pack", { packId });
      await loadStatuses();
      await loadDiskUsage();
    } catch (e) {
      setError(`Uninstall failed: ${e}`);
    }
  }, []);

  // Filter packs
  const visiblePacks =
    manifest?.packs.filter((p) => {
      if (selectedCategory !== "all" && p.category !== selectedCategory)
        return false;
      if (showLiteOnly && p.quality === "full") return false;
      return true;
    }) ?? [];

  // Group by category
  const grouped = CATEGORY_ORDER.reduce<Record<string, SamplePack[]>>(
    (acc, cat) => {
      const packs = visiblePacks.filter((p) => p.category === cat);
      if (packs.length > 0) acc[cat] = packs;
      return acc;
    },
    {},
  );

  const installedCount = Object.values(statuses).filter(
    (s) => s.type === "installed" || s.type === "update_available",
  ).length;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: "#060c12",
        color: "#e0e8f0",
        fontFamily: "'Inter', system-ui, sans-serif",
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "16px 20px",
          borderBottom: "1px solid #0f1e2e",
          flexShrink: 0,
        }}
      >
        <div>
          <div style={{ fontSize: 18, fontWeight: 700, color: "#e0e8f0" }}>
            Sample Library
          </div>
          <div style={{ fontSize: 12, color: "#4a6a8a", marginTop: 2 }}>
            {installedCount} packs installed · {formatBytes(diskUsage)} used
          </div>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <button
            onClick={loadManifest}
            disabled={loading}
            style={{
              padding: "5px 12px",
              background: "transparent",
              border: "1px solid #1a2a3a",
              borderRadius: 6,
              color: "#4a6a8a",
              fontSize: 12,
              cursor: loading ? "not-allowed" : "pointer",
            }}
          >
            {loading ? "Loading…" : "↻ Refresh"}
          </button>
          {onClose && (
            <button
              onClick={onClose}
              style={{
                width: 28,
                height: 28,
                background: "transparent",
                border: "1px solid #1a2a3a",
                borderRadius: 6,
                color: "#4a6a8a",
                cursor: "pointer",
                fontSize: 14,
              }}
            >
              ✕
            </button>
          )}
        </div>
      </div>

      {/* Filters */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          padding: "10px 20px",
          borderBottom: "1px solid #0f1e2e",
          flexShrink: 0,
          flexWrap: "wrap",
        }}
      >
        <button
          onClick={() => setSelectedCategory("all")}
          style={filterBtnStyle(selectedCategory === "all")}
        >
          All
        </button>
        {CATEGORY_ORDER.map((cat) => (
          <button
            key={cat}
            onClick={() => setSelectedCategory(cat)}
            style={filterBtnStyle(selectedCategory === cat)}
          >
            {CATEGORY_ICONS[cat]} {CATEGORY_LABELS[cat]}
          </button>
        ))}
        <div
          style={{
            marginLeft: "auto",
            display: "flex",
            alignItems: "center",
            gap: 6,
          }}
        >
          <label
            style={{
              fontSize: 11,
              color: "#4a6a8a",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              gap: 4,
            }}
          >
            <input
              type="checkbox"
              checked={showLiteOnly}
              onChange={(e) => setShowLiteOnly(e.target.checked)}
              style={{ cursor: "pointer" }}
            />
            Lite only
          </label>
        </div>
      </div>

      {/* Error banner */}
      {error && (
        <div
          style={{
            padding: "10px 20px",
            background: "rgba(239,83,80,0.1)",
            borderBottom: "1px solid rgba(239,83,80,0.2)",
            color: "#ef5350",
            fontSize: 12,
            flexShrink: 0,
          }}
        >
          {error}
          <button
            onClick={() => setError(null)}
            style={{
              marginLeft: 8,
              background: "none",
              border: "none",
              color: "#ef5350",
              cursor: "pointer",
            }}
          >
            ✕
          </button>
        </div>
      )}

      {/* Not in Tauri warning */}
      {!isTauri && (
        <div
          style={{
            padding: "10px 20px",
            background: "rgba(255,179,71,0.08)",
            borderBottom: "1px solid rgba(255,179,71,0.15)",
            color: "#ffb347",
            fontSize: 12,
            flexShrink: 0,
          }}
        >
          ⚠ Running in browser mode — downloads require the Tauri desktop app.
        </div>
      )}

      {/* Pack list */}
      <div style={{ flex: 1, overflowY: "auto", padding: "16px 20px" }}>
        {loading && !manifest && (
          <div
            style={{
              color: "#4a6a8a",
              fontSize: 13,
              textAlign: "center",
              paddingTop: 40,
            }}
          >
            Loading sample library…
          </div>
        )}

        {!loading && !manifest && !error && (
          <div
            style={{
              color: "#4a6a8a",
              fontSize: 13,
              textAlign: "center",
              paddingTop: 40,
            }}
          >
            No manifest loaded. Click Refresh to fetch from GitHub Releases.
          </div>
        )}

        {Object.entries(grouped).map(([category, packs]) => (
          <div key={category} style={{ marginBottom: 24 }}>
            <div
              style={{
                fontSize: 11,
                fontWeight: 700,
                color: "#4a6a8a",
                letterSpacing: "0.08em",
                textTransform: "uppercase",
                marginBottom: 10,
                display: "flex",
                alignItems: "center",
                gap: 6,
              }}
            >
              {CATEGORY_ICONS[category as PackCategory]}
              {CATEGORY_LABELS[category as PackCategory]}
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
              {packs.map((pack) => (
                <PackRow
                  key={pack.id}
                  pack={pack}
                  status={statuses[pack.id] ?? { type: "not_installed" }}
                  onDownload={handleDownload}
                  onUninstall={handleUninstall}
                  isTauri={isTauri}
                />
              ))}
            </div>
          </div>
        ))}
      </div>

      {/* Footer */}
      <div
        style={{
          padding: "10px 20px",
          borderTop: "1px solid #0f1e2e",
          fontSize: 10,
          color: "#2a3a4a",
          flexShrink: 0,
        }}
      >
        Samples hosted on GitHub Releases · CC0 / CC BY / Public Domain licenses
        · Attribution shown per pack
      </div>
    </div>
  );
}

// ── Pack row ──────────────────────────────────────────────────────────────────

interface PackRowProps {
  pack: SamplePack;
  status: PackStatus;
  onDownload: (id: string) => void;
  onUninstall: (id: string) => void;
  isTauri: boolean;
}

function PackRow({
  pack,
  status,
  onDownload,
  onUninstall,
  isTauri,
}: PackRowProps) {
  const isInstalled =
    status.type === "installed" || status.type === "update_available";
  const isDownloading = status.type === "downloading";
  const progress = isDownloading
    ? (status as { type: "downloading"; progress_pct: number }).progress_pct
    : 0;

  return (
    <div
      style={{
        background: isInstalled ? "rgba(0,229,160,0.04)" : "#0a1520",
        border: `1px solid ${isInstalled ? "rgba(0,229,160,0.15)" : "#1a2a3a"}`,
        borderRadius: 8,
        padding: "12px 14px",
        position: "relative",
        overflow: "hidden",
      }}
    >
      {/* Download progress bar */}
      {isDownloading && (
        <div
          style={{
            position: "absolute",
            bottom: 0,
            left: 0,
            height: 2,
            width: `${progress}%`,
            background: "linear-gradient(90deg, #38bdf8, #818cf8)",
            transition: "width 0.3s",
          }}
        />
      )}

      <div style={{ display: "flex", alignItems: "flex-start", gap: 12 }}>
        {/* Info */}
        <div style={{ flex: 1, minWidth: 0 }}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              marginBottom: 3,
            }}
          >
            <span style={{ fontSize: 13, fontWeight: 600, color: "#e0e8f0" }}>
              {pack.name}
            </span>
            <span
              style={{
                fontSize: 9,
                padding: "1px 5px",
                borderRadius: 3,
                background:
                  pack.quality === "lite"
                    ? "rgba(56,189,248,0.15)"
                    : "rgba(129,140,248,0.15)",
                color: pack.quality === "lite" ? "#38bdf8" : "#818cf8",
                fontWeight: 700,
                letterSpacing: "0.05em",
                textTransform: "uppercase",
              }}
            >
              {pack.quality}
            </span>
            {isInstalled && (
              <span
                style={{
                  fontSize: 9,
                  padding: "1px 5px",
                  borderRadius: 3,
                  background: "rgba(0,229,160,0.15)",
                  color: "#00e5a0",
                  fontWeight: 700,
                }}
              >
                ✓ Installed
              </span>
            )}
            {status.type === "update_available" && (
              <span
                style={{
                  fontSize: 9,
                  padding: "1px 5px",
                  borderRadius: 3,
                  background: "rgba(255,179,71,0.15)",
                  color: "#ffb347",
                  fontWeight: 700,
                }}
              >
                Update available
              </span>
            )}
          </div>
          <div
            style={{
              fontSize: 11,
              color: "#4a6a8a",
              marginBottom: 4,
              lineHeight: 1.4,
            }}
          >
            {pack.description}
          </div>
          <div
            style={{ display: "flex", gap: 12, fontSize: 10, color: "#2a3a4a" }}
          >
            <span>{formatBytes(pack.compressed_bytes)} download</span>
            <span>{formatBytes(pack.uncompressed_bytes)} installed</span>
            <span style={{ color: "#1e3a2a" }}>{pack.license}</span>
          </div>
        </div>

        {/* Actions */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 4,
            flexShrink: 0,
          }}
        >
          {isDownloading ? (
            <div
              style={{
                padding: "5px 12px",
                background: "rgba(56,189,248,0.1)",
                border: "1px solid rgba(56,189,248,0.2)",
                borderRadius: 6,
                color: "#38bdf8",
                fontSize: 11,
                fontWeight: 600,
                minWidth: 80,
                textAlign: "center",
              }}
            >
              {progress}%
            </div>
          ) : isInstalled ? (
            <button
              onClick={() => onUninstall(pack.id)}
              style={{
                padding: "5px 12px",
                background: "transparent",
                border: "1px solid #1a2a3a",
                borderRadius: 6,
                color: "#4a6a8a",
                fontSize: 11,
                cursor: "pointer",
                transition: "all 0.12s",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = "rgba(239,83,80,0.4)";
                e.currentTarget.style.color = "#ef5350";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = "#1a2a3a";
                e.currentTarget.style.color = "#4a6a8a";
              }}
            >
              Uninstall
            </button>
          ) : (
            <button
              onClick={() => (isTauri ? onDownload(pack.id) : undefined)}
              disabled={!isTauri}
              style={{
                padding: "5px 12px",
                background: isTauri ? "rgba(56,189,248,0.1)" : "transparent",
                border: `1px solid ${isTauri ? "rgba(56,189,248,0.3)" : "#1a2a3a"}`,
                borderRadius: 6,
                color: isTauri ? "#38bdf8" : "#2a3a4a",
                fontSize: 11,
                fontWeight: 600,
                cursor: isTauri ? "pointer" : "not-allowed",
                transition: "all 0.12s",
              }}
              onMouseEnter={(e) => {
                if (!isTauri) return;
                e.currentTarget.style.background = "rgba(56,189,248,0.18)";
                e.currentTarget.style.boxShadow =
                  "0 0 12px rgba(56,189,248,0.2)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = "rgba(56,189,248,0.1)";
                e.currentTarget.style.boxShadow = "none";
              }}
            >
              ↓ Download
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

// ── Style helpers ─────────────────────────────────────────────────────────────

function filterBtnStyle(active: boolean): React.CSSProperties {
  return {
    padding: "4px 10px",
    background: active ? "rgba(56,189,248,0.12)" : "transparent",
    border: `1px solid ${active ? "rgba(56,189,248,0.3)" : "#1a2a3a"}`,
    borderRadius: 5,
    color: active ? "#38bdf8" : "#4a6a8a",
    fontSize: 11,
    fontWeight: active ? 600 : 400,
    cursor: "pointer",
    transition: "all 0.12s",
    whiteSpace: "nowrap",
  };
}
