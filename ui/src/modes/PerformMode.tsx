/**
 * Aether Studio — Perform Mode
 * Live performance clip launcher. Large, tactile, built for the stage.
 *
 * Clips are wired to the engine:
 * - Playing a clip sends inject_midi NoteOn to the assigned MIDI channel
 * - Stopping sends NoteOff
 * - Scene launch fires all clips in a row simultaneously
 * - Master volume updates a Gain node if one exists in the graph
 */

import { useState, useCallback, useEffect, useRef } from "react";
import { useEngineStore } from "../studio/store/engineStore";
import "./PerformMode.css";

interface Clip {
  id: string;
  name: string;
  color: string;
  isPlaying: boolean;
  hasContent: boolean;
  midiChannel: number; // which MIDI channel this clip uses
  midiNote: number; // root note for this clip
}

type Grid = (Clip | null)[][];

const TRACK_NAMES = [
  "Krar",
  "Kebero",
  "Masenqo",
  "Washint",
  "Bass",
  "Pad",
  "FX",
  "Master",
];
const TRACK_COLORS = [
  "var(--region-east-africa)",
  "var(--accent-error)",
  "var(--region-middle-east)",
  "var(--region-south-asia)",
  "var(--accent-primary)",
  "var(--region-electronic)",
  "var(--region-americas)",
  "var(--text-secondary)",
];
// Each track gets its own MIDI channel (1-8)
const TRACK_MIDI_CHANNELS = [1, 2, 3, 4, 5, 6, 7, 8];
// Root notes per track (C3, D3, E3, F3, G3, A3, B3, C4)
const TRACK_ROOT_NOTES = [48, 50, 52, 53, 55, 57, 59, 60];

function makeGrid(): Grid {
  return TRACK_NAMES.map((name, t) =>
    Array.from({ length: 8 }, (_, s) => {
      const hasContent = Math.random() > 0.55;
      if (!hasContent) return null;
      return {
        id: `${t}-${s}`,
        name: `${name} ${s + 1}`,
        color: TRACK_COLORS[t],
        isPlaying: t < 3 && s === 0,
        hasContent: true,
        midiChannel: TRACK_MIDI_CHANNELS[t],
        midiNote: TRACK_ROOT_NOTES[t] + s,
      };
    }),
  );
}

export function PerformMode() {
  const [grid, setGrid] = useState<Grid>(makeGrid);
  const [bpm, setBpm] = useState(120);
  const [masterVol, setMasterVol] = useState(85);
  const [isPlaying, setIsPlaying] = useState(true);
  const tapTimesRef = useRef<number[]>([]);

  const sendIntent = useEngineStore((s) => s.sendIntent);
  const nodes = useEngineStore((s) => s.nodes);

  // Find the first Gain node to use as master volume
  const gainNodeId = nodes.find((n) => n.data.nodeType === "Gain")?.id ?? null;

  // Send NoteOn for a clip
  const playClip = useCallback(
    (clip: Clip) => {
      sendIntent?.({
        type: "inject_midi",
        channel: clip.midiChannel,
        note: clip.midiNote,
        velocity: 100,
        is_note_on: true,
      });
    },
    [sendIntent],
  );

  // Send NoteOff for a clip
  const stopClip = useCallback(
    (clip: Clip) => {
      sendIntent?.({
        type: "inject_midi",
        channel: clip.midiChannel,
        note: clip.midiNote,
        velocity: 0,
        is_note_on: false,
      });
    },
    [sendIntent],
  );

  const toggleClip = useCallback(
    (trackIdx: number, sceneIdx: number) => {
      setGrid((prev) => {
        const next = prev.map((track) =>
          track.map((clip) => (clip ? { ...clip } : null)),
        );
        const clip = next[trackIdx][sceneIdx];
        if (!clip) return prev;

        const wasPlaying = prev[trackIdx][sceneIdx]?.isPlaying ?? false;

        // Stop other clips in same track
        next[trackIdx].forEach((c) => {
          if (c && c.isPlaying) {
            stopClip(c);
            c.isPlaying = false;
          }
        });

        if (!wasPlaying) {
          clip.isPlaying = true;
          playClip(clip);
        }
        return next;
      });
    },
    [playClip, stopClip],
  );

  const launchScene = useCallback(
    (sceneIdx: number) => {
      setGrid((prev) => {
        const next = prev.map((track) =>
          track.map((clip) => (clip ? { ...clip } : null)),
        );
        next.forEach((track, trackIdx) => {
          track.forEach((clip, s) => {
            if (!clip) return;
            if (clip.isPlaying && s !== sceneIdx) {
              stopClip(clip);
              clip.isPlaying = false;
            } else if (!clip.isPlaying && s === sceneIdx) {
              clip.isPlaying = true;
              playClip(clip);
            }
          });
          // Also start the clip at sceneIdx if it exists
          const target = track[sceneIdx];
          if (target && !target.isPlaying) {
            target.isPlaying = true;
            playClip(target);
          }
          void trackIdx;
        });
        return next;
      });
    },
    [playClip, stopClip],
  );

  const stopAll = useCallback(() => {
    setGrid((prev) => {
      const next = prev.map((track) =>
        track.map((clip) => (clip ? { ...clip } : null)),
      );
      next.forEach((track) => {
        track.forEach((clip) => {
          if (clip?.isPlaying) {
            stopClip(clip);
            clip.isPlaying = false;
          }
        });
      });
      return next;
    });
    setIsPlaying(false);
  }, [stopClip]);

  // Master volume → Gain node param
  useEffect(() => {
    if (!gainNodeId) return;
    const node = nodes.find((n) => n.id === gainNodeId);
    if (!node) return;
    sendIntent?.({
      type: "update_param",
      node_id: parseInt(gainNodeId, 10),
      generation: node.data.generation ?? 0,
      param_index: 0,
      value: masterVol / 100,
      ramp_ms: 20,
    });
  }, [masterVol, gainNodeId, nodes, sendIntent]);

  // Tap tempo
  const handleTapTempo = useCallback(() => {
    const now = performance.now();
    tapTimesRef.current.push(now);
    // Keep last 4 taps
    if (tapTimesRef.current.length > 4) tapTimesRef.current.shift();
    if (tapTimesRef.current.length >= 2) {
      const intervals = tapTimesRef.current
        .slice(1)
        .map((t, i) => t - tapTimesRef.current[i]);
      const avgInterval =
        intervals.reduce((a, b) => a + b, 0) / intervals.length;
      const newBpm = Math.round(60000 / avgInterval);
      if (newBpm >= 20 && newBpm <= 300) setBpm(newBpm);
    }
  }, []);

  // Keyboard shortcuts: Q-I = row 1, A-K = row 2, Z-M = row 3
  useEffect(() => {
    const ROW_KEYS = [
      ["q", "w", "e", "r", "t", "y", "u", "i"],
      ["a", "s", "d", "f", "g", "h", "j", "k"],
      ["z", "x", "c", "v", "b", "n", "m", ","],
    ];
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey || e.altKey) return;
      if (e.target instanceof HTMLInputElement) return;
      for (let row = 0; row < ROW_KEYS.length; row++) {
        const col = ROW_KEYS[row].indexOf(e.key.toLowerCase());
        if (col !== -1 && col < TRACK_NAMES.length) {
          toggleClip(col, row);
          return;
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleClip]);

  return (
    <div className="perform-mode">
      {/* Track headers */}
      <div className="pm-track-headers">
        <div className="pm-corner" />
        {TRACK_NAMES.map((name, i) => (
          <div
            key={i}
            className="pm-track-header"
            style={{ "--track-color": TRACK_COLORS[i] } as React.CSSProperties}
          >
            <div className="pm-track-color-bar" />
            <span className="pm-track-name">{name}</span>
            <span
              style={{ fontSize: 9, color: "#2a3a4a", fontFamily: "monospace" }}
            >
              CH{TRACK_MIDI_CHANNELS[i]}
            </span>
          </div>
        ))}
        <div className="pm-scene-header">Scene</div>
      </div>

      {/* Clip grid */}
      <div className="pm-grid-scroll">
        <div className="pm-grid">
          {Array.from({ length: 8 }).map((_, sceneIdx) => (
            <div key={sceneIdx} className="pm-scene-row">
              {grid.map((track, trackIdx) => {
                const clip = track[sceneIdx];
                return (
                  <div
                    key={trackIdx}
                    className={`pm-cell ${clip ? "has-clip" : "empty"} ${clip?.isPlaying ? "playing" : ""}`}
                    style={
                      {
                        "--clip-color": clip?.color || "transparent",
                      } as React.CSSProperties
                    }
                    onClick={() => clip && toggleClip(trackIdx, sceneIdx)}
                  >
                    {clip && (
                      <>
                        <div className="pm-clip-indicator">
                          {clip.isPlaying ? "▶" : "■"}
                        </div>
                        <div className="pm-clip-name">{clip.name}</div>
                        {clip.isPlaying && <div className="pm-clip-pulse" />}
                      </>
                    )}
                  </div>
                );
              })}
              <button
                className="pm-scene-launch"
                onClick={() => launchScene(sceneIdx)}
                title={`Launch Scene ${sceneIdx + 1}`}
              >
                <span className="pm-scene-num">{sceneIdx + 1}</span>
                <span className="pm-scene-icon">▶</span>
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* Master controls */}
      <div className="pm-master">
        <div className="pm-master-left">
          <div className="pm-transport">
            <button
              className={`pm-transport-btn play ${isPlaying ? "active" : ""}`}
              onClick={() => setIsPlaying(true)}
              title="Play"
            >
              ▶
            </button>
            <button
              className="pm-transport-btn stop"
              onClick={stopAll}
              title="Stop All"
            >
              ■
            </button>
            <button className="pm-transport-btn record" title="Record">
              ●
            </button>
          </div>

          <div className="pm-bpm-control">
            <label className="pm-control-label">BPM</label>
            <button
              className="pm-bpm-btn"
              onClick={() => setBpm((b) => Math.max(20, b - 1))}
            >
              −
            </button>
            <input
              type="number"
              className="pm-bpm-input"
              value={bpm}
              onChange={(e) => setBpm(Number(e.target.value))}
              min={20}
              max={300}
            />
            <button
              className="pm-bpm-btn"
              onClick={() => setBpm((b) => Math.min(300, b + 1))}
            >
              +
            </button>
            <button className="pm-tap-btn" onClick={handleTapTempo}>
              Tap
            </button>
          </div>
        </div>

        <div className="pm-master-right">
          <label className="pm-control-label">
            Master{" "}
            {gainNodeId ? (
              ""
            ) : (
              <span style={{ fontSize: 9, color: "#4a6a8a" }}>
                (add Gain node)
              </span>
            )}
          </label>
          <div className="pm-vol-fader-container">
            <input
              type="range"
              className="pm-vol-fader"
              min={0}
              max={100}
              value={masterVol}
              onChange={(e) => setMasterVol(Number(e.target.value))}
            />
            <span className="pm-vol-value">{masterVol}%</span>
          </div>
        </div>

        <div className="pm-hint">
          <kbd>Q–I</kbd> Row 1 · <kbd>A–K</kbd> Row 2 · <kbd>Z–M</kbd> Row 3
        </div>
      </div>
    </div>
  );
}
