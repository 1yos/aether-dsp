import { memo, useState } from "react";
import { Handle, Position, NodeProps } from "reactflow";
import { useEngineStore, ParamDef } from "../store/engineStore";
import { ScopeCanvas } from "./ScopeCanvas";

interface StudioNodeData {
  nodeType: string;
  generation: number;
  params: number[];
  paramDefs: ParamDef[];
  inputCount: number;
  color: string;
  isOutput: boolean;
}

const WAVEFORM_NAMES = ["Sine", "Saw", "Square", "Tri"];
const FILTER_MODES = ["LP", "HP", "BP"];

function formatParam(value: number, def: ParamDef): string {
  if (def.name === "Waveform")
    return WAVEFORM_NAMES[Math.round(value)] ?? String(Math.round(value));
  if (def.name === "Mode")
    return FILTER_MODES[Math.round(value)] ?? String(Math.round(value));
  if (def.name === "Frequency")
    return value >= 1000
      ? `${(value / 1000).toFixed(1)}k`
      : `${value.toFixed(0)}`;
  if (def.name === "Attack" || def.name === "Decay" || def.name === "Release")
    return `${(value * 1000).toFixed(0)}ms`;
  if (def.name === "Sustain") return `${(value * 100).toFixed(0)}%`;
  const range = def.max - def.min;
  if (range >= 100) return value.toFixed(0);
  if (range >= 1) return value.toFixed(2);
  return value.toFixed(3);
}

const NODE_ICONS: Record<string, string> = {
  Oscillator: "∿",
  StateVariableFilter: "⌇",
  AdsrEnvelope: "⋀",
  DelayLine: "⟳",
  Gain: "▲",
  Mixer: "⊕",
  SamplerNode: "◈",
  TimbreTransferNode: "⟡",
  ScopeNode: "〜",
};

const StudioNode = ({ id, data, selected }: NodeProps<StudioNodeData>) => {
  const updateParam = useEngineStore((s) => s.updateParam);
  const setOutputNode = useEngineStore((s) => s.setOutputNode);
  const removeNode = useEngineStore((s) => s.removeNode);
  const setSelected = useEngineStore((s) => s.setSelectedNode);
  const outputNodeId = useEngineStore((s) => s.outputNodeId);
  const audioActive = useEngineStore((s) => s.audioActive);
  const scopeFrame = useEngineStore((s) => s.scopeFrame);
  const [activeParam, setActiveParam] = useState<number | null>(null);

  const isOutput = outputNodeId === id;
  const accent = data.color;

  return (
    <div
      onClick={() => setSelected(id)}
      style={{
        background: "linear-gradient(160deg, #0d1b2a 0%, #091422 100%)",
        border: `1px solid ${isOutput ? "#00e5a0" : selected ? accent : "#1a2a3a"}`,
        borderRadius: 10,
        minWidth: 210,
        color: "#e2e8f0",
        fontFamily: "'Inter', system-ui, sans-serif",
        fontSize: 12,
        cursor: "pointer",
        boxShadow: isOutput
          ? `0 0 0 1px #00e5a020, 0 0 20px #00e5a015, 0 4px 20px rgba(0,0,0,0.5)`
          : selected
            ? `0 0 0 1px ${accent}20, 0 4px 20px rgba(0,0,0,0.5)`
            : "0 2px 12px rgba(0,0,0,0.4)",
        transition: "box-shadow 0.15s, border-color 0.15s",
        position: "relative",
        overflow: "visible",
      }}
    >
      {/* Input handles */}
      {Array.from({ length: data.inputCount }).map((_, i) => (
        <Handle
          key={`in-${i}`}
          type="target"
          position={Position.Left}
          id={`input-${i}`}
          style={{
            top:
              data.inputCount === 1
                ? "50%"
                : `${25 + (i * 50) / Math.max(data.inputCount - 1, 1)}%`,
            background: "#0d1b2a",
            width: 12,
            height: 12,
            border: `2px solid ${accent}`,
            borderRadius: "50%",
            left: -6,
          }}
        />
      ))}

      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "10px 12px 8px",
          borderBottom: `1px solid ${accent}18`,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          {/* Icon badge */}
          <div
            style={{
              width: 24,
              height: 24,
              borderRadius: 5,
              background: `${accent}18`,
              border: `1px solid ${accent}30`,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 13,
              color: accent,
              fontFamily: "monospace",
            }}
          >
            {NODE_ICONS[data.nodeType] ?? "◦"}
          </div>
          <div>
            <div
              style={{
                fontSize: 11,
                fontWeight: 600,
                color: "#e2e8f0",
                letterSpacing: "0.01em",
                lineHeight: 1.2,
              }}
            >
              {data.nodeType.replace(/([A-Z])/g, " $1").trim()}
            </div>
            {isOutput && (
              <div
                style={{
                  fontSize: 9,
                  color: audioActive ? "#00e5a0" : "#00e5a060",
                  fontWeight: 600,
                  letterSpacing: "0.08em",
                  transition: "color 0.1s",
                }}
              >
                {audioActive ? "● OUTPUT" : "○ OUTPUT"}
              </div>
            )}
          </div>
        </div>

        {/* Action buttons — show on select */}
        {selected && (
          <div style={{ display: "flex", gap: 4 }}>
            {!isOutput && (
              <NodeBtn
                title="Set as audio output"
                color="#00e5a0"
                onClick={(e) => {
                  e.stopPropagation();
                  setOutputNode(id, data.generation);
                }}
              >
                ◉
              </NodeBtn>
            )}
            <NodeBtn
              title="Delete node"
              color="#ef5350"
              onClick={(e) => {
                e.stopPropagation();
                removeNode(id);
              }}
            >
              ✕
            </NodeBtn>
          </div>
        )}
      </div>

      {/* Params */}
      {data.paramDefs.length > 0 && (
        <div style={{ padding: "8px 12px 10px" }}>
          {data.paramDefs.map((def, i) => {
            const value = data.params[i] ?? def.default;
            const pct = ((value - def.min) / (def.max - def.min)) * 100;
            const isActive = activeParam === i;

            return (
              <div
                key={i}
                style={{ marginBottom: i < data.paramDefs.length - 1 ? 8 : 0 }}
              >
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    marginBottom: 4,
                  }}
                >
                  <span
                    style={{
                      fontSize: 10,
                      color: isActive ? "#94a3b8" : "#475569",
                      fontWeight: 500,
                      letterSpacing: "0.02em",
                      transition: "color 0.1s",
                    }}
                  >
                    {def.name.toUpperCase()}
                  </span>
                  <span
                    style={{
                      fontSize: 10,
                      color: isActive ? accent : "#64748b",
                      fontFamily: "monospace",
                      fontVariantNumeric: "tabular-nums",
                      transition: "color 0.1s",
                    }}
                  >
                    {formatParam(value, def)}
                  </span>
                </div>

                {/* Track */}
                <div
                  style={{
                    position: "relative",
                    height: 3,
                    background: "#0f1e2e",
                    borderRadius: 2,
                  }}
                >
                  {/* Fill */}
                  <div
                    style={{
                      position: "absolute",
                      left: 0,
                      top: 0,
                      height: "100%",
                      borderRadius: 2,
                      background: isActive
                        ? accent
                        : `linear-gradient(90deg, ${accent}60, ${accent}90)`,
                      width: `${pct}%`,
                      transition: "background 0.1s",
                    }}
                  />
                  {/* Thumb dot */}
                  <div
                    style={{
                      position: "absolute",
                      top: "50%",
                      left: `${pct}%`,
                      transform: "translate(-50%, -50%)",
                      width: isActive ? 8 : 6,
                      height: isActive ? 8 : 6,
                      borderRadius: "50%",
                      background: isActive ? accent : `${accent}cc`,
                      boxShadow: isActive ? `0 0 6px ${accent}` : "none",
                      transition: "all 0.1s",
                      pointerEvents: "none",
                    }}
                  />
                  {/* Invisible range input */}
                  <input
                    type="range"
                    min={def.min}
                    max={def.max}
                    step={(def.max - def.min) / 1000}
                    value={value}
                    onMouseDown={() => setActiveParam(i)}
                    onMouseUp={() => setActiveParam(null)}
                    onChange={(e) =>
                      updateParam(
                        id,
                        data.generation,
                        i,
                        parseFloat(e.target.value),
                      )
                    }
                    style={{
                      position: "absolute",
                      inset: "-4px 0",
                      width: "100%",
                      height: "calc(100% + 8px)",
                      opacity: 0,
                      cursor: "ew-resize",
                      margin: 0,
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Scope canvas for ScopeNode */}
      {data.nodeType === "ScopeNode" && (
        <div style={{ padding: "8px 12px 10px" }}>
          <ScopeCanvas samples={scopeFrame} />
        </div>
      )}

      {/* Output handle */}
      <Handle
        type="source"
        position={Position.Right}
        id="output"
        style={{
          background: "#0d1b2a",
          width: 12,
          height: 12,
          border: "2px solid #ff7043",
          borderRadius: "50%",
          right: -6,
        }}
      />
    </div>
  );
};

function NodeBtn({
  children,
  color,
  onClick,
  title,
}: {
  children: React.ReactNode;
  color: string;
  onClick: (e: React.MouseEvent) => void;
  title: string;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      title={title}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: 22,
        height: 22,
        background: hovered ? `${color}20` : "transparent",
        border: `1px solid ${hovered ? color : `${color}40`}`,
        borderRadius: 4,
        color: hovered ? color : `${color}80`,
        cursor: "pointer",
        fontSize: 10,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: 0,
        transition: "all 0.1s",
      }}
    >
      {children}
    </button>
  );
}

export default memo(StudioNode);
