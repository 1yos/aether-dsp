/**
 * usePatchShare — share patches via GitHub Gist.
 *
 * Creates a public GitHub Gist containing the current patch JSON.
 * Returns a short URL that anyone can use to open the patch.
 *
 * The Gist API is public and doesn't require authentication for
 * anonymous gists (rate-limited to 60 requests/hour per IP).
 *
 * URL format: https://aether.studio/patch/{gist_id}
 * (falls back to the raw Gist URL if the custom domain isn't set up)
 */

import { useCallback, useState } from "react";
import { useEngineStore } from "../studio/store/engineStore";

const GIST_API = "https://api.github.com/gists";
const PATCH_FILENAME = "aether-patch.json";

export interface ShareState {
  status: "idle" | "sharing" | "shared" | "error";
  url: string | null;
  gistId: string | null;
  error: string | null;
}

export function usePatchShare() {
  const nodes = useEngineStore((s) => s.nodes);
  const edges = useEngineStore((s) => s.edges);
  const outputNodeId = useEngineStore((s) => s.outputNodeId);
  const loadPatch = useEngineStore((s) => s.loadPatch);

  const [shareState, setShareState] = useState<ShareState>({
    status: "idle",
    url: null,
    gistId: null,
    error: null,
  });

  /** Build the patch JSON from current graph state */
  const buildPatchJson = useCallback((): string => {
    const patchNodes = nodes.map((n) => ({
      id: n.id,
      type: n.data.nodeType as string,
      params: Object.fromEntries(
        ((n.data.paramDefs as Array<{ name: string }>) ?? []).map((def, i) => [
          def.name,
          (n.data.params as number[])?.[i] ?? 0,
        ]),
      ),
    }));

    const connections = edges.map((e) => ({
      from: e.source,
      to: e.target,
      slot: parseInt(e.targetHandle?.replace("input-", "") ?? "0", 10),
    }));

    const patch = {
      version: "1.0",
      created_at: new Date().toISOString(),
      app: "Aether Studio",
      nodes: patchNodes,
      connections,
      output_node: outputNodeId ?? "",
    };

    return JSON.stringify(patch, null, 2);
  }, [nodes, edges, outputNodeId]);

  /** Upload patch as a GitHub Gist and return the share URL */
  const sharePatch = useCallback(async (): Promise<string | null> => {
    if (nodes.length === 0) {
      setShareState({
        status: "error",
        url: null,
        gistId: null,
        error: "Nothing to share — add some nodes first.",
      });
      return null;
    }

    setShareState({ status: "sharing", url: null, gistId: null, error: null });

    try {
      const patchJson = buildPatchJson();

      const response = await fetch(GIST_API, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/vnd.github.v3+json",
        },
        body: JSON.stringify({
          description: `Aether Studio patch — ${new Date().toLocaleDateString()}`,
          public: true,
          files: {
            [PATCH_FILENAME]: {
              content: patchJson,
            },
          },
        }),
      });

      if (!response.ok) {
        const err = await response.text();
        throw new Error(`GitHub API error ${response.status}: ${err}`);
      }

      const gist = (await response.json()) as { id: string; html_url: string };
      const gistId = gist.id;

      // Build the share URL — use a custom format that the app can parse
      const shareUrl = `${window.location.origin}?patch=${gistId}`;

      setShareState({ status: "shared", url: shareUrl, gistId, error: null });
      return shareUrl;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setShareState({ status: "error", url: null, gistId: null, error: msg });
      return null;
    }
  }, [nodes.length, buildPatchJson]);

  /** Load a patch from a GitHub Gist ID */
  const loadFromGist = useCallback(
    async (gistId: string): Promise<boolean> => {
      try {
        const response = await fetch(`${GIST_API}/${gistId}`, {
          headers: { Accept: "application/vnd.github.v3+json" },
        });

        if (!response.ok) throw new Error(`Gist not found: ${gistId}`);

        const gist = (await response.json()) as {
          files: Record<string, { content: string }>;
        };

        // Find the patch file
        const patchFile =
          gist.files[PATCH_FILENAME] ??
          Object.values(gist.files).find((f) => f.content.includes('"nodes"'));

        if (!patchFile) throw new Error("No patch file found in Gist");

        const patch = JSON.parse(patchFile.content);
        loadPatch(patch);
        return true;
      } catch (e) {
        console.error("Failed to load patch from Gist:", e);
        return false;
      }
    },
    [loadPatch],
  );

  /** Check URL params on load and auto-load a shared patch */
  const checkUrlForSharedPatch = useCallback(async (): Promise<boolean> => {
    const params = new URLSearchParams(window.location.search);
    const gistId = params.get("patch");
    if (!gistId) return false;

    // Remove the param from URL without reload
    const url = new URL(window.location.href);
    url.searchParams.delete("patch");
    window.history.replaceState({}, "", url.toString());

    return loadFromGist(gistId);
  }, [loadFromGist]);

  const resetShareState = useCallback(() => {
    setShareState({ status: "idle", url: null, gistId: null, error: null });
  }, []);

  return {
    shareState,
    sharePatch,
    loadFromGist,
    checkUrlForSharedPatch,
    resetShareState,
  };
}
