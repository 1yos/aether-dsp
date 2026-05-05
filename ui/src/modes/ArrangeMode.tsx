/**
 * Aether Studio — Arrange Mode
 *
 * Full DAW-style arrange view with non-Western piano roll.
 *
 * Features:
 * - Piano roll with scale highlighting for 14 tuning systems
 * - Microtonal cent-deviation indicators on keys
 * - Non-Western rhythmic grid: Teentaal, Rupak, Jhaptaal, Maqsum, Gamelan, etc.
 * - Beat names shown in grid header (Dha, Dhin, Dum, Tak, etc.)
 * - Accent pattern highlighting (downbeats glow)
 * - Note add/remove by clicking the grid
 * - Velocity editor
 * - Timeline with track clips
 */

import { useState, useCallback, useRef } from "react";
import { RHYTHMIC_SYSTEMS, getRhythmicSystem } from "./rhythmic-systems";
import {
  SCALE_SYSTEMS,
  getScaleSystem,
  isInScale,
  getCentDeviation,
} from "./scale-systems";
import "./ArrangeMode.css";

// ── Types ─────────────────────────────────────────────────────────────────────

interface Note {
  id: string;
  pitch: number; // MIDI note number
  beat: number; // beat position (0-indexed)
  duration: number; // in beats
  velocity: number; // 0-127
}

// ── Constants ─────────────────────────────────────────────────────────────────

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
const BLACK_PITCH_CLASSES = new Set([1, 3, 6, 8, 10]);

const OCTAVES = 4;
const START_OCTAVE = 2;
const TOTAL_NOTES = OCTAVES * 12;
const ROW_H = 18;
const KEY_W = 52;
const CELL_MIN_W = 28;

// ── Helpers ───────────────────────────────────────────────────────────────────

function noteId(): string {
  return Math.random().toString(36).slice(2, 9);
}

function midiToName(midi: number): string {
  const pc = midi % 12;
  const oct = Math.floor(midi / 12) - 1;
  return `${NOTE_NAMES[pc]}${oct}`;
}

// ── Piano Roll ────────────────────────────────────────────────────────────────

interface PianoRollProps {
  notes: Note[];
  onAddNote: (pitch: number, beat: number) => void;
  onRemoveNote: (id: string) => void;
  scaleId: string;
  rhythmId: string;
  totalCycles: number;
}

function PianoRoll({
  notes,
  onAddNote,
  onRemoveNote,
  scaleId,
  rhythmId,
  totalCycles,
}: PianoRollProps) {
  const scale = getScaleSystem(scaleId);
  const rhythm = getRhythmicSystem(rhythmId);
  const totalBeats = rhythm.beatsPerCycle * totalCycles;
  const gridRef = useRef<HTMLDivElement>(null);

  const cellW = Math.max(CELL_MIN_W, 800 / totalBeats);

  const handleGridClick = useCallback(
    (e: React.MouseEvent, rowMidi: number) => {
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const x = e.clientX - rect.left;
      const beat = Math.floor(x / cellW);
      if (beat < 0 || beat >= totalBeats) return;

      // Check if clicking an existing note
      const existing = notes.find(
        (n) =>
          n.pitch === rowMidi && beat >= n.beat && beat < n.beat + n.duration,
      );
      if (existing) {
        onRemoveNote(existing.id);
      } else {
        onAddNote(rowMidi, beat);
      }
    },
    [notes, onAddNote, onRemoveNote, cellW, totalBeats],
  );

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        flex: 1,
        overflow: "hidden",
        minHeight: 0,
      }}
    >
      {/* Beat header */}
      <div style={{ display: "flex", flexShrink: 0 }}>
        <div
          style={{
            width: KEY_W,
            flexShrink: 0,
            background: "#060c12",
            borderBottom: "1px solid #0f1e2e",
          }}
        />
        <div
          style={{
            flex: 1,
            overflowX: "hidden",
            background: "#060c12",
            borderBottom: "1px solid #0f1e2e",
          }}
        >
          <div
            style={{
              display: "flex",
              height: 32,
              position: "relative",
              width: totalBeats * cellW,
            }}
          >
            {Array.from({ length: totalBeats }, (_, beat) => {
              const beatInCycle = beat % rhythm.beatsPerCycle;
              const isAccent = rhythm.accentPattern.includes(beatInCycle);
              const isCycleStart = beatInCycle === 0;
              const beatName = rhythm.beatNames?.[beatInCycle];
              return (
                <div
                  key={beat}
                  style={{
                    width: cellW,
                    flexShrink: 0,
                    borderLeft: `1px solid ${isCycleStart ? "#1e3a5f" : "#0a1520"}`,
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    background: isAccent
                      ? "rgba(56,189,248,0.06)"
                      : "transparent",
                    position: "relative",
                  }}
                >
                  {isCycleStart && (
                    <span
                      style={{
                        fontSize: 9,
                        color: "#38bdf8",
                        fontWeight: 700,
                        fontFamily: "monospace",
                      }}
                    >
                      {Math.floor(beat / rhythm.beatsPerCycle) + 1}
                    </span>
                  )}
                  {beatName && (
                    <span
                      style={{
                        fontSize: 8,
                        color: isAccent ? "#38bdf8" : "#2a3a4a",
                        fontFamily: "monospace",
                        fontWeight: isAccent ? 700 : 400,
                      }}
                    >
                      {beatName}
                    </span>
                  )}
                  {!beatName && !isCycleStart && (
                    <span
                      style={{
                        fontSize: 8,
                        color: "#1a2a3a",
                        fontFamily: "monospace",
                      }}
                    >
                      {beatInCycle + 1}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      </div>

      {/* Keys + Grid */}
      <div ref={gridRef} style={{ display: "flex", flex: 1, overflow: "auto" }}>
        {/* Piano keys */}
        <div
          style={{
            width: KEY_W,
            flexShrink: 0,
            position: "sticky",
            left: 0,
            zIndex: 2,
            background: "#060c12",
          }}
        >
          {Array.from({ length: TOTAL_NOTES }, (_, i) => {
            const noteIdx = TOTAL_NOTES - 1 - i;
            const midi = noteIdx + START_OCTAVE * 12;
            const pc = midi % 12;
            const isBlack = BLACK_PITCH_CLASSES.has(pc);
            const inScale = isInScale(pc, scale);
            const cents = getCentDeviation(pc, scale);
            const showCents = Math.abs(cents) > 5;
            const isC = pc === 0;

            return (
              <div
                key={i}
                style={{
                  height: ROW_H,
                  display: "flex",
                  alignItems: "center",
                  paddingLeft: 6,
                  paddingRight: 4,
                  background: isBlack
                    ? inScale
                      ? "rgba(167,139,250,0.15)"
                      : "#0a1218"
                    : inScale
                      ? `rgba(${hexToRgbStr(scale.color)},0.08)`
                      : "#0d1520",
                  borderBottom: "1px solid #0a1520",
                  borderLeft: `3px solid ${inScale ? scale.color : isBlack ? "#1a2a3a" : "#1e2d3d"}`,
                  position: "relative",
                  overflow: "hidden",
                  cursor: "default",
                }}
                title={
                  showCents
                    ? `${NOTE_NAMES[pc]} ${cents > 0 ? "+" : ""}${cents.toFixed(1)}¢`
                    : undefined
                }
              >
                {/* Cent deviation bar */}
                {showCents && (
                  <div
                    style={{
                      position: "absolute",
                      right: 0,
                      top: 0,
                      bottom: 0,
                      width: Math.min((Math.abs(cents) / 50) * 12, 12),
                      background:
                        cents < 0
                          ? "rgba(56,189,248,0.4)"
                          : "rgba(251,191,36,0.4)",
                    }}
                  />
                )}
                <span
                  style={{
                    fontSize: 9,
                    color: inScale
                      ? scale.color
                      : isBlack
                        ? "#2a3a4a"
                        : "#334155",
                    fontFamily: "monospace",
                    fontWeight: isC ? 700 : 400,
                    zIndex: 1,
                  }}
                >
                  {isC ? midiToName(midi) : inScale ? NOTE_NAMES[pc] : ""}
                </span>
              </div>
            );
          })}
        </div>

        {/* Note grid */}
        <div style={{ flex: 1, position: "relative" }}>
          {Array.from({ length: TOTAL_NOTES }, (_, i) => {
            const noteIdx = TOTAL_NOTES - 1 - i;
            const midi = noteIdx + START_OCTAVE * 12;
            const pc = midi % 12;
            const isBlack = BLACK_PITCH_CLASSES.has(pc);
            const inScale = isInScale(pc, scale);
            const rowNotes = notes.filter((n) => n.pitch === midi);

            return (
              <div
                key={i}
                style={{
                  height: ROW_H,
                  width: totalBeats * cellW,
                  display: "flex",
                  position: "relative",
                  background: isBlack
                    ? inScale
                      ? "rgba(167,139,250,0.04)"
                      : "#080e18"
                    : inScale
                      ? `rgba(${hexToRgbStr(scale.color)},0.03)`
                      : "#0b1420",
                  borderBottom: "1px solid #0a1520",
                  cursor: "crosshair",
                }}
                onClick={(e) => handleGridClick(e, midi)}
              >
                {/* Beat cells */}
                {Array.from({ length: totalBeats }, (_, beat) => {
                  const beatInCycle = beat % rhythm.beatsPerCycle;
                  const isCycleStart = beatInCycle === 0;
                  const isAccent = rhythm.accentPattern.includes(beatInCycle);
                  return (
                    <div
                      key={beat}
                      style={{
                        width: cellW,
                        flexShrink: 0,
                        height: "100%",
                        borderLeft: `1px solid ${isCycleStart ? "#1a2a3a" : "#0a1520"}`,
                        background: isAccent
                          ? "rgba(56,189,248,0.02)"
                          : "transparent",
                      }}
                    />
                  );
                })}

                {/* Notes */}
                {rowNotes.map((note) => (
                  <div
                    key={note.id}
                    onClick={(e) => {
                      e.stopPropagation();
                      onRemoveNote(note.id);
                    }}
                    style={{
                      position: "absolute",
                      left: note.beat * cellW + 1,
                      width: note.duration * cellW - 2,
                      top: 2,
                      height: ROW_H - 4,
                      background: `linear-gradient(90deg, ${scale.color}, ${scale.color}cc)`,
                      borderRadius: 3,
                      cursor: "pointer",
                      boxShadow: `0 0 6px ${scale.color}60`,
                      zIndex: 1,
                    }}
                    title={`${midiToName(midi)} vel:${note.velocity}`}
                  />
                ))}
              </div>
            );
          })}
        </div>
      </div>

      {/* Velocity editor */}
      <div
        style={{
          height: 60,
          flexShrink: 0,
          display: "flex",
          borderTop: "1px solid #0f1e2e",
          background: "#060c12",
        }}
      >
        <div
          style={{
            width: KEY_W,
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            paddingLeft: 8,
          }}
        >
          <span
            style={{
              fontSize: 9,
              color: "#2a3a4a",
              fontFamily: "monospace",
              letterSpacing: 1,
            }}
          >
            VEL
          </span>
        </div>
        <div style={{ flex: 1, position: "relative", overflow: "hidden" }}>
          <div
            style={{
              position: "absolute",
              inset: 0,
              display: "flex",
              alignItems: "flex-end",
              padding: "4px 0",
            }}
          >
            {notes.map((note) => (
              <div
                key={note.id}
                style={{
                  position: "absolute",
                  left: note.beat * cellW,
                  width: Math.max(cellW - 2, 4),
                  bottom: 4,
                  height: `${(note.velocity / 127) * 48}px`,
                  background: scale.color,
                  opacity: 0.7,
                  borderRadius: "2px 2px 0 0",
                  cursor: "ns-resize",
                }}
                title={`Velocity: ${note.velocity}`}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Main ArrangeMode ──────────────────────────────────────────────────────────

export function ArrangeMode() {
  const [notes, setNotes] = useState<Note[]>([
    { id: "1", pitch: 60, beat: 0, duration: 1, velocity: 90 },
    { id: "2", pitch: 62, beat: 2, duration: 1, velocity: 80 },
    { id: "3", pitch: 64, beat: 4, duration: 2, velocity: 100 },
    { id: "4", pitch: 67, beat: 8, duration: 1, velocity: 85 },
    { id: "5", pitch: 60, beat: 10, duration: 1, velocity: 75 },
    { id: "6", pitch: 64, beat: 12, duration: 2, velocity: 95 },
  ]);

  const [scaleId, setScaleId] = useState("ethiopian-tizita");
  const [rhythmId, setRhythmId] = useState("12-8");
  const [totalCycles, setTotalCycles] = useState(2);
  const [showPianoRoll, setShowPianoRoll] = useState(true);

  const scale = getScaleSystem(scaleId);
  const rhythm = getRhythmicSystem(rhythmId);

  const addNote = useCallback((pitch: number, beat: number) => {
    setNotes((prev) => [
      ...prev,
      { id: noteId(), pitch, beat, duration: 1, velocity: 90 },
    ]);
  }, []);

  const removeNote = useCallback((id: string) => {
    setNotes((prev) => prev.filter((n) => n.id !== id));
  }, []);

  return (
    <div className="arrange-mode">
      {/* Piano Roll */}
      {showPianoRoll && (
        <div
          className="piano-roll"
          style={{ display: "flex", flexDirection: "column" }}
        >
          {/* Header */}
          <div className="pr-header" style={{ flexShrink: 0 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <div className="pr-title">Piano Roll</div>
              <div
                style={{
                  width: 8,
                  height: 8,
                  borderRadius: "50%",
                  background: scale.color,
                  boxShadow: `0 0 6px ${scale.color}`,
                }}
              />
              <span
                style={{ fontSize: 11, color: scale.color, fontWeight: 600 }}
              >
                {scale.name}
              </span>
            </div>
            <div className="pr-controls">
              {/* Scale selector */}
              <select
                className="pr-scale-select"
                value={scaleId}
                onChange={(e) => setScaleId(e.target.value)}
                style={{ minWidth: 180 }}
              >
                {SCALE_SYSTEMS.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.name}
                  </option>
                ))}
              </select>

              {/* Rhythm selector */}
              <select
                className="pr-scale-select"
                value={rhythmId}
                onChange={(e) => setRhythmId(e.target.value)}
                style={{ minWidth: 180 }}
              >
                {RHYTHMIC_SYSTEMS.map((r) => (
                  <option key={r.id} value={r.id}>
                    {r.name}
                  </option>
                ))}
              </select>

              {/* Cycles */}
              <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
                <span style={{ fontSize: 10, color: "#4a6a8a" }}>Cycles:</span>
                <button
                  className="pr-tool-btn"
                  onClick={() => setTotalCycles((c) => Math.max(1, c - 1))}
                >
                  −
                </button>
                <span
                  style={{
                    fontSize: 11,
                    color: "#e0e8f0",
                    minWidth: 16,
                    textAlign: "center",
                  }}
                >
                  {totalCycles}
                </span>
                <button
                  className="pr-tool-btn"
                  onClick={() => setTotalCycles((c) => Math.min(8, c + 1))}
                >
                  +
                </button>
              </div>

              <button
                className="pr-close-btn"
                onClick={() => setShowPianoRoll(false)}
              >
                ✕
              </button>
            </div>
          </div>

          {/* Rhythm info bar */}
          <div
            style={{
              padding: "4px 12px",
              background: "#060c12",
              borderBottom: "1px solid #0a1520",
              display: "flex",
              alignItems: "center",
              gap: 12,
              flexShrink: 0,
            }}
          >
            <span style={{ fontSize: 10, color: "#4a6a8a" }}>
              {rhythm.region} · {rhythm.beatsPerCycle} beats/cycle ·{" "}
              {rhythm.description}
            </span>
            <span
              style={{ fontSize: 10, color: "#2a3a4a", marginLeft: "auto" }}
            >
              {scale.description}
            </span>
          </div>

          <PianoRoll
            notes={notes}
            onAddNote={addNote}
            onRemoveNote={removeNote}
            scaleId={scaleId}
            rhythmId={rhythmId}
            totalCycles={totalCycles}
          />
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
          <div className="tl-ruler">
            <div className="tl-ruler-offset" />
            {Array.from(
              { length: totalCycles * rhythm.beatsPerCycle + 1 },
              (_, i) => (
                <div
                  key={i}
                  className="tl-ruler-mark"
                  style={{
                    left: `${(i / (totalCycles * rhythm.beatsPerCycle)) * 100}%`,
                  }}
                >
                  {i % rhythm.beatsPerCycle === 0
                    ? Math.floor(i / rhythm.beatsPerCycle) + 1
                    : ""}
                </div>
              ),
            )}
          </div>

          {[
            {
              name: "Krar",
              color: "#d4a017",
              clips: [
                { start: 0, width: 50 },
                { start: 60, width: 30 },
              ],
            },
            {
              name: "Kebero",
              color: "#ef4444",
              clips: [{ start: 0, width: 100 }],
            },
            {
              name: "Masenqo",
              color: "#a78bfa",
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

// ── Helpers ───────────────────────────────────────────────────────────────────

function hexToRgbStr(hex: string): string {
  if (!hex.startsWith("#") || hex.length < 7) return "56,189,248";
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `${r},${g},${b}`;
}
