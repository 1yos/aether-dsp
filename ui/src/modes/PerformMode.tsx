/**
 * Aether Studio v2.0 - Perform Mode
 * Live performance with clip launcher
 */

import { useState } from "react";
import "./PerformMode.css";

interface Clip {
  id: string;
  name: string;
  color: string;
  isPlaying: boolean;
}

type ClipGrid = (Clip | null)[][];

export function PerformMode() {
  const [bpm, setBpm] = useState(120);
  const [masterVolume, setMasterVolume] = useState(80);

  const [clips, setClips] = useState<ClipGrid>([
    [
      {
        id: "1-1",
        name: "Krar Intro",
        color: "var(--region-east-africa)",
        isPlaying: true,
      },
      null,
      {
        id: "1-3",
        name: "Krar Verse",
        color: "var(--region-east-africa)",
        isPlaying: false,
      },
      null,
      null,
      null,
      null,
      null,
    ],
    [
      null,
      {
        id: "2-2",
        name: "Kebero Beat",
        color: "var(--accent-error)",
        isPlaying: true,
      },
      null,
      null,
      null,
      null,
      null,
      null,
    ],
    [
      {
        id: "3-1",
        name: "Bass Line",
        color: "var(--accent-info)",
        isPlaying: false,
      },
      null,
      null,
      {
        id: "3-4",
        name: "Bass Drop",
        color: "var(--accent-info)",
        isPlaying: true,
      },
      null,
      null,
      null,
      null,
    ],
    [
      null,
      null,
      {
        id: "4-3",
        name: "Melody",
        color: "var(--accent-primary)",
        isPlaying: false,
      },
      null,
      null,
      null,
      null,
      null,
    ],
    [null, null, null, null, null, null, null, null],
    [null, null, null, null, null, null, null, null],
    [null, null, null, null, null, null, null, null],
    [null, null, null, null, null, null, null, null],
  ]);

  const toggleClip = (trackIndex: number, sceneIndex: number) => {
    setClips((prevClips) => {
      const newClips = prevClips.map((track) => [...track]);
      const clip = newClips[trackIndex][sceneIndex];
      if (clip) {
        clip.isPlaying = !clip.isPlaying;
      }
      return newClips;
    });
  };

  const launchScene = (sceneIndex: number) => {
    setClips((prevClips) => {
      const newClips = prevClips.map((track) => [...track]);
      newClips.forEach((track) => {
        if (track[sceneIndex]) {
          track[sceneIndex]!.isPlaying = true;
        }
      });
      return newClips;
    });
  };

  return (
    <div className="perform-mode">
      <div className="perform-content">
        {/* Clip Launcher Grid */}
        <div className="clip-launcher">
          <div className="clip-grid">
            {clips.map((track, trackIndex) => (
              <div key={trackIndex} className="clip-track">
                <div className="track-label">Track {trackIndex + 1}</div>
                {track.map((clip, sceneIndex) => (
                  <div
                    key={sceneIndex}
                    className={`clip-cell ${clip ? "has-clip" : "empty"} ${clip?.isPlaying ? "playing" : ""}`}
                    style={
                      { "--clip-color": clip?.color } as React.CSSProperties
                    }
                    onClick={() => clip && toggleClip(trackIndex, sceneIndex)}
                  >
                    {clip && (
                      <>
                        <div className="clip-indicator">
                          {clip.isPlaying ? "▶" : "■"}
                        </div>
                        <div className="clip-name">{clip.name}</div>
                      </>
                    )}
                  </div>
                ))}
              </div>
            ))}
          </div>

          {/* Scene Launch Buttons */}
          <div className="scene-launcher">
            <div className="scene-label">Scenes</div>
            {Array.from({ length: 8 }).map((_, i) => (
              <button
                key={i}
                className="scene-btn"
                onClick={() => launchScene(i)}
                title={`Launch Scene ${i + 1}`}
              >
                ▶
              </button>
            ))}
          </div>
        </div>

        {/* Master Controls */}
        <div className="master-controls">
          <div className="control-section">
            <label>BPM</label>
            <div className="bpm-control">
              <button onClick={() => setBpm((b) => Math.max(20, b - 1))}>
                −
              </button>
              <input
                type="number"
                value={bpm}
                onChange={(e) => setBpm(Number(e.target.value))}
                min="20"
                max="300"
              />
              <button onClick={() => setBpm((b) => Math.min(300, b + 1))}>
                +
              </button>
            </div>
            <button className="tap-tempo-btn">Tap Tempo</button>
          </div>

          <div className="control-section">
            <label>Master Volume</label>
            <div className="volume-control">
              <input
                type="range"
                min="0"
                max="100"
                value={masterVolume}
                onChange={(e) => setMasterVolume(Number(e.target.value))}
                className="volume-slider"
              />
              <div className="volume-value">{masterVolume}%</div>
            </div>
          </div>

          <div className="control-section">
            <label>Transport</label>
            <div className="transport-buttons">
              <button className="transport-btn play">▶</button>
              <button className="transport-btn stop">■</button>
              <button className="transport-btn record">●</button>
            </div>
          </div>
        </div>
      </div>

      {/* Keyboard Shortcuts Help */}
      <div className="shortcuts-hint">
        <span>💡 Tip: Use Q-I, A-K, Z-M keys to trigger clips</span>
      </div>
    </div>
  );
}
