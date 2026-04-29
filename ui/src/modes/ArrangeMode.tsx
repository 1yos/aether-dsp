/**
 * Aether Studio v2.0 - Arrange Mode
 * Timeline and piano roll for composition
 */

import { useState } from "react";
import "./ArrangeMode.css";

interface Note {
  id: string;
  pitch: number; // MIDI note number
  start: number; // In beats
  duration: number; // In beats
  velocity: number; // 0-127
}

const SCALE_NOTES = [0, 2, 3, 5, 7, 8, 10]; // Tizita scale (Ethiopian)
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

export function ArrangeMode() {
  const [notes] = useState<Note[]>([
    { id: "1", pitch: 60, start: 0, duration: 1, velocity: 80 },
    { id: "2", pitch: 62, start: 1, duration: 1, velocity: 90 },
    { id: "3", pitch: 64, start: 2, duration: 2, velocity: 100 },
    { id: "4", pitch: 67, start: 4, duration: 1, velocity: 85 },
  ]);
  const [showPianoRoll, setShowPianoRoll] = useState(true);

  const renderPianoRoll = () => {
    const octaves = 3;
    const startOctave = 3;
    const rows = [];

    for (
      let octave = startOctave + octaves - 1;
      octave >= startOctave;
      octave--
    ) {
      for (let note = 11; note >= 0; note--) {
        const midiNote = octave * 12 + note;
        const noteName = NOTE_NAMES[note];
        const isWhiteKey = ![1, 3, 6, 8, 10].includes(note);
        const isInScale = SCALE_NOTES.includes(note);

        rows.push(
          <div
            key={midiNote}
            className={`piano-roll-row ${isWhiteKey ? "white-key" : "black-key"} ${isInScale ? "in-scale" : ""}`}
          >
            <div className="piano-key-label">
              <span className="note-name">{noteName}</span>
              {note === 0 && <span className="octave-number">{octave}</span>}
            </div>
            <div className="note-grid">
              {/* Grid lines */}
              {Array.from({ length: 16 }).map((_, i) => (
                <div
                  key={i}
                  className="grid-line"
                  style={{ left: `${i * 6.25}%` }}
                />
              ))}
              {/* Notes */}
              {notes
                .filter((n) => n.pitch === midiNote)
                .map((note) => (
                  <div
                    key={note.id}
                    className="note-block"
                    style={{
                      left: `${(note.start / 16) * 100}%`,
                      width: `${(note.duration / 16) * 100}%`,
                    }}
                  >
                    <div className="note-resize-handle left" />
                    <div className="note-resize-handle right" />
                  </div>
                ))}
            </div>
          </div>,
        );
      }
    }

    return rows;
  };

  return (
    <div className="arrange-mode">
      {/* Piano Roll */}
      {showPianoRoll && (
        <div className="piano-roll-section">
          <div className="piano-roll-header">
            <h2>Piano Roll • Krar Melody</h2>
            <div className="piano-roll-controls">
              <select className="scale-selector">
                <option>Tizita (Ethiopian)</option>
                <option>Bati (Ethiopian)</option>
                <option>Anchihoye (Ethiopian)</option>
                <option>Chromatic</option>
              </select>
              <label className="checkbox-label">
                <input type="checkbox" defaultChecked />
                <span>Ghost Notes</span>
              </label>
              <button className="tool-btn" title="Quantize">
                ⊞
              </button>
              <button className="tool-btn" title="Humanize">
                ≈
              </button>
              <button
                className="close-btn"
                onClick={() => setShowPianoRoll(false)}
                title="Close Piano Roll"
              >
                ✕
              </button>
            </div>
          </div>

          <div className="piano-roll-content">
            <div className="piano-roll-grid">{renderPianoRoll()}</div>
          </div>

          <div className="velocity-editor">
            <div className="velocity-label">Velocity</div>
            <div className="velocity-bars">
              {notes.map((note) => (
                <div
                  key={note.id}
                  className="velocity-bar"
                  style={{
                    left: `${(note.start / 16) * 100}%`,
                    width: `${(note.duration / 16) * 100}%`,
                    height: `${(note.velocity / 127) * 100}%`,
                  }}
                />
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Timeline */}
      <div className="timeline-section">
        <div className="timeline-header">
          <h2>Timeline</h2>
          <div className="timeline-controls">
            <button className="tool-btn">+ Track</button>
            <button className="tool-btn">+ Marker</button>
            {!showPianoRoll && (
              <button
                className="tool-btn"
                onClick={() => setShowPianoRoll(true)}
              >
                Open Piano Roll
              </button>
            )}
          </div>
        </div>

        <div className="timeline-content">
          <div className="timeline-tracks">
            {/* Track 1 */}
            <div className="timeline-track">
              <div className="track-header">
                <span className="track-name">Krar</span>
                <div className="track-controls">
                  <button className="track-btn solo">S</button>
                  <button className="track-btn mute">M</button>
                </div>
              </div>
              <div className="track-lane">
                <div className="clip" style={{ left: "0%", width: "25%" }}>
                  <span className="clip-name">Melody 1</span>
                </div>
                <div className="clip" style={{ left: "50%", width: "25%" }}>
                  <span className="clip-name">Melody 2</span>
                </div>
              </div>
            </div>

            {/* Track 2 */}
            <div className="timeline-track">
              <div className="track-header">
                <span className="track-name">Kebero</span>
                <div className="track-controls">
                  <button className="track-btn solo">S</button>
                  <button className="track-btn mute">M</button>
                </div>
              </div>
              <div className="track-lane">
                <div className="clip" style={{ left: "0%", width: "100%" }}>
                  <span className="clip-name">Drums</span>
                </div>
              </div>
            </div>

            {/* Track 3 */}
            <div className="timeline-track">
              <div className="track-header">
                <span className="track-name">Masenqo</span>
                <div className="track-controls">
                  <button className="track-btn solo">S</button>
                  <button className="track-btn mute">M</button>
                </div>
              </div>
              <div className="track-lane">
                <div className="clip" style={{ left: "25%", width: "50%" }}>
                  <span className="clip-name">Bass Line</span>
                </div>
              </div>
            </div>
          </div>

          {/* Ruler */}
          <div className="timeline-ruler">
            {Array.from({ length: 17 }).map((_, i) => (
              <div
                key={i}
                className="ruler-mark"
                style={{ left: `${(i / 16) * 100}%` }}
              >
                <span className="ruler-label">{i + 1}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
