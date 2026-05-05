/**
 * PatchBrowser — browse and load community patches shared via GitHub Gists.
 *
 * Searches public Gists tagged with "aether-patch" in the description.
 * Shows patch name, author, date, and a one-click load button.
 */

import { useState, useCallback, useEffect } from "react";
import { useEngineStore } from "../studio/store/engineStore";

interface GistEntry {
  id: string;
  description: string;
  owner: string;
  avatar: string;
  created_at: string;
  url: string;
}

const GIST_SEARCH_URL = "https://api.github.com/gists/public";

async function fetchCommunityPatches(): Promise<GistEntry[]> {
  // GitHub public gists API — filter by description containing "aether-patch"
  const res = await fetch(`${GIST_SEARCH_URL}?per_page=30`, {
    headers: { Accept: "application/vnd.github.v3+json" },
  });
  if (!res.ok) throw new Error(`GitHub API ${res.status}`);
  const gists = (await res.json()) as Array<{
    id: string;
    description: string;
    owner: { login: string; avatar_url: string };
    created_at: string;
    html_url: string;
    files: Record<string, { filename: string; content?: string }>;
  }>;

  return gists
    .filter(
      (g) =>
        g.description?.toLowerCase().includes("aether") ||
        Object.keys(g.files).some(
          (f) => f.endsWith(".aether-patch") || f === "aether-patch.json",
        ),
    )
    .map((g) => ({
      id: g.id,
      description: g.description || "Untitled patch",
      owner: g.owner.login,
      avatar: g.owner.avatar_url,
      created_at: g.created_at,
      url: g.html_url,
    }));
}

async function loadGistPatch(gistId: string): Promise<object | null> {
  const res = await fetch(`https://api.github.com/gists/${gistId}`, {
    headers: { Accept: "application/vnd.github.v3+json" },
  });
  if (!res.ok) return null;
  const gist = (await res.json()) as {
    files: Record<string, { content: string }>;
  };
  const patchFile =
    gist.files["aether-patch.json"] ??
    Object.values(gist.files).find((f) => f.content?.includes('"nodes"'));
  if (!patchFile) return null;
  try {
    return JSON.parse(patchFile.content);
  } catch {
    return null;
  }
}

interface PatchBrowserProps {
  onClose: () => void;
}

export function PatchBrowser({ onClose }: PatchBrowserProps) {
  const [patches, setPatches] = useState<GistEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [loadingId, setLoadingId] = useState<string | null>(null);
  const [search, setSearch] = useState("");

  const loadPatch = useEngineStore((s) => s.loadPatch);

  useEffect(() => {
    fetchCommunityPatches()
      .then(setPatches)
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, []);

  const handleLoad = useCallback(
    async (gistId: string) => {
      setLoadingId(gistId);
      try {
        const patch = await loadGistPatch(gistId);
        if (patch) {
          loadPatch(patch);
          onClose();
        } else {
          setError("Could not parse patch from Gist");
        }
      } catch (e) {
        setError(String(e));
      } finally {
        setLoadingId(null);
      }
    },
    [loadPatch, onClose],
  );

  const filtered = patches.filter(
    (p) =>
      !search ||
      p.description.toLowerCase().includes(search.toLowerCase()) ||
      p.owner.toLowerCase().includes(search.toLowerCase()),
  );

  const bg = "#060c12";
  const border = "#0f1e2e";
  const text = "#e0e8f0";
  const dim = "#4a6a8a";
  const accent = "#38bdf8";
  const surface = "#0a1520";

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 2000,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "rgba(2,5,10,0.88)",
        backdropFilter: "blur(12px)",
      }}
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div
        style={{
          width: 560,
          maxHeight: "80vh",
          display: "flex",
          flexDirection: "column",
          background: bg,
          border: `1px solid ${border}`,
          borderRadius: 12,
          boxShadow: "0 24px 64px rgba(0,0,0,0.7)",
        }}
      >
        {/* Header */}
        <div
          style={{
            padding: "16px 20px",
            borderBottom: `1px solid ${border}`,
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            flexShrink: 0,
          }}
        >
          <div>
            <div style={{ fontSize: 15, fontWeight: 700, color: text }}>
              Community Patches
            </div>
            <div style={{ fontSize: 11, color: dim, marginTop: 2 }}>
              Public patches shared via GitHub Gists
            </div>
          </div>
          <button
            onClick={onClose}
            style={{
              background: "none",
              border: "none",
              color: dim,
              cursor: "pointer",
              fontSize: 18,
            }}
          >
            ✕
          </button>
        </div>

        {/* Search */}
        <div
          style={{
            padding: "12px 20px",
            borderBottom: `1px solid ${border}`,
            flexShrink: 0,
          }}
        >
          <input
            type="text"
            placeholder="Search patches..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            style={{
              width: "100%",
              background: surface,
              border: `1px solid ${border}`,
              borderRadius: 8,
              color: text,
              padding: "8px 12px",
              fontSize: 12,
              boxSizing: "border-box",
            }}
          />
        </div>

        {/* List */}
        <div style={{ flex: 1, overflowY: "auto", padding: "8px 0" }}>
          {loading && (
            <div
              style={{
                padding: 24,
                textAlign: "center",
                color: dim,
                fontSize: 12,
              }}
            >
              Loading patches...
            </div>
          )}
          {error && (
            <div
              style={{
                padding: 16,
                color: "#ef5350",
                fontSize: 12,
                textAlign: "center",
              }}
            >
              {error}
            </div>
          )}
          {!loading && !error && filtered.length === 0 && (
            <div
              style={{
                padding: 24,
                textAlign: "center",
                color: dim,
                fontSize: 12,
              }}
            >
              No patches found. Share yours with the Share button!
            </div>
          )}
          {filtered.map((patch) => (
            <div
              key={patch.id}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 12,
                padding: "10px 20px",
                borderBottom: `1px solid ${border}`,
                transition: "background 0.1s",
              }}
              onMouseEnter={(e) => (e.currentTarget.style.background = surface)}
              onMouseLeave={(e) =>
                (e.currentTarget.style.background = "transparent")
              }
            >
              <img
                src={patch.avatar}
                alt={patch.owner}
                style={{
                  width: 32,
                  height: 32,
                  borderRadius: "50%",
                  flexShrink: 0,
                }}
              />
              <div style={{ flex: 1, minWidth: 0 }}>
                <div
                  style={{
                    fontSize: 12,
                    color: text,
                    fontWeight: 500,
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {patch.description}
                </div>
                <div style={{ fontSize: 10, color: dim, marginTop: 2 }}>
                  {patch.owner} ·{" "}
                  {new Date(patch.created_at).toLocaleDateString()}
                </div>
              </div>
              <div style={{ display: "flex", gap: 6, flexShrink: 0 }}>
                <a
                  href={patch.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  style={{
                    fontSize: 10,
                    color: dim,
                    textDecoration: "none",
                    padding: "4px 8px",
                    border: `1px solid ${border}`,
                    borderRadius: 4,
                  }}
                >
                  View
                </a>
                <button
                  onClick={() => handleLoad(patch.id)}
                  disabled={loadingId === patch.id}
                  style={{
                    padding: "4px 12px",
                    background: accent,
                    border: "none",
                    borderRadius: 6,
                    color: "#000",
                    fontSize: 11,
                    fontWeight: 700,
                    cursor: "pointer",
                    opacity: loadingId === patch.id ? 0.5 : 1,
                  }}
                >
                  {loadingId === patch.id ? "..." : "Load"}
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
