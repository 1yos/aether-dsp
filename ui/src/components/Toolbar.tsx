import { useRef } from "react";
import { useGraphStore, WsStatus } from "../hooks/useGraphStore";
import { NODE_PARAM_DEFS, GraphSnapshot } from "../types/graph";

const NODE_TYPES = Object.keys(NODE_PARAM_DEFS);

const STATUS_COLOR: Record<WsStatus, string> = {
  disconnected: "#ef5350",
  connecting: "#ffd54f",
  connected: "#66bb6a",
  error: "#ff7043",
};

const STATUS_LABEL: Record<WsStatus, string> = {
  disconnected: "Disconnected",
  connecting: "Connecting…",
  connected: "Connected",
  error: "Error",
};

export function Toolbar() {
  const addDspNode = useGraphStore((s) => s.addDspNode);
  const deleteSelected = useGraphStore((s) => s.deleteSelectedNode);
  const selectedNode = useGraphStore((s) => s.selectedNode);
  const wsStatus = useGraphStore((s) => s.wsStatus);
  const audioActive = useGraphStore((s) => s.audioActive);
  const loadSnapshot = useGraphStore((s) => s.loadSnapshot);
  const fileRef = useRef<HTMLInputElement>(null);

  const handleLoadPatch = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      try {
        const json = JSON.parse(ev.target?.result as string);
        // Support both raw GraphSnapshot and aether.json manifest format
        if (json.nodes && json.connections !== undefined) {
          // aether.json manifest — convert to snapshot format
          const snap: GraphSnapshot = {
            nodes: json.nodes.map(
              (
                n: {
                  id: string;
                  type: string;
                  params?: Record<string, number>;
                },
                i: number,
              ) => ({
                id: i,
                generation: 0,
                node_type: n.type,
                params: Object.values(n.params ?? {}),
              }),
            ),
            edges: json.connections.map(
              (c: { from: string; to: string; slot?: number }) => {
                const srcIdx = json.nodes.findIndex(
                  (n: { id: string }) => n.id === c.from,
                );
                const dstIdx = json.nodes.findIndex(
                  (n: { id: string }) => n.id === c.to,
                );
                return { src_id: srcIdx, dst_id: dstIdx, slot: c.slot ?? 0 };
              },
            ),
          };
          loadSnapshot(snap);
        } else if (json.nodes && Array.isArray(json.nodes)) {
          // Raw GraphSnapshot
          loadSnapshot(json as GraphSnapshot);
        }
      } catch {
        alert(
          "Could not parse patch file. Expected aether.json or snapshot JSON.",
        );
      }
    };
    reader.readAsText(file);
    e.target.value = "";
  };

  return (
    <>
      {/* Main toolbar */}
      <div
        style={{
          position: "absolute",
          top: 16,
          left: 16,
          zIndex: 10,
          display: "flex",
          alignItems: "center",
          gap: 6,
          background: "#0d0d1a",
          padding: "8px 12px",
          borderRadius: 8,
          border: "1px solid #1e3a5f",
          boxShadow: "0 2px 12px rgba(0,0,0,0.5)",
          flexWrap: "wrap",
          maxWidth: "calc(100vw - 32px)",
        }}
      >
        {/* Logo */}
        <span
          style={{
            color: "#4fc3f7",
            fontFamily: "monospace",
            fontSize: 14,
            fontWeight: "bold",
            marginRight: 4,
          }}
        >
          Aether<span style={{ color: "#ff7043" }}>DSP</span>
        </span>

        <div
          style={{
            width: 1,
            height: 20,
            background: "#1e3a5f",
            margin: "0 4px",
          }}
        />

        {/* Add node buttons */}
        {NODE_TYPES.map((type) => (
          <button
            key={type}
            onClick={() => addDspNode(type)}
            style={addBtnStyle}
          >
            + {type}
          </button>
        ))}

        <div
          style={{
            width: 1,
            height: 20,
            background: "#1e3a5f",
            margin: "0 4px",
          }}
        />

        {/* Load patch */}
        <button
          onClick={() => fileRef.current?.click()}
          style={actionBtnStyle("#1a2a3a", "#4fc3f7")}
        >
          📂 Load Patch
        </button>
        <input
          ref={fileRef}
          type="file"
          accept=".json"
          style={{ display: "none" }}
          onChange={handleLoadPatch}
        />

        {/* Delete selected */}
        <button
          onClick={deleteSelected}
          disabled={!selectedNode}
          title="Delete selected node (Del)"
          style={actionBtnStyle(
            selectedNode ? "#3a1a1a" : "#111",
            selectedNode ? "#ef5350" : "#444",
          )}
        >
          🗑 Delete
        </button>
      </div>

      {/* Status bar — bottom right */}
      <div
        style={{
          position: "absolute",
          bottom: 16,
          right: 16,
          zIndex: 10,
          display: "flex",
          alignItems: "center",
          gap: 10,
          background: "#0d0d1a",
          padding: "6px 12px",
          borderRadius: 6,
          border: "1px solid #1e3a5f",
          fontFamily: "monospace",
          fontSize: 12,
        }}
      >
        {/* Audio activity pulse */}
        <div
          title="Audio activity"
          style={{
            width: 10,
            height: 10,
            borderRadius: "50%",
            background: audioActive ? "#66bb6a" : "#1a3a1a",
            boxShadow: audioActive ? "0 0 6px #66bb6a" : "none",
            transition: "background 0.1s, box-shadow 0.1s",
          }}
        />
        <span style={{ color: "#888" }}>Audio</span>

        <div style={{ width: 1, height: 14, background: "#1e3a5f" }} />

        {/* WS status dot */}
        <div
          style={{
            width: 8,
            height: 8,
            borderRadius: "50%",
            background: STATUS_COLOR[wsStatus],
            boxShadow:
              wsStatus === "connected"
                ? `0 0 5px ${STATUS_COLOR[wsStatus]}`
                : "none",
          }}
        />
        <span style={{ color: STATUS_COLOR[wsStatus] }}>
          {STATUS_LABEL[wsStatus]}
        </span>

        <div style={{ width: 1, height: 14, background: "#1e3a5f" }} />

        <span style={{ color: "#555" }}>ws://127.0.0.1:9001</span>
      </div>

      {/* Keyboard shortcut hint */}
      {selectedNode && (
        <div
          style={{
            position: "absolute",
            bottom: 52,
            right: 16,
            zIndex: 10,
            background: "#0d0d1a",
            border: "1px solid #1e3a5f",
            borderRadius: 4,
            padding: "4px 10px",
            fontFamily: "monospace",
            fontSize: 11,
            color: "#555",
          }}
        >
          Del — delete node &nbsp;|&nbsp; ◉ — set as output
        </div>
      )}
    </>
  );
}

const addBtnStyle: React.CSSProperties = {
  background: "#111827",
  color: "#9e9e9e",
  border: "1px solid #1e3a5f",
  borderRadius: 4,
  padding: "4px 9px",
  cursor: "pointer",
  fontFamily: "monospace",
  fontSize: 11,
  transition: "border-color 0.15s, color 0.15s",
};

function actionBtnStyle(bg: string, color: string): React.CSSProperties {
  return {
    background: bg,
    color,
    border: `1px solid ${color}`,
    borderRadius: 4,
    padding: "4px 10px",
    cursor: "pointer",
    fontFamily: "monospace",
    fontSize: 11,
  };
}
