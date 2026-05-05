/**
 * NodeParamEditor — double-click to open a full parameter editor for a node.
 *
 * Shows all parameters with sliders, value display, and reset-to-default.
 * Also shows the node's MIDI channel assignment and output routing.
 */

import { useCallback, useEffect, useRef } from "react";
import { useEngineStore, NODE_DEFS } from "../store/engineStore";

interface NodeParamEditorProps {
  nodeId: string;
  nodeType: string;
  generation: number;
  onClose: () => void;
}

export function NodeParamEditor({
  nodeId,
  nodeType,
  generation,
  onClose,
}: NodeParamEditorProps) {
  const panelRef = useRef<HTMLDivElement>(null);
  const updateParam = useEngineStore((s) => s.updateParam);
  const setOutputNode = useEngineStore((s) => s.setOutputNode);
  const nodes = useEngineStore((s) => s.nodes);

  const node = nodes.find((n) => n.id === nodeId);
  const def = NODE_DEFS[nodeType];

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onClose]);

  const handleParamChange = useCallback(
    (paramIndex: number, value: number) => {
      updateParam(nodeId, generation, paramIndex, value);
    },
    [nodeId, generation, updateParam],
  );

  const bg = "#060c12";
  const border = "#0f1e2e";
  const text = "#e0e8f0";
  const dim = "#4a6a8a";
  const accent = "#38bdf8";
  const surface = "#0a1520";

  if (!node || !def) return null;

  const params = (node.data.params as number[]) ?? [];

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 2500,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "rgba(2,5,10,0.7)",
        backdropFilter: "blur(4px)",
      }}
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div
        ref={panelRef}
        style={{
          width: 400,
          maxHeight: "70vh",
          overflow: "auto",
          background: bg,
          border: `1px solid ${border}`,
          borderRadius: 12,
          boxShadow: "0 24px 64px rgba(0,0,0,0.7)",
        }}
      >
        {/* Header */}
        <div
          style={{
            padding: "14px 18px",
            borderBottom: `1px solid ${border}`,
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <div>
            <div style={{ fontSize: 14, fontWeight: 700, color: text }}>
              {nodeType.replace(/([A-Z])/g, " $1").trim()}
            </div>
            <div
              style={{
                fontSize: 10,
                color: dim,
                marginTop: 2,
                fontFamily: "monospace",
              }}
            >
              id:{nodeId} · gen:{generation}
            </div>
          </div>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <button
              onClick={() => setOutputNode(nodeId, generation)}
              style={{
                padding: "4px 10px",
                background: node.data.isOutput
                  ? "rgba(0,229,160,0.15)"
                  : surface,
                border: `1px solid ${node.data.isOutput ? "#00e5a0" : border}`,
                borderRadius: 6,
                color: node.data.isOutput ? "#00e5a0" : dim,
                fontSize: 10,
                cursor: "pointer",
                fontWeight: node.data.isOutput ? 700 : 400,
              }}
            >
              {node.data.isOutput ? "◉ Output" : "Set Output"}
            </button>
            <button
              onClick={onClose}
              style={{
                background: "none",
                border: "none",
                color: dim,
                cursor: "pointer",
                fontSize: 16,
              }}
            >
              ✕
            </button>
          </div>
        </div>

        {/* Parameters */}
        <div
          style={{
            padding: 18,
            display: "flex",
            flexDirection: "column",
            gap: 16,
          }}
        >
          {def.params.length === 0 && (
            <div
              style={{
                fontSize: 12,
                color: dim,
                textAlign: "center",
                padding: "12px 0",
              }}
            >
              No parameters
            </div>
          )}

          {def.params.map((paramDef, i) => {
            const value = params[i] ?? paramDef.default;
            const pct =
              ((value - paramDef.min) / (paramDef.max - paramDef.min)) * 100;

            return (
              <div key={paramDef.name}>
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    marginBottom: 6,
                  }}
                >
                  <label style={{ fontSize: 11, color: text, fontWeight: 500 }}>
                    {paramDef.name}
                  </label>
                  <div
                    style={{ display: "flex", alignItems: "center", gap: 6 }}
                  >
                    <span
                      style={{
                        fontSize: 11,
                        color: accent,
                        fontFamily: "monospace",
                        minWidth: 52,
                        textAlign: "right",
                      }}
                    >
                      {value.toFixed(value % 1 === 0 ? 0 : 3)}
                    </span>
                    <button
                      onClick={() => handleParamChange(i, paramDef.default)}
                      style={{
                        background: "none",
                        border: `1px solid ${border}`,
                        borderRadius: 4,
                        color: dim,
                        fontSize: 9,
                        cursor: "pointer",
                        padding: "1px 5px",
                      }}
                      title="Reset to default"
                    >
                      ↺
                    </button>
                  </div>
                </div>
                <div style={{ position: "relative" }}>
                  <div
                    style={{
                      position: "absolute",
                      left: 0,
                      top: "50%",
                      transform: "translateY(-50%)",
                      width: `${pct}%`,
                      height: 4,
                      background: accent,
                      borderRadius: 2,
                      pointerEvents: "none",
                      zIndex: 1,
                    }}
                  />
                  <input
                    type="range"
                    min={paramDef.min}
                    max={paramDef.max}
                    step={(paramDef.max - paramDef.min) / 1000}
                    value={value}
                    onChange={(e) =>
                      handleParamChange(i, parseFloat(e.target.value))
                    }
                    style={{ width: "100%", position: "relative", zIndex: 2 }}
                  />
                </div>
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    marginTop: 2,
                  }}
                >
                  <span
                    style={{ fontSize: 9, color: dim, fontFamily: "monospace" }}
                  >
                    {paramDef.min}
                  </span>
                  <span
                    style={{ fontSize: 9, color: dim, fontFamily: "monospace" }}
                  >
                    {paramDef.max}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
