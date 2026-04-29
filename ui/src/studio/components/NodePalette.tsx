import { useState } from "react";
import { useEngineStore, NODE_DEFS } from "../store/engineStore";

const NODE_ICONS: Record<string, string> = {
  Oscillator: "∿",
  StateVariableFilter: "⌇",
  AdsrEnvelope: "⋀",
  DelayLine: "⟳",
  Gain: "▲",
  Mixer: "⊕",
  SamplerNode: "◈",
  TimbreTransferNode: "⟡",
};

const NODE_DESCRIPTIONS: Record<string, string> = {
  Oscillator: "Sine · Saw · Square · Triangle",
  StateVariableFilter: "LP · HP · BP filter",
  AdsrEnvelope: "Attack Decay Sustain Release",
  DelayLine: "Echo with feedback",
  Gain: "Amplifier / attenuator",
  Mixer: "4-input summing bus",
  SamplerNode: "MIDI-driven sampler",
  TimbreTransferNode: "Spectral timbre shaper",
};

export function NodePalette() {
  const addNode = useEngineStore((s) => s.addNode);
  const wsStatus = useEngineStore((s) => s.wsStatus);
  const [hoveredType, setHoveredType] = useState<string | null>(null);
  const isConnected = wsStatus === "connected";

  return (
    <div
      style={{
        width: 200,
        background: "#080e18",
        borderRight: "1px solid #0f1e2e",
        display: "flex",
        flexDirection: "column",
        flexShrink: 0,
        overflow: "hidden",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "14px 16px 10px",
          borderBottom: "1px solid #0f1e2e",
        }}
      >
        <div
          style={{
            fontSize: 10,
            fontFamily: "'Inter', system-ui, sans-serif",
            fontWeight: 600,
            letterSpacing: "0.12em",
            textTransform: "uppercase",
            color: "#334155",
          }}
        >
          Node Library
        </div>
        {!isConnected && (
          <div
            style={{
              marginTop: 6,
              padding: "4px 8px",
              background: "rgba(239,83,80,0.08)",
              border: "1px solid rgba(239,83,80,0.2)",
              borderRadius: 4,
              fontSize: 10,
              color: "#ef5350",
              fontFamily: "'Inter', system-ui, sans-serif",
            }}
          >
            ⚠ Start aether-host to add nodes
          </div>
        )}
      </div>

      {/* Node list */}
      <div style={{ flex: 1, overflowY: "auto", padding: "8px 0" }}>
        {Object.values(NODE_DEFS).map((def) => {
          const isHovered = hoveredType === def.type;
          return (
            <button
              key={def.type}
              onClick={() => addNode(def.type)}
              onMouseEnter={() => setHoveredType(def.type)}
              onMouseLeave={() => setHoveredType(null)}
              title={NODE_DESCRIPTIONS[def.type]}
              style={{
                width: "100%",
                background: isHovered
                  ? "rgba(255,255,255,0.04)"
                  : "transparent",
                border: "none",
                borderLeft: `2px solid ${isHovered ? def.color : "transparent"}`,
                padding: "9px 16px",
                cursor: "pointer",
                display: "flex",
                alignItems: "center",
                gap: 10,
                textAlign: "left",
                transition: "all 0.1s",
              }}
            >
              {/* Icon */}
              <div
                style={{
                  width: 28,
                  height: 28,
                  borderRadius: 6,
                  background: isHovered ? `${def.color}18` : "#0c1420",
                  border: `1px solid ${isHovered ? `${def.color}40` : "#1a2a3a"}`,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: 14,
                  color: isHovered ? def.color : "#334155",
                  flexShrink: 0,
                  transition: "all 0.1s",
                  fontFamily: "monospace",
                }}
              >
                {NODE_ICONS[def.type] ?? "◦"}
              </div>

              {/* Label */}
              <div>
                <div
                  style={{
                    fontSize: 12,
                    fontFamily: "'Inter', system-ui, sans-serif",
                    fontWeight: 500,
                    color: isHovered ? "#e2e8f0" : "#64748b",
                    transition: "color 0.1s",
                    lineHeight: 1.2,
                  }}
                >
                  {def.type.replace(/([A-Z])/g, " $1").trim()}
                </div>
                {isHovered && (
                  <div
                    style={{
                      fontSize: 10,
                      color: "#475569",
                      fontFamily: "'Inter', system-ui, sans-serif",
                      marginTop: 2,
                      lineHeight: 1.3,
                    }}
                  >
                    {NODE_DESCRIPTIONS[def.type]}
                  </div>
                )}
              </div>
            </button>
          );
        })}
      </div>

      {/* Footer hint */}
      <div
        style={{
          padding: "10px 16px",
          borderTop: "1px solid #0f1e2e",
          fontSize: 10,
          color: "#1e2d3d",
          fontFamily: "'Inter', system-ui, sans-serif",
          lineHeight: 1.5,
        }}
      >
        Click to add · Drag to connect
      </div>
    </div>
  );
}
