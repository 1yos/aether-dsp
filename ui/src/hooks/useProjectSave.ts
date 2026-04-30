/**
 * useProjectSave — Ctrl+S / Ctrl+O project persistence.
 *
 * Saves the current graph as a .aether-project JSON file.
 * Loads a project file and sends it to the host via LoadPatch intent.
 */

import { useEffect, useCallback } from "react";
import { useEngineStore } from "../studio/store/engineStore";

const AUTOSAVE_KEY = "aether_autosave";
const AUTOSAVE_INTERVAL_MS = 2 * 60 * 1000; // 2 minutes

export function useProjectSave() {
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const nodes = useEngineStore((s) => s.nodes);
  const edges = useEngineStore((s) => s.edges);
  const outputNodeId = useEngineStore((s) => s.outputNodeId);

  // Build a PatchDef from current UI state
  const buildPatchDef = useCallback(() => {
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

    return {
      version: "1.0",
      saved_at: new Date().toISOString(),
      nodes: patchNodes,
      connections,
      output_node: outputNodeId ?? "",
    };
  }, [nodes, edges, outputNodeId]);

  // Save project to a file download
  const saveProject = useCallback(
    (filename?: string) => {
      const patch = buildPatchDef();
      const json = JSON.stringify(patch, null, 2);
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename ?? `aether-project-${Date.now()}.aether-project`;
      a.click();
      URL.revokeObjectURL(url);

      // Also save to localStorage as autosave
      try {
        localStorage.setItem(AUTOSAVE_KEY, json);
        localStorage.setItem(`${AUTOSAVE_KEY}_time`, new Date().toISOString());
      } catch {
        // localStorage full — ignore
      }
    },
    [buildPatchDef],
  );

  // Load project from a file
  const loadProject = useCallback(() => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".aether-project,.json";
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = (ev) => {
        try {
          const json = ev.target?.result as string;
          const patch = JSON.parse(json);

          // Send to host
          sendIntent?.({ type: "load_patch", patch });
        } catch (err) {
          console.error("Failed to load project:", err);
          alert(
            "Failed to load project file. Make sure it's a valid .aether-project file.",
          );
        }
      };
      reader.readAsText(file);
    };
    input.click();
  }, [sendIntent]);

  // Restore autosave on mount
  const restoreAutosave = useCallback(() => {
    try {
      const json = localStorage.getItem(AUTOSAVE_KEY);
      const time = localStorage.getItem(`${AUTOSAVE_KEY}_time`);
      if (!json || !sendIntent) return null;
      return { json, time };
    } catch {
      return null;
    }
  }, [sendIntent]);

  // Keyboard shortcuts: Ctrl+S = save, Ctrl+O = load
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!e.ctrlKey && !e.metaKey) return;
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;

      if (e.key === "s" || e.key === "S") {
        e.preventDefault();
        saveProject();
      } else if (e.key === "o" || e.key === "O") {
        e.preventDefault();
        loadProject();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [saveProject, loadProject]);

  // Autosave every 2 minutes
  useEffect(() => {
    const interval = setInterval(() => {
      if (nodes.length > 0) {
        try {
          const patch = buildPatchDef();
          localStorage.setItem(AUTOSAVE_KEY, JSON.stringify(patch));
          localStorage.setItem(
            `${AUTOSAVE_KEY}_time`,
            new Date().toISOString(),
          );
        } catch {
          // ignore
        }
      }
    }, AUTOSAVE_INTERVAL_MS);

    return () => clearInterval(interval);
  }, [nodes.length, buildPatchDef]);

  return { saveProject, loadProject, restoreAutosave };
}
