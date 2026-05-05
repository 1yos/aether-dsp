/**
 * PropertiesPanel — bottom panel showing selected track/clip properties.
 */

import { useDawStore } from "../store/dawStore";
import { SCALE_SYSTEMS } from "../../modes/scale-systems";
import { RHYTHMIC_SYSTEMS } from "../../modes/rhythmic-systems";

interface PropertiesPanelProps {
  height: number;
}

export function PropertiesPanel({ height }: PropertiesPanelProps) {
  const selectedTrackId = useDawStore((s) => s.selectedTrackId);
  const selectedClipId = useDawStore((s) => s.selectedClipId);
  const tracks = useDawStore((s) => s.tracks);
  const updateTrack = useDawStore((s) => s.updateTrack);
  const scaleId = useDawStore((s) => s.scaleId);
  const rhythmId = useDawStore((s) => s.rhythmId);
  const setScaleId = useDawStore((s) => s.setScaleId);
  const setRhythmId = useDawStore((s) => s.setRhythmId);

  const track = tracks.find((t) => t.id === selectedTrackId);

  const border = "#0f1e2e";
  const dim = "#475569";
  const text = "#e0e8f0";
  const accent = "#4db8ff";

  return (
    <div
      style={{
        height,
        background: "#080e18",
        borderTop: `1px solid ${border}`,
        display: "flex",
        overflow: "hidden",
        flexShrink: 0,
      }}
    >
      {/* Track properties */}
      <div
        style={{
          width: 280,
          borderRight: `1px solid ${border}`,
          padding: "10px 14px",
          display: "flex",
          flexDirection: "column",
          gap: 8,
          overflow: "hidden",
        }}
      >
        <div
          style={{
            fontSize: 10,
            color: dim,
            letterSpacing: "0.1em",
            textTransform: "uppercase",
            fontWeight: 600,
          }}
        >
          {track ? "Track" : "No Selection"}
        </div>

        {track && (
          <>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <div
                style={{
                  width: 10,
                  height: 10,
                  borderRadius: 2,
                  background: track.color,
                  flexShrink: 0,
                }}
              />
              <input
                value={track.name}
                onChange={(e) =>
                  updateTrack(track.id, { name: e.target.value })
                }
                style={{
                  flex: 1,
                  background: "transparent",
                  border: "none",
                  borderBottom: `1px solid ${border}`,
                  color: text,
                  fontSize: 12,
                  fontFamily: "var(--font-sans)",
                  fontWeight: 600,
                  outline: "none",
                  padding: "2px 0",
                }}
              />
            </div>

            {/* Volume */}
            <div>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  marginBottom: 3,
                }}
              >
                <span style={{ fontSize: 10, color: dim }}>Volume</span>
                <span
                  style={{
                    fontSize: 10,
                    color: accent,
                    fontFamily: "monospace",
                  }}
                >
                  {Math.round(track.volume * 100)}%
                </span>
              </div>
              <input
                type="range"
                min={0}
                max={1}
                step={0.01}
                value={track.volume}
                onChange={(e) =>
                  updateTrack(track.id, { volume: parseFloat(e.target.value) })
                }
                style={{ width: "100%" }}
              />
            </div>

            {/* Pan */}
            <div>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  marginBottom: 3,
                }}
              >
                <span style={{ fontSize: 10, color: dim }}>Pan</span>
                <span
                  style={{
                    fontSize: 10,
                    color: accent,
                    fontFamily: "monospace",
                  }}
                >
                  {track.pan === 0
                    ? "C"
                    : track.pan > 0
                      ? `R${Math.round(track.pan * 100)}`
                      : `L${Math.round(-track.pan * 100)}`}
                </span>
              </div>
              <input
                type="range"
                min={-1}
                max={1}
                step={0.01}
                value={track.pan}
                onChange={(e) =>
                  updateTrack(track.id, { pan: parseFloat(e.target.value) })
                }
                style={{ width: "100%" }}
              />
            </div>

            {/* Mute / Solo */}
            <div style={{ display: "flex", gap: 6 }}>
              <button
                onClick={() => updateTrack(track.id, { muted: !track.muted })}
                style={{
                  flex: 1,
                  padding: "4px",
                  background: track.muted
                    ? "rgba(239,83,80,0.15)"
                    : "transparent",
                  border: `1px solid ${track.muted ? "rgba(239,83,80,0.4)" : border}`,
                  borderRadius: 4,
                  color: track.muted ? "#ef5350" : dim,
                  fontSize: 10,
                  cursor: "pointer",
                  fontFamily: "var(--font-sans)",
                }}
              >
                {track.muted ? "MUTED" : "MUTE"}
              </button>
              <button
                onClick={() => updateTrack(track.id, { solo: !track.solo })}
                style={{
                  flex: 1,
                  padding: "4px",
                  background: track.solo
                    ? "rgba(255,213,79,0.15)"
                    : "transparent",
                  border: `1px solid ${track.solo ? "rgba(255,213,79,0.4)" : border}`,
                  borderRadius: 4,
                  color: track.solo ? "#ffd54f" : dim,
                  fontSize: 10,
                  cursor: "pointer",
                  fontFamily: "var(--font-sans)",
                }}
              >
                {track.solo ? "SOLO" : "SOLO"}
              </button>
            </div>
          </>
        )}
      </div>

      {/* Scale / Rhythm */}
      <div
        style={{
          width: 280,
          borderRight: `1px solid ${border}`,
          padding: "10px 14px",
          display: "flex",
          flexDirection: "column",
          gap: 8,
        }}
      >
        <div
          style={{
            fontSize: 10,
            color: dim,
            letterSpacing: "0.1em",
            textTransform: "uppercase",
            fontWeight: 600,
          }}
        >
          Tuning & Rhythm
        </div>

        <div>
          <label
            style={{
              fontSize: 10,
              color: dim,
              display: "block",
              marginBottom: 4,
            }}
          >
            Scale System
          </label>
          <select
            value={scaleId}
            onChange={(e) => setScaleId(e.target.value)}
            style={{
              width: "100%",
              background: "#0a1520",
              border: `1px solid ${border}`,
              borderRadius: 4,
              color: text,
              padding: "4px 6px",
              fontSize: 11,
              fontFamily: "var(--font-sans)",
              cursor: "pointer",
            }}
          >
            {SCALE_SYSTEMS.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label
            style={{
              fontSize: 10,
              color: dim,
              display: "block",
              marginBottom: 4,
            }}
          >
            Rhythmic System
          </label>
          <select
            value={rhythmId}
            onChange={(e) => setRhythmId(e.target.value)}
            style={{
              width: "100%",
              background: "#0a1520",
              border: `1px solid ${border}`,
              borderRadius: 4,
              color: text,
              padding: "4px 6px",
              fontSize: 11,
              fontFamily: "var(--font-sans)",
              cursor: "pointer",
            }}
          >
            {RHYTHMIC_SYSTEMS.map((r) => (
              <option key={r.id} value={r.id}>
                {r.name}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Clip info */}
      <div style={{ flex: 1, padding: "10px 14px" }}>
        <div
          style={{
            fontSize: 10,
            color: dim,
            letterSpacing: "0.1em",
            textTransform: "uppercase",
            fontWeight: 600,
            marginBottom: 8,
          }}
        >
          {selectedClipId ? "Clip" : "Clip Properties"}
        </div>
        {!selectedClipId && (
          <div style={{ fontSize: 11, color: "#1e2d3d" }}>
            Select a clip to see its properties
          </div>
        )}
      </div>
    </div>
  );
}
