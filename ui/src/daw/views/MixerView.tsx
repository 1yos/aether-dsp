/**
 * MixerView — channel strips wired to the audio graph.
 * Each track gets a channel strip with volume fader, pan, mute/solo, and sends.
 */

import { useDawStore } from "../store/dawStore";
import { useEngineStore } from "../../studio/store/engineStore";
import { useEffect } from "react";

export function MixerView() {
  const channels = useDawStore((s) => s.channels);
  const tracks = useDawStore((s) => s.tracks);
  const updateChannel = useDawStore((s) => s.updateChannel);
  const audioActive = useEngineStore((s) => s.audioActive);

  const border = "#0f1e2e";
  const dim = "#475569";
  const text = "#e0e8f0";

  // Sync channel volumes to engine Gain nodes when they change
  const nodes = useEngineStore((s) => s.nodes);
  const updateParam = useEngineStore((s) => s.updateParam);

  useEffect(() => {
    // For each channel, find a Gain node with matching name and update its volume
    channels.forEach((ch) => {
      const gainNode = nodes.find(
        (n) =>
          n.data.nodeType === "Gain" &&
          (n.data as { label?: string }).label === ch.name,
      );
      if (gainNode) {
        updateParam(
          gainNode.id,
          (gainNode.data.generation as number) ?? 0,
          0,
          ch.volume,
        );
      }
    });
  }, [channels, nodes, updateParam]);

  return (
    <div
      style={{
        height: "100%",
        background: "#060c12",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Header */}
      <div
        style={{
          height: 36,
          background: "#080e18",
          borderBottom: `1px solid ${border}`,
          display: "flex",
          alignItems: "center",
          padding: "0 16px",
          gap: 8,
          flexShrink: 0,
        }}
      >
        <span style={{ fontSize: 11, fontWeight: 600, color: text }}>
          Mixer
        </span>
        <span style={{ fontSize: 10, color: dim }}>
          {channels.length} channels
        </span>
      </div>

      {/* Channel strips */}
      <div
        style={{
          flex: 1,
          display: "flex",
          overflowX: "auto",
          overflowY: "hidden",
          padding: "12px 8px",
          gap: 4,
          alignItems: "flex-end",
        }}
      >
        {channels.map((ch) => {
          const track = tracks.find((t) => t.channelId === ch.id);
          void track; // reserved for future track-channel linking
          const isMaster = ch.id === "master";

          return (
            <div
              key={ch.id}
              style={{
                width: isMaster ? 72 : 60,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: 4,
                background: "#080e18",
                border: `1px solid ${border}`,
                borderRadius: 6,
                padding: "8px 6px",
                flexShrink: 0,
              }}
            >
              {/* Channel name */}
              <div
                style={{
                  fontSize: 9,
                  fontWeight: 600,
                  color: ch.color,
                  textAlign: "center",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                  width: "100%",
                  letterSpacing: "0.05em",
                }}
              >
                {ch.name}
              </div>

              {/* VU meter */}
              <div
                style={{
                  width: "100%",
                  height: 80,
                  background: "#0a1520",
                  borderRadius: 3,
                  position: "relative",
                  overflow: "hidden",
                  border: `1px solid ${border}`,
                }}
              >
                {/* Level bar */}
                <div
                  style={{
                    position: "absolute",
                    bottom: 0,
                    left: 2,
                    right: 2,
                    height: audioActive ? `${ch.volume * 100}%` : "0%",
                    background: `linear-gradient(to top, ${ch.color}, ${ch.color}80)`,
                    borderRadius: 2,
                    transition: "height 0.08s",
                  }}
                />
                {/* dB markers */}
                {[0, -6, -12, -18, -24].map((db) => (
                  <div
                    key={db}
                    style={{
                      position: "absolute",
                      right: 2,
                      bottom: `${((db + 60) / 60) * 100}%`,
                      fontSize: 6,
                      color: "#1e2d3a",
                      fontFamily: "monospace",
                      lineHeight: 1,
                    }}
                  >
                    {db}
                  </div>
                ))}
              </div>

              {/* Pan knob (simplified as slider) */}
              <input
                type="range"
                min={-1}
                max={1}
                step={0.01}
                value={ch.pan}
                onChange={(e) =>
                  updateChannel(ch.id, { pan: parseFloat(e.target.value) })
                }
                style={{ width: "100%", height: 12 }}
                title={`Pan: ${ch.pan === 0 ? "C" : ch.pan > 0 ? `R${Math.round(ch.pan * 100)}` : `L${Math.round(-ch.pan * 100)}`}`}
              />

              {/* Volume fader */}
              <div
                style={{
                  position: "relative",
                  height: 120,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.01}
                  value={ch.volume}
                  onChange={(e) =>
                    updateChannel(ch.id, { volume: parseFloat(e.target.value) })
                  }
                  style={{
                    writingMode: "vertical-lr",
                    direction: "rtl",
                    height: 110,
                    width: 20,
                    cursor: "pointer",
                  }}
                  title={`Volume: ${Math.round(ch.volume * 100)}%`}
                />
              </div>

              {/* Volume readout */}
              <div style={{ fontSize: 9, color: dim, fontFamily: "monospace" }}>
                {Math.round(ch.volume * 100)}%
              </div>

              {/* Mute / Solo */}
              <div style={{ display: "flex", gap: 3, width: "100%" }}>
                <button
                  onClick={() => updateChannel(ch.id, { muted: !ch.muted })}
                  style={{
                    flex: 1,
                    padding: "2px 0",
                    background: ch.muted
                      ? "rgba(239,83,80,0.2)"
                      : "transparent",
                    border: `1px solid ${ch.muted ? "rgba(239,83,80,0.4)" : border}`,
                    borderRadius: 3,
                    color: ch.muted ? "#ef5350" : dim,
                    fontSize: 8,
                    cursor: "pointer",
                    fontFamily: "var(--font-sans)",
                  }}
                >
                  M
                </button>
                <button
                  onClick={() => updateChannel(ch.id, { solo: !ch.solo })}
                  style={{
                    flex: 1,
                    padding: "2px 0",
                    background: ch.solo
                      ? "rgba(255,213,79,0.2)"
                      : "transparent",
                    border: `1px solid ${ch.solo ? "rgba(255,213,79,0.4)" : border}`,
                    borderRadius: 3,
                    color: ch.solo ? "#ffd54f" : dim,
                    fontSize: 8,
                    cursor: "pointer",
                    fontFamily: "var(--font-sans)",
                  }}
                >
                  S
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
