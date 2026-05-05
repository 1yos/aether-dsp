/**
 * Aether Studio — Modulation Matrix
 *
 * Connects modulation sources (LFO, envelope) to any parameter on any node.
 * Each connection sends set_modulation / remove_modulation intents to the host.
 * The host stores connections and applies them each audio buffer.
 */

import { useState, useCallback } from "react";
import { useEngineStore } from "../studio/store/engineStore";
import "./ModulationMatrix.css";

interface ModSource {
  id: string;
  label: string;
  nodeId: string | null;
  nodeGen: number;
  color: string;
}

interface ModDestination {
  id: string;
  label: string;
  nodeId: string;
  nodeGen: number;
  paramIndex: number;
  paramName: string;
}

interface ModConnection {
  id: string;
  sourceId: string;
  destId: string;
  amount: number; // -1.0 to 1.0
}

export function ModulationMatrix({ onClose }: { onClose: () => void }) {
  const nodes = useEngineStore((s) => s.nodes);
  const sendIntent = useEngineStore((s) => s.sendIntent);

  const [connections, setConnections] = useState<ModConnection[]>([]);
  const [draggingSource, setDraggingSource] = useState<string | null>(null);

  // Build sources from LFO and envelope nodes in the graph
  const sources: ModSource[] = [
    ...nodes
      .filter((n) => n.data.nodeType === "Lfo")
      .map((n) => ({
        id: `lfo-${n.id}`,
        label: `LFO ${n.id}`,
        nodeId: n.id,
        nodeGen: n.data.generation ?? 0,
        color: "#a78bfa",
      })),
    ...nodes
      .filter((n) => n.data.nodeType === "AdsrEnvelope")
      .map((n) => ({
        id: `env-${n.id}`,
        label: `Env ${n.id}`,
        nodeId: n.id,
        nodeGen: n.data.generation ?? 0,
        color: "#ffb347",
      })),
  ];

  // Build destinations from all nodes with parameters
  const destinations: ModDestination[] = nodes.flatMap((n) =>
    ((n.data.paramDefs as Array<{ name: string }>) || []).map((def, i) => ({
      id: `${n.id}-${i}`,
      label: `${n.data.nodeType} · ${def.name}`,
      nodeId: n.id,
      nodeGen: n.data.generation ?? 0,
      paramIndex: i,
      paramName: def.name,
    })),
  );

  const addConnection = useCallback(
    (sourceId: string, destId: string) => {
      const existing = connections.find(
        (c) => c.sourceId === sourceId && c.destId === destId,
      );
      if (existing) return;

      const src = sources.find((s) => s.id === sourceId);
      const dst = destinations.find((d) => d.id === destId);
      if (!src || !dst || !src.nodeId) return;

      const amount = 0.5;
      setConnections((prev) => [
        ...prev,
        { id: `${sourceId}->${destId}`, sourceId, destId, amount },
      ]);

      // Send to host
      sendIntent?.({
        type: "set_modulation",
        src_node_id: parseInt(src.nodeId, 10),
        src_gen: src.nodeGen,
        dst_node_id: parseInt(dst.nodeId, 10),
        dst_gen: dst.nodeGen,
        dst_param_index: dst.paramIndex,
        amount,
      });
    },
    [connections, sources, destinations, sendIntent],
  );

  const removeConnection = useCallback(
    (id: string) => {
      const conn = connections.find((c) => c.id === id);
      if (!conn) return;

      const src = sources.find((s) => s.id === conn.sourceId);
      const dst = destinations.find((d) => d.id === conn.destId);

      setConnections((prev) => prev.filter((c) => c.id !== id));

      if (src?.nodeId && dst) {
        sendIntent?.({
          type: "remove_modulation",
          src_node_id: parseInt(src.nodeId, 10),
          dst_node_id: parseInt(dst.nodeId, 10),
          dst_param_index: dst.paramIndex,
        });
      }
    },
    [connections, sources, destinations, sendIntent],
  );

  const updateAmount = useCallback(
    (id: string, amount: number) => {
      const conn = connections.find((c) => c.id === id);
      if (!conn) return;

      setConnections((prev) =>
        prev.map((c) => (c.id === id ? { ...c, amount } : c)),
      );

      const src = sources.find((s) => s.id === conn.sourceId);
      const dst = destinations.find((d) => d.id === conn.destId);
      if (src?.nodeId && dst) {
        sendIntent?.({
          type: "set_modulation",
          src_node_id: parseInt(src.nodeId, 10),
          src_gen: src.nodeGen,
          dst_node_id: parseInt(dst.nodeId, 10),
          dst_gen: dst.nodeGen,
          dst_param_index: dst.paramIndex,
          amount,
        });
      }
    },
    [connections, sources, destinations, sendIntent],
  );

  const getConnectionColor = (sourceId: string) =>
    sources.find((s) => s.id === sourceId)?.color || "#a78bfa";

  return (
    <div className="mod-matrix-overlay">
      <div className="mod-matrix-backdrop" onClick={onClose} />
      <div className="mod-matrix-panel animate-scale-in">
        <div className="mod-matrix-header">
          <div>
            <h2 className="mod-matrix-title">Modulation Matrix</h2>
            <p className="mod-matrix-subtitle">
              Click a source, then click a destination to connect
            </p>
          </div>
          <button className="mod-matrix-close" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="mod-matrix-body">
          {/* Sources column */}
          <div className="mod-col">
            <div className="mod-col-header">Sources</div>
            <div className="mod-col-items">
              {sources.length === 0 && (
                <div className="mod-empty">
                  Add LFO or Envelope nodes to the canvas
                </div>
              )}
              {sources.map((src) => (
                <div
                  key={src.id}
                  className={`mod-source-item ${draggingSource === src.id ? "dragging" : ""}`}
                  style={{ "--mod-color": src.color } as React.CSSProperties}
                  onClick={() =>
                    setDraggingSource(draggingSource === src.id ? null : src.id)
                  }
                >
                  <span className="mod-source-dot" />
                  <span className="mod-source-label">{src.label}</span>
                  {draggingSource === src.id && (
                    <span className="mod-source-hint">→ click destination</span>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Connections column */}
          <div className="mod-connections-col">
            <div className="mod-col-header">Active ({connections.length})</div>
            <div className="mod-connections-list">
              {connections.length === 0 && (
                <div className="mod-empty">No connections yet</div>
              )}
              {connections.map((conn) => {
                const src = sources.find((s) => s.id === conn.sourceId);
                const dst = destinations.find((d) => d.id === conn.destId);
                if (!src || !dst) return null;
                const connColor = getConnectionColor(conn.sourceId);
                return (
                  <div
                    key={conn.id}
                    className="mod-connection"
                    style={{ "--mod-color": connColor } as React.CSSProperties}
                  >
                    <div className="mod-conn-labels">
                      <span className="mod-conn-src">{src.label}</span>
                      <span className="mod-conn-arrow">→</span>
                      <span className="mod-conn-dst">{dst.label}</span>
                    </div>
                    <div className="mod-conn-controls">
                      <input
                        type="range"
                        min="-100"
                        max="100"
                        value={Math.round(conn.amount * 100)}
                        onChange={(e) =>
                          updateAmount(conn.id, Number(e.target.value) / 100)
                        }
                        className="mod-amount-slider"
                      />
                      <span className="mod-amount-value">
                        {conn.amount >= 0 ? "+" : ""}
                        {Math.round(conn.amount * 100)}%
                      </span>
                      <button
                        className="mod-remove-btn"
                        onClick={() => removeConnection(conn.id)}
                        title="Remove"
                      >
                        ✕
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Destinations column */}
          <div className="mod-col">
            <div className="mod-col-header">Destinations</div>
            <div className="mod-col-items">
              {destinations.length === 0 && (
                <div className="mod-empty">
                  Add nodes with parameters to the canvas
                </div>
              )}
              {destinations.map((dst) => {
                const isConnected = connections.some(
                  (c) => c.destId === dst.id,
                );
                return (
                  <div
                    key={dst.id}
                    className={`mod-dest-item ${isConnected ? "connected" : ""} ${draggingSource ? "selectable" : ""}`}
                    onClick={() => {
                      if (draggingSource) {
                        addConnection(draggingSource, dst.id);
                        setDraggingSource(null);
                      }
                    }}
                  >
                    <span className="mod-dest-label">{dst.label}</span>
                    {isConnected && <span className="mod-dest-dot" />}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
