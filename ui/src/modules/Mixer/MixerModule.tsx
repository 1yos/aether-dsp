/**
 * Mixer Module — channel strip per node in the DSP graph.
 * Faders send UpdateParam intents to Gain nodes (param index 0).
 * Mute button sends set_mute intent.
 */
import React, { useState } from "react";
import { useEngineStore } from "../../studio/store/engineStore";

const THEME = {
  bg: "#060c12",
  strip: "#0d1a26",
  border: "#1a2a3a",
  text: "#e0e8f0",
  textDim: "#4a6a8a",
  accent: "#4fc3f7",
  muted: "#ef5350",
  faderTrack: "#0a1520",
  faderThumb: "#4fc3f7",
};

interface ChannelStripProps {
  nodeId: string;
  generation: number;
  label: string;
  nodeType: string;
  gainValue: number;
  isMuted: boolean;
  onFaderChange: (value: number) => void;
  onMuteToggle: () => void;
}

const ChannelStrip: React.FC<ChannelStripProps> = ({
  label,
  nodeType,
  gainValue,
  isMuted,
  onFaderChange,
  onMuteToggle,
}) => {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        width: 72,
        background: THEME.strip,
        border: `1px solid ${THEME.border}`,
        borderRadius: 6,
        padding: "8px 4px",
        gap: 8,
        flexShrink: 0,
      }}
    >
      {/* Node type badge */}
      <div
        style={{
          fontSize: 9,
          color: THEME.textDim,
          textTransform: "uppercase",
          letterSpacing: 1,
          textAlign: "center",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          width: "100%",
        }}
      >
        {nodeType}
      </div>

      {/* Label */}
      <div
        style={{
          fontSize: 11,
          color: THEME.text,
          textAlign: "center",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          width: "100%",
          fontFamily: "monospace",
        }}
      >
        {label}
      </div>

      {/* Volume fader — vertical slider */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          flex: 1,
          width: "100%",
        }}
      >
        <input
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={gainValue}
          onChange={(e) => onFaderChange(parseFloat(e.target.value))}
          style={{
            writingMode: "vertical-lr",
            direction: "rtl",
            height: 100,
            width: 24,
            cursor: "pointer",
            accentColor: THEME.faderThumb,
          }}
          title={`Volume: ${Math.round(gainValue * 100)}%`}
        />
        <div style={{ fontSize: 10, color: THEME.textDim, marginTop: 4 }}>
          {Math.round(gainValue * 100)}
        </div>
      </div>

      {/* Mute button */}
      <button
        onClick={onMuteToggle}
        style={{
          width: 40,
          height: 24,
          background: isMuted ? THEME.muted : THEME.faderTrack,
          border: `1px solid ${isMuted ? THEME.muted : THEME.border}`,
          borderRadius: 4,
          color: isMuted ? "#fff" : THEME.textDim,
          fontSize: 10,
          fontWeight: "bold",
          cursor: "pointer",
          letterSpacing: 1,
        }}
        title={isMuted ? "Unmute" : "Mute"}
      >
        M
      </button>
    </div>
  );
};

export const MixerModule: React.FC = () => {
  const nodes = useEngineStore((s) => s.nodes);
  const muted = useEngineStore((s) => s.muted);
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const updateParam = useEngineStore((s) => s.updateParam);

  // Local fader state — keyed by node id
  const [faderValues, setFaderValues] = useState<Record<string, number>>({});

  const handleFaderChange = (
    nodeId: string,
    generation: number,
    value: number,
  ) => {
    setFaderValues((prev) => ({ ...prev, [nodeId]: value }));
    // Only send UpdateParam for Gain nodes (param index 0 = gain)
    const node = nodes.find((n) => n.id === nodeId);
    if (node?.data?.nodeType === "Gain") {
      updateParam(nodeId, generation, 0, value);
    }
  };

  const handleMuteToggle = () => {
    sendIntent?.({ type: "set_mute", muted: !muted });
  };

  if (nodes.length === 0) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          color: THEME.textDim,
          fontFamily: "monospace",
          fontSize: 13,
          background: THEME.bg,
        }}
      >
        No nodes in graph
      </div>
    );
  }

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: THEME.bg,
        fontFamily: "monospace",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "6px 12px",
          borderBottom: `1px solid ${THEME.border}`,
          fontSize: 11,
          color: THEME.textDim,
          letterSpacing: 2,
          textTransform: "uppercase",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <span>Mixer</span>
        <span style={{ color: muted ? THEME.muted : THEME.accent }}>
          {muted ? "MUTED" : "LIVE"}
        </span>
      </div>

      {/* Channel strips */}
      <div
        style={{
          display: "flex",
          flexDirection: "row",
          gap: 8,
          padding: 12,
          overflowX: "auto",
          flex: 1,
          alignItems: "flex-start",
        }}
      >
        {nodes.map((node) => {
          const nodeId = node.id;
          const generation = node.data?.generation ?? 0;
          const nodeType = node.data?.nodeType ?? "Unknown";
          const label = `${nodeType}-${nodeId}`;

          // Use local fader value or fall back to node's current gain param
          const defaultGain =
            nodeType === "Gain" && node.data?.params?.[0] != null
              ? node.data.params[0]
              : 0.8;
          const gainValue = faderValues[nodeId] ?? defaultGain;

          return (
            <ChannelStrip
              key={nodeId}
              nodeId={nodeId}
              generation={generation}
              label={label}
              nodeType={nodeType}
              gainValue={gainValue}
              isMuted={muted}
              onFaderChange={(v) => handleFaderChange(nodeId, generation, v)}
              onMuteToggle={handleMuteToggle}
            />
          );
        })}
      </div>
    </div>
  );
};

export default MixerModule;
