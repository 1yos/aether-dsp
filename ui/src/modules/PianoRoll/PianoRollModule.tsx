/**
 * Piano Roll Module — canvas-based pitch × time grid.
 * Click to add notes, click existing to delete, drag to move.
 * Emits InjectMidi intents during playback.
 */
import React, { useRef, useEffect, useState, useCallback } from "react";
import { useEngineStore } from "../../studio/store/engineStore";
import { useTransportStore } from "../useTransportStore";

const THEME = {
  bg: "#060c12",
  panel: "#0d1a26",
  border: "#1a2a3a",
  text: "#e0e8f0",
  textDim: "#4a6a8a",
  accent: "#4fc3f7",
  stop: "#ef5350",
  noteColor: "#4fc3f7",
  noteSelected: "#ffb74d",
  playhead: "#ef5350",
  blackKey: "#0a1520",
  whiteKey: "#1a2a3a",
  gridLine: "#0d1a26",
};

const NOTE_HEIGHT = 10;
const BEAT_WIDTH = 60;
const TOTAL_NOTES = 128;
const VISIBLE_NOTES = 64; // show notes 36–100 by default
const MIN_NOTE = 24;
const BEATS_VISIBLE = 16;

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
const isBlackKey = (note: number) => [1, 3, 6, 8, 10].includes(note % 12);

interface PianoNote {
  id: string;
  note: number; // MIDI note 0–127
  beat: number; // start beat (float)
  duration: number; // in beats
}

let noteIdCounter = 0;
const makeNoteId = () => `n${++noteIdCounter}`;

export const PianoRollModule: React.FC = () => {
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const { bpm, isPlaying, setBpm, play, stop } = useTransportStore();

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [notes, setNotes] = useState<PianoNote[]>([]);
  const [localBpm, setLocalBpm] = useState(bpm);
  const [scrollNote] = useState(MIN_NOTE); // bottom note visible
  const [scrollBeat] = useState(0);

  const notesRef = useRef(notes);
  notesRef.current = notes;

  const rafRef = useRef<number | null>(null);
  const playStartTimeRef = useRef<number>(0);
  const playStartBeatRef = useRef<number>(0);
  const firedNotesRef = useRef<Set<string>>(new Set());

  // Draw the canvas
  const draw = useCallback(
    (playheadBeat: number) => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;

      const W = canvas.width;
      const H = canvas.height;
      ctx.clearRect(0, 0, W, H);

      // Background
      ctx.fillStyle = THEME.bg;
      ctx.fillRect(0, 0, W, H);

      const pianoWidth = 36;

      // Draw piano keys on left
      for (let i = 0; i < VISIBLE_NOTES; i++) {
        const midiNote = scrollNote + VISIBLE_NOTES - 1 - i;
        if (midiNote < 0 || midiNote >= TOTAL_NOTES) continue;
        const y = i * NOTE_HEIGHT;
        const black = isBlackKey(midiNote);
        ctx.fillStyle = black ? THEME.blackKey : THEME.whiteKey;
        ctx.fillRect(0, y, pianoWidth - 2, NOTE_HEIGHT - 1);

        // Note name on C notes
        if (midiNote % 12 === 0) {
          ctx.fillStyle = THEME.textDim;
          ctx.font = "8px monospace";
          ctx.fillText(
            `C${Math.floor(midiNote / 12) - 1}`,
            2,
            y + NOTE_HEIGHT - 2,
          );
        }
      }

      // Grid lines
      for (let i = 0; i <= VISIBLE_NOTES; i++) {
        const y = i * NOTE_HEIGHT;
        const midiNote = scrollNote + VISIBLE_NOTES - i;
        ctx.strokeStyle = midiNote % 12 === 0 ? "#1a2a3a" : "#0d1520";
        ctx.lineWidth = midiNote % 12 === 0 ? 1 : 0.5;
        ctx.beginPath();
        ctx.moveTo(pianoWidth, y);
        ctx.lineTo(W, y);
        ctx.stroke();
      }

      // Beat grid lines
      for (let b = 0; b <= BEATS_VISIBLE; b++) {
        const x = pianoWidth + (b - (scrollBeat % 1)) * BEAT_WIDTH;
        if (x < pianoWidth || x > W) continue;
        const beatNum = Math.floor(scrollBeat) + b;
        ctx.strokeStyle = beatNum % 4 === 0 ? "#1a2a3a" : "#0d1520";
        ctx.lineWidth = beatNum % 4 === 0 ? 1 : 0.5;
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, H);
        ctx.stroke();

        // Bar numbers
        if (beatNum % 4 === 0) {
          ctx.fillStyle = THEME.textDim;
          ctx.font = "9px monospace";
          ctx.fillText(`${Math.floor(beatNum / 4) + 1}`, x + 2, 10);
        }
      }

      // Draw notes
      for (const n of notesRef.current) {
        const row = scrollNote + VISIBLE_NOTES - 1 - n.note;
        if (row < 0 || row >= VISIBLE_NOTES) continue;
        const x = pianoWidth + (n.beat - scrollBeat) * BEAT_WIDTH;
        const w = n.duration * BEAT_WIDTH - 2;
        const y = row * NOTE_HEIGHT + 1;
        if (x + w < pianoWidth || x > W) continue;

        ctx.fillStyle = THEME.noteColor;
        ctx.fillRect(x, y, Math.max(w, 4), NOTE_HEIGHT - 2);
        ctx.fillStyle = "#000";
        ctx.font = "8px monospace";
        ctx.fillText(NOTE_NAMES[n.note % 12], x + 2, y + NOTE_HEIGHT - 3);
      }

      // Playhead
      if (isPlaying || playheadBeat > 0) {
        const px = pianoWidth + (playheadBeat - scrollBeat) * BEAT_WIDTH;
        if (px >= pianoWidth && px <= W) {
          ctx.strokeStyle = THEME.playhead;
          ctx.lineWidth = 2;
          ctx.beginPath();
          ctx.moveTo(px, 0);
          ctx.lineTo(px, H);
          ctx.stroke();
        }
      }
    },
    [scrollNote, scrollBeat, isPlaying],
  );

  // Animation loop
  useEffect(() => {
    if (!isPlaying) {
      draw(0);
      return;
    }

    playStartTimeRef.current = performance.now();
    playStartBeatRef.current = 0;
    firedNotesRef.current = new Set();

    const tick = () => {
      const elapsed = (performance.now() - playStartTimeRef.current) / 1000;
      const beat = elapsed * (localBpm / 60);

      // Fire notes that start in this beat window
      for (const n of notesRef.current) {
        const key = n.id;
        if (
          !firedNotesRef.current.has(key) &&
          n.beat <= beat &&
          n.beat + n.duration > beat
        ) {
          firedNotesRef.current.add(key);
          sendIntent?.({
            type: "inject_midi",
            channel: 0,
            note: n.note,
            velocity: 100,
            is_note_on: true,
          });
          // Schedule note off
          const offDelay = ((n.duration * 60) / localBpm) * 1000 - 20;
          setTimeout(
            () => {
              sendIntent?.({
                type: "inject_midi",
                channel: 0,
                note: n.note,
                velocity: 0,
                is_note_on: false,
              });
            },
            Math.max(offDelay, 10),
          );
        }
      }

      draw(beat);
      rafRef.current = requestAnimationFrame(tick);
    };

    rafRef.current = requestAnimationFrame(tick);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    };
  }, [isPlaying, localBpm, draw, sendIntent]);

  // Draw on note changes when not playing
  useEffect(() => {
    if (!isPlaying) draw(0);
  }, [notes, draw, isPlaying]);

  // Canvas click handler
  const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const pianoWidth = 36;
    if (x < pianoWidth) return;

    const beat = scrollBeat + (x - pianoWidth) / BEAT_WIDTH;
    const noteRow = Math.floor(y / NOTE_HEIGHT);
    const midiNote = scrollNote + VISIBLE_NOTES - 1 - noteRow;
    if (midiNote < 0 || midiNote >= TOTAL_NOTES) return;

    // Check if clicking an existing note
    const existing = notesRef.current.find(
      (n) =>
        n.note === midiNote && beat >= n.beat && beat < n.beat + n.duration,
    );

    if (existing) {
      // Delete note
      setNotes((prev) => prev.filter((n) => n.id !== existing.id));
    } else {
      // Add note (snapped to 1/4 beat)
      const snappedBeat = Math.floor(beat * 4) / 4;
      setNotes((prev) => [
        ...prev,
        { id: makeNoteId(), note: midiNote, beat: snappedBeat, duration: 0.5 },
      ]);
    }
  };

  const handleBpmChange = (v: number) => {
    setLocalBpm(v);
    setBpm(v);
  };

  const handlePlayStop = () => {
    if (isPlaying) {
      stop();
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    } else {
      play();
    }
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: THEME.bg,
        fontFamily: "monospace",
        color: THEME.text,
      }}
    >
      {/* Controls */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 12,
          padding: "6px 12px",
          borderBottom: `1px solid ${THEME.border}`,
          flexWrap: "wrap",
        }}
      >
        <span
          style={{
            fontSize: 11,
            color: THEME.textDim,
            letterSpacing: 2,
            textTransform: "uppercase",
          }}
        >
          Piano Roll
        </span>

        <button
          onClick={handlePlayStop}
          style={{
            padding: "3px 14px",
            background: isPlaying ? THEME.stop : THEME.accent,
            border: "none",
            borderRadius: 4,
            color: "#000",
            fontWeight: "bold",
            fontSize: 12,
            cursor: "pointer",
            fontFamily: "monospace",
          }}
        >
          {isPlaying ? "■ Stop" : "▶ Play"}
        </button>

        <label
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            fontSize: 12,
          }}
        >
          <span style={{ color: THEME.textDim }}>BPM</span>
          <input
            type="number"
            min={40}
            max={300}
            value={localBpm}
            onChange={(e) => handleBpmChange(Number(e.target.value))}
            style={{
              width: 52,
              background: THEME.panel,
              border: `1px solid ${THEME.border}`,
              borderRadius: 4,
              color: THEME.text,
              padding: "2px 6px",
              fontSize: 12,
              fontFamily: "monospace",
            }}
          />
        </label>

        <button
          onClick={() => setNotes([])}
          style={{
            padding: "3px 10px",
            background: THEME.panel,
            border: `1px solid ${THEME.border}`,
            borderRadius: 4,
            color: THEME.textDim,
            fontSize: 11,
            cursor: "pointer",
            fontFamily: "monospace",
          }}
        >
          Clear
        </button>

        <span style={{ fontSize: 11, color: THEME.textDim }}>
          {notes.length} note{notes.length !== 1 ? "s" : ""}
        </span>
      </div>

      {/* Canvas */}
      <div style={{ flex: 1, overflow: "hidden", position: "relative" }}>
        <canvas
          ref={canvasRef}
          width={800}
          height={VISIBLE_NOTES * NOTE_HEIGHT}
          onClick={handleCanvasClick}
          style={{
            display: "block",
            cursor: "crosshair",
            width: "100%",
            height: "100%",
          }}
        />
      </div>
    </div>
  );
};

export default PianoRollModule;
