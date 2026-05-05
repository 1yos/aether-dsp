/**
 * PianoRollView — full piano roll for editing MIDI clips.
 * Opens when you double-click a clip in the Song View.
 */

import { useCallback, useRef } from "react";
import { useDawStore } from "../store/dawStore";
import {
  getScaleSystem,
  isInScale,
  getCentDeviation,
} from "../../modes/scale-systems";
import { getRhythmicSystem } from "../../modes/rhythmic-systems";
import { SCALE_SYSTEMS } from "../../modes/scale-systems";
import { RHYTHMIC_SYSTEMS } from "../../modes/rhythmic-systems";

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
const BLACK_PCS = new Set([1, 3, 6, 8, 10]);
const ROW_H = 16;
const KEY_W = 48;
const BEAT_W = 40;
const TOTAL_OCTAVES = 5;
const START_OCTAVE = 2;
const TOTAL_NOTES = TOTAL_OCTAVES * 12;

function midiToName(midi: number) {
  return `${NOTE_NAMES[midi % 12]}${Math.floor(midi / 12) - 1}`;
}

export function PianoRollView() {
  const pianoRollClipId = useDawStore((s) => s.pianoRollClipId);
  const pianoRollTrackId = useDawStore((s) => s.pianoRollTrackId);
  const tracks = useDawStore((s) => s.tracks);
  const scaleId = useDawStore((s) => s.scaleId);
  const rhythmId = useDawStore((s) => s.rhythmId);
  const setScaleId = useDawStore((s) => s.setScaleId);
  const setRhythmId = useDawStore((s) => s.setRhythmId);
  const addNote = useDawStore((s) => s.addNote);
  const removeNote = useDawStore((s) => s.removeNote);
  const setView = useDawStore((s) => s.setView);

  const scale = getScaleSystem(scaleId);
  const rhythm = getRhythmicSystem(rhythmId);

  const track = tracks.find((t) => t.id === pianoRollTrackId);
  const clip = track?.clips.find((c) => c.id === pianoRollClipId);

  const totalBeats = clip ? clip.lengthBeats : 16;
  const gridRef = useRef<HTMLDivElement>(null);

  const handleGridClick = useCallback(
    (e: React.MouseEvent, midi: number) => {
      if (!clip) return;
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const x = e.clientX - rect.left;
      const beat = Math.floor(x / BEAT_W);
      if (beat < 0 || beat >= totalBeats) return;

      const existing = clip.notes.find(
        (n) => n.pitch === midi && beat >= n.beat && beat < n.beat + n.duration,
      );
      if (existing) {
        removeNote(clip.id, existing.id);
      } else {
        addNote(clip.id, { pitch: midi, beat, duration: 1, velocity: 90 });
      }
    },
    [clip, totalBeats, addNote, removeNote],
  );

  const border = "#0f1e2e";
  const dim = "#475569";

  if (!clip) {
    return (
      <div
        style={{
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexDirection: "column",
          gap: 12,
          color: dim,
          background: "#060c12",
        }}
      >
        <span style={{ fontSize: 32 }}>♩</span>
        <span style={{ fontSize: 13 }}>No clip selected</span>
        <button
          onClick={() => setView("song")}
          style={{
            padding: "6px 16px",
            background: "rgba(77,184,255,0.08)",
            border: "1px solid rgba(77,184,255,0.2)",
            borderRadius: 6,
            color: "#4db8ff",
            fontSize: 12,
            cursor: "pointer",
            fontFamily: "var(--font-sans)",
          }}
        >
          ← Back to Song
        </button>
      </div>
    );
  }

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: "#060c12",
        overflow: "hidden",
      }}
    >
      {/* Toolbar */}
      <div
        style={{
          height: 36,
          background: "#080e18",
          borderBottom: `1px solid ${border}`,
          display: "flex",
          alignItems: "center",
          padding: "0 12px",
          gap: 8,
          flexShrink: 0,
        }}
      >
        <button
          onClick={() => setView("song")}
          style={{
            padding: "3px 10px",
            background: "transparent",
            border: `1px solid ${border}`,
            borderRadius: 4,
            color: dim,
            fontSize: 11,
            cursor: "pointer",
            fontFamily: "var(--font-sans)",
          }}
        >
          ← Song
        </button>

        <div style={{ width: 1, height: 16, background: border }} />

        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            padding: "2px 8px",
            background: `${track?.color ?? "#4db8ff"}18`,
            border: `1px solid ${track?.color ?? "#4db8ff"}40`,
            borderRadius: 4,
          }}
        >
          <div
            style={{
              width: 8,
              height: 8,
              borderRadius: 2,
              background: track?.color ?? "#4db8ff",
            }}
          />
          <span
            style={{
              fontSize: 11,
              color: track?.color ?? "#4db8ff",
              fontWeight: 600,
            }}
          >
            {track?.name} — {clip.name}
          </span>
        </div>

        <div style={{ flex: 1 }} />

        <select
          value={scaleId}
          onChange={(e) => setScaleId(e.target.value)}
          style={{
            background: "#0a1520",
            border: `1px solid ${border}`,
            borderRadius: 4,
            color: "#e0e8f0",
            padding: "3px 6px",
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

        <select
          value={rhythmId}
          onChange={(e) => setRhythmId(e.target.value)}
          style={{
            background: "#0a1520",
            border: `1px solid ${border}`,
            borderRadius: 4,
            color: "#e0e8f0",
            padding: "3px 6px",
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

      {/* Beat header */}
      <div style={{ display: "flex", flexShrink: 0 }}>
        <div
          style={{
            width: KEY_W,
            flexShrink: 0,
            background: "#060c12",
            borderBottom: `1px solid ${border}`,
          }}
        />
        <div
          style={{
            flex: 1,
            overflowX: "hidden",
            background: "#060c12",
            borderBottom: `1px solid ${border}`,
          }}
        >
          <div
            style={{ display: "flex", height: 24, width: totalBeats * BEAT_W }}
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
                    width: BEAT_W,
                    flexShrink: 0,
                    borderLeft: `1px solid ${isCycleStart ? "#1e3a5f" : "#0a1520"}`,
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    background: isAccent
                      ? "rgba(77,184,255,0.04)"
                      : "transparent",
                  }}
                >
                  {isCycleStart && (
                    <span
                      style={{
                        fontSize: 8,
                        color: "#4db8ff",
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
                        fontSize: 7,
                        color: isAccent ? "#4db8ff" : "#1e2d3a",
                        fontFamily: "monospace",
                      }}
                    >
                      {beatName}
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
            const isBlack = BLACK_PCS.has(pc);
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
                  paddingLeft: 5,
                  paddingRight: 3,
                  background: isBlack
                    ? inScale
                      ? "rgba(167,139,250,0.12)"
                      : "#0a1218"
                    : inScale
                      ? `rgba(77,184,255,0.06)`
                      : "#0d1520",
                  borderBottom: "1px solid #0a1520",
                  borderLeft: `3px solid ${inScale ? scale.color : isBlack ? "#1a2a3a" : "#1e2d3d"}`,
                  position: "relative",
                  overflow: "hidden",
                }}
              >
                {showCents && (
                  <div
                    style={{
                      position: "absolute",
                      right: 0,
                      top: 0,
                      bottom: 0,
                      width: Math.min((Math.abs(cents) / 50) * 10, 10),
                      background:
                        cents < 0
                          ? "rgba(77,184,255,0.35)"
                          : "rgba(251,191,36,0.35)",
                    }}
                  />
                )}
                <span
                  style={{
                    fontSize: 8,
                    color: inScale
                      ? scale.color
                      : isBlack
                        ? "#1e2d3a"
                        : "#2a3a4a",
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
            const isBlack = BLACK_PCS.has(pc);
            const inScale = isInScale(pc, scale);
            const rowNotes = clip.notes.filter((n) => n.pitch === midi);

            return (
              <div
                key={i}
                style={{
                  height: ROW_H,
                  width: totalBeats * BEAT_W,
                  display: "flex",
                  position: "relative",
                  background: isBlack
                    ? inScale
                      ? "rgba(167,139,250,0.03)"
                      : "#080e18"
                    : inScale
                      ? "rgba(77,184,255,0.02)"
                      : "#0b1420",
                  borderBottom: "1px solid #0a1520",
                  cursor: "crosshair",
                }}
                onClick={(e) => handleGridClick(e, midi)}
              >
                {Array.from({ length: totalBeats }, (_, beat) => {
                  const beatInCycle = beat % rhythm.beatsPerCycle;
                  const isCycleStart = beatInCycle === 0;
                  const isAccent = rhythm.accentPattern.includes(beatInCycle);
                  return (
                    <div
                      key={beat}
                      style={{
                        width: BEAT_W,
                        flexShrink: 0,
                        height: "100%",
                        borderLeft: `1px solid ${isCycleStart ? "#1a2a3a" : "#0a1520"}`,
                        background: isAccent
                          ? "rgba(77,184,255,0.015)"
                          : "transparent",
                      }}
                    />
                  );
                })}

                {rowNotes.map((note) => (
                  <div
                    key={note.id}
                    onClick={(e) => {
                      e.stopPropagation();
                      removeNote(clip.id, note.id);
                    }}
                    style={{
                      position: "absolute",
                      left: note.beat * BEAT_W + 1,
                      width: note.duration * BEAT_W - 2,
                      top: 2,
                      height: ROW_H - 4,
                      background: `linear-gradient(90deg, ${scale.color}, ${scale.color}cc)`,
                      borderRadius: 2,
                      cursor: "pointer",
                      boxShadow: `0 0 4px ${scale.color}50`,
                      zIndex: 1,
                    }}
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
          height: 52,
          flexShrink: 0,
          display: "flex",
          borderTop: `1px solid ${border}`,
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
              fontSize: 8,
              color: "#1e2d3a",
              fontFamily: "monospace",
              letterSpacing: 1,
            }}
          >
            VEL
          </span>
        </div>
        <div style={{ flex: 1, position: "relative", overflow: "hidden" }}>
          {clip.notes.map((note) => (
            <div
              key={note.id}
              style={{
                position: "absolute",
                left: note.beat * BEAT_W,
                width: Math.max(BEAT_W - 2, 4),
                bottom: 4,
                height: `${(note.velocity / 127) * 44}px`,
                background: scale.color,
                opacity: 0.7,
                borderRadius: "2px 2px 0 0",
              }}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
