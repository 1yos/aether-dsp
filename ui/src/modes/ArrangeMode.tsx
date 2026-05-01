/**
 * Aether Studio — Arrange Mode
 * Timeline + Piano Roll. Professional DAW layout.
 */

import { useState } from "react";
import "./ArrangeMode.css";

interface Note {
  id: string;
  pitch: number;
  start: number;
  duration: number;
  velocity: number;
}

const NOTE_NAMES = [
  "C",
  "C#",
  "D",
  "D#",
  "E",
  "F",
  "F#",
  "G",
  "G#",
  "A",
  "A#",
  "B",
];
const SCALE_NOTES = [0, 2, 3, 5, 7, 8, 10]; // Tizita

export function ArrangeMode() {
  const [notes] = useState<Note[]>([
    { id: "1", pitch: 60, start: 0, duration: 1, velocity: 90 },
    { id: "2", pitch: 62, start: 1, duration: 1, velocity: 80 },
    { id: "3", pitch: 64, start: 2, duration: 2, velocity: 100 },
    { id: "4", pitch: 67, start: 4, duration: 1, velocity: 85 },
    { id: "5", pitch: 60, start: 6, duration: 1, velocity: 75 },
    { id: "6", pitch: 64, start: 7, duration: 2, velocity: 95 },
  ]);
  const [scale, setScale] = useState("Tizita (Ethiopian)");
  const [showPianoRoll, setShowPianoRoll] = useState(true);

  const octaves = 3;
  const startOctave = 3;
  const totalNotes = octaves * 12;
  const totalBeats = 16;

  return (
    <div className="arrange-mode">
      {/* Piano Roll */}
      {showPianoRoll && (
        <div className="piano-roll">
          <div className="pr-header">
            <div className="pr-title">Piano Roll</div>
            <div className="pr-controls">
              <select
                className="pr-scale-select"
                value={scale}
                onChange={(e) => setScale(e.target.value)}
              >
                <option>Tizita (Ethiopian)</option>
                <option>Bati (Ethiopian)</option>
                <option>Anchihoye (Ethiopian)</option>
                <option>Arabic Maqam Rast</option>
                <option>Just Intonation</option>
                <option>Chromatic</option>
              </select>
              <button className="pr-tool-btn" title="Quantize">
                ⊞
              </button>
              <button className="pr-tool-btn" title="Humanize">
                ≈
              </button>
              <button
                className="pr-close-btn"
                onClick={() => setShowPianoRoll(false)}
              >
                ✕
              </button>
            </div>
          </div>

          <div className="pr-content">
            {/* Piano keys column */}
            <div className="pr-keys">
              {Array.from({ length: totalNotes }).map((_, i) => {
                const noteIdx = totalNotes - 1 - i;
                const pitchClass = noteIdx % 12;
                const octave = Math.floor(noteIdx / 12) + startOctave;
                const isBlack = [1, 3, 6, 8, 10].includes(pitchClass);
                const isInScale = SCALE_NOTES.includes(pitchClass);
                const noteName = NOTE_NAMES[pitchClass];
                return (
                  <div
                    key={i}
                    className={`pr-key ${isBlack ? "black" : "white"} ${isInScale ? "in-scale" : ""}`}
                  >
                    {!isBlack && (
                      <span className="pr-key-label">
                        {noteName}
                        {pitchClass === 0 ? octave : ""}
                      </span>
                    )}
                  </div>
                );
              })}
            </div>

            {/* Note grid */}
            <div className="pr-grid">
              {Array.from({ length: totalNotes }).map((_, rowIdx) => {
                const noteIdx = totalNotes - 1 - rowIdx;
                const pitchClass = noteIdx % 12;
                const midiNote = noteIdx + startOctave * 12;
                const isBlack = [1, 3, 6, 8, 10].includes(pitchClass);
                const isInScale = SCALE_NOTES.includes(pitchClass);
                return (
                  <div
                    key={rowIdx}
                    className={`pr-row ${isBlack ? "black" : "white"} ${isInScale ? "in-scale" : ""}`}
                  >
                    {/* Beat lines */}
                    {Array.from({ length: totalBeats }).map((_, beat) => (
                      <div
                        key={beat}
                        className={`pr-cell ${beat % 4 === 0 ? "bar-start" : ""}`}
                        style={{
                          left: `${(beat / totalBeats) * 100}%`,
                          width: `${100 / totalBeats}%`,
                        }}
                      />
                    ))}
                    {/* Notes */}
                    {notes
                      .filter((n) => n.pitch === midiNote)
                      .map((note) => (
                        <div
                          key={note.id}
                          className="pr-note"
                          style={{
                            left: `${(note.start / totalBeats) * 100}%`,
                            width: `${(note.duration / totalBeats) * 100}%`,
                          }}
                        />
                      ))}
                  </div>
                );
              })}
            </div>
          </div>

          {/* Velocity editor */}
          <div className="pr-velocity">
            <div className="pr-velocity-label">Velocity</div>
            <div className="pr-velocity-bars">
              {notes.map((note) => (
                <div
                  key={note.id}
                  className="pr-vel-bar"
                  style={{
                    left: `${(note.start / totalBeats) * 100}%`,
                    width: `${(note.duration / totalBeats) * 100}%`,
                    height: `${(note.velocity / 127) * 100}%`,
                  }}
                />
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Timeline */}
      <div className="timeline">
        <div className="tl-header">
          <div className="tl-title">Timeline</div>
          <div className="tl-controls">
            <button className="pr-tool-btn">+ Track</button>
            {!showPianoRoll && (
              <button
                className="pr-tool-btn"
                onClick={() => setShowPianoRoll(true)}
              >
                Piano Roll
              </button>
            )}
          </div>
        </div>

        <div className="tl-content">
          {/* Ruler */}
          <div className="tl-ruler">
            <div className="tl-ruler-offset" />
            {Array.from({ length: 17 }).map((_, i) => (
              <div
                key={i}
                className="tl-ruler-mark"
                style={{ left: `${(i / 16) * 100}%` }}
              >
                {i + 1}
              </div>
            ))}
          </div>

          {/* Tracks */}
          {[
            {
              name: "Krar",
              color: "var(--region-east-africa)",
              clips: [
                { start: 0, width: 25 },
                { start: 50, width: 25 },
              ],
            },
            {
              name: "Kebero",
              color: "var(--accent-error)",
              clips: [{ start: 0, width: 100 }],
            },
            {
              name: "Masenqo",
              color: "var(--region-middle-east)",
              clips: [{ start: 25, width: 50 }],
            },
          ].map((track, i) => (
            <div key={i} className="tl-track">
              <div className="tl-track-header">
                <div
                  className="tl-track-color"
                  style={{ background: track.color }}
                />
                <span className="tl-track-name">{track.name}</span>
                <div className="tl-track-btns">
                  <button className="tl-track-btn">S</button>
                  <button className="tl-track-btn">M</button>
                </div>
              </div>
              <div className="tl-track-lane">
                {track.clips.map((clip, j) => (
                  <div
                    key={j}
                    className="tl-clip"
                    style={{
                      left: `${clip.start}%`,
                      width: `${clip.width}%`,
                      borderColor: track.color,
                    }}
                  >
                    <div
                      className="tl-clip-wave"
                      style={{ background: track.color }}
                    />
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
