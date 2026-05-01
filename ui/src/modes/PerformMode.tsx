/**
 * Aether Studio — Perform Mode
 * Live performance clip launcher. Large, tactile, built for the stage.
 */

import { useState, useCallback } from "react";
import "./PerformMode.css";

interface Clip {
  id: string;
  name: string;
  color: string;
  isPlaying: boolean;
  hasContent: boolean;
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
      };
    }),
  );
}

export function PerformMode() {
  const [grid, setGrid] = useState<Grid>(makeGrid);
  const [bpm, setBpm] = useState(120);
  const [masterVol, setMasterVol] = useState(85);
  const [isPlaying, setIsPlaying] = useState(true);

  const toggleClip = useCallback((trackIdx: number, sceneIdx: number) => {
    setGrid((prev) => {
      const next = prev.map((track) =>
        track.map((clip) => (clip ? { ...clip } : null)),
      );
      const clip = next[trackIdx][sceneIdx];
      if (clip) {
        // Stop other clips in same track
        next[trackIdx].forEach((c) => {
          if (c) c.isPlaying = false;
        });
        clip.isPlaying = !prev[trackIdx][sceneIdx]?.isPlaying;
      }
      return next;
    });
  }, []);

  const launchScene = useCallback((sceneIdx: number) => {
    setGrid((prev) => {
      const next = prev.map((track) =>
        track.map((clip) => (clip ? { ...clip } : null)),
      );
      next.forEach((track) => {
        track.forEach((clip, s) => {
          if (clip) clip.isPlaying = s === sceneIdx;
        });
      });
      return next;
    });
  }, []);

  const stopAll = useCallback(() => {
    setGrid((prev) =>
      prev.map((track) =>
        track.map((clip) => (clip ? { ...clip, isPlaying: false } : null)),
      ),
    );
    setIsPlaying(false);
  }, []);

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
              {/* Scene launch */}
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
            <button className="pm-tap-btn">Tap</button>
          </div>
        </div>

        <div className="pm-master-right">
          <label className="pm-control-label">Master</label>
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
