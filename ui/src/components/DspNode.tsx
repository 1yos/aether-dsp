import { memo } from "react";
import { Handle, Position, NodeProps } from "reactflow";
import { useGraphStore } from "../hooks/useGraphStore";
import { ParamDef, NODE_INPUT_COUNT } from "../types/graph";

interface DspNodeData {
  label: string;
  nodeType: string;
  params: number[];
  paramDefs: ParamDef[];
  isOutput: boolean;
}

const DspNodeComponent = ({ id, data, selected }: NodeProps<DspNodeData>) => {
  const updateParam = useGraphStore((s) => s.updateParam);
  const setSelected = useGraphStore((s) => s.setSelectedNode);
  const deleteSelected = useGraphStore((s) => s.deleteSelectedNode);
  const setOutputNode = useGraphStore((s) => s.setOutputNode);
  const outputNodeId = useGraphStore((s) => s.outputNodeId);
  const audioActive = useGraphStore((s) => s.audioActive);

  const inputCount = NODE_INPUT_COUNT[data.nodeType] ?? 1;
  const isOutput = outputNodeId === id;

  return (
    <div
      onClick={() => setSelected(id)}
      style={{
        background: selected ? "#1e3a5f" : "#111827",
        border: `1px solid ${isOutput ? "#66bb6a" : selected ? "#4fc3f7" : "#1e3a5f"}`,
        borderRadius: 10,
        padding: "10px 14px 12px",
        minWidth: 190,
        color: "#e0e0e0",
        fontFamily: "monospace",
        fontSize: 12,
        cursor: "pointer",
        boxShadow: isOutput
          ? `0 0 12px ${audioActive ? "#66bb6a" : "#1a3a1a"}`
          : selected
            ? "0 0 8px #4fc3f7"
            : "0 2px 8px rgba(0,0,0,0.4)",
        position: "relative",
        transition: "box-shadow 0.15s",
      }}
    >
      {/* Input handles — only as many as the node actually uses */}
      {Array.from({ length: inputCount }).map((_, i) => (
        <Handle
          key={`in-${i}`}
          type="target"
          position={Position.Left}
          id={`input-${i}`}
          style={{
            top:
              inputCount === 1
                ? "50%"
                : `${15 + (i * 70) / Math.max(inputCount - 1, 1)}%`,
            background: "#4fc3f7",
            width: 10,
            height: 10,
            border: "2px solid #0a0a14",
          }}
        />
      ))}

      {/* Header row */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 8,
        }}
      >
        <div
          style={{
            fontWeight: "bold",
            color: isOutput ? "#66bb6a" : "#4fc3f7",
            fontSize: 13,
          }}
        >
          {data.label}
          {isOutput && (
            <span
              style={{
                marginLeft: 6,
                fontSize: 10,
                color: audioActive ? "#66bb6a" : "#2a5a2a",
                transition: "color 0.15s",
              }}
            >
              ◉ OUT
            </span>
          )}
        </div>

        {/* Action buttons — only visible when selected */}
        {selected && (
          <div style={{ display: "flex", gap: 4 }}>
            {!isOutput && (
              <button
                title="Set as audio output"
                onClick={(e) => {
                  e.stopPropagation();
                  setOutputNode(id);
                }}
                style={btnStyle("#1a3a1a", "#66bb6a")}
              >
                ◉
              </button>
            )}
            <button
              title="Delete node (Del)"
              onClick={(e) => {
                e.stopPropagation();
                deleteSelected();
              }}
              style={btnStyle("#3a1a1a", "#ef5350")}
            >
              ✕
            </button>
          </div>
        )}
      </div>

      {/* Params */}
      {data.paramDefs.map((def, i) => (
        <div key={i} style={{ marginBottom: 7 }}>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              marginBottom: 2,
            }}
          >
            <span style={{ color: "#9e9e9e" }}>{def.name}</span>
            <span
              style={{ color: "#e0e0e0", fontVariantNumeric: "tabular-nums" }}
            >
              {formatValue(data.params[i] ?? def.default, def)}
            </span>
          </div>
          <input
            type="range"
            min={def.min}
            max={def.max}
            step={(def.max - def.min) / 1000}
            value={data.params[i] ?? def.default}
            onChange={(e) => updateParam(id, i, parseFloat(e.target.value))}
            style={{
              width: "100%",
              accentColor: "#4fc3f7",
              cursor: "ew-resize",
            }}
          />
        </div>
      ))}

      {/* Output handle */}
      <Handle
        type="source"
        position={Position.Right}
        id="output"
        style={{
          background: "#ff7043",
          width: 10,
          height: 10,
          border: "2px solid #0a0a14",
        }}
      />
    </div>
  );
};

function btnStyle(bg: string, color: string) {
  return {
    background: bg,
    color,
    border: `1px solid ${color}`,
    borderRadius: 3,
    width: 20,
    height: 20,
    cursor: "pointer",
    fontSize: 10,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    padding: 0,
    lineHeight: 1,
  } as React.CSSProperties;
}

function formatValue(v: number, def: ParamDef): string {
  const range = def.max - def.min;
  if (range >= 1000) return v.toFixed(0);
  if (range >= 10) return v.toFixed(2);
  return v.toFixed(3);
}

export default memo(DspNodeComponent);
