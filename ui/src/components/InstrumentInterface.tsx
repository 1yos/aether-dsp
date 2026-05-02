/**
 * Custom instrument playing interfaces.
 *
 * Each instrument family gets a purpose-built UI:
 *   - plucked-string  → string fretboard (Krar: 5 strings, Oud: 11 strings, etc.)
 *   - percussion      → drum pad grid (Djembe, Kpanlogo, Tabla, etc.)
 *   - wind            → breath/pitch pad
 *   - bowed-string    → bow pressure + string layout
 *   - keyboard        → falls back to KeyboardPlayer piano
 *
 * All interfaces emit MIDI note-on/off via the same handleNoteOn/handleNoteOff
 * callbacks so they work identically with the engine or Web Audio fallback.
 */

import { useState, useCallback, useRef } from "react";
import type { Instrument } from "../catalog/types";

interface InterfaceProps {
  instrument: Instrument;
  octave: number;
  velocity: number;
  activeNotes: Set<number>;
  onNoteOn: (midi: number) => void;
  onNoteOff: (midi: number) => void;
}

// ── String instrument layouts ─────────────────────────────────────────────────

interface StringConfig {
  strings: number; // number of strings
  frets: number; // frets to show
  tuning: number[]; // open-string MIDI notes (low → high)
  label: string;
  color: string;
}

const STRING_CONFIGS: Record<string, StringConfig> = {
  krar: {
    strings: 5,
    frets: 8,
    tuning: [48, 52, 55, 59, 62], // C3 E3 G3 B3 D4
    label: "Krar — Ethiopian lyre",
    color: "#d4a017",
  },
  oud: {
    strings: 11,
    frets: 12,
    tuning: [40, 45, 50, 55, 57, 62, 40, 45, 50, 55, 57],
    label: "Oud — Arabic lute",
    color: "#c8a060",
  },
  sitar: {
    strings: 7,
    frets: 20,
    tuning: [48, 50, 53, 55, 57, 60, 65],
    label: "Sitar — Indian plucked string",
    color: "#e8c070",
  },
  kora: {
    strings: 21,
    frets: 1,
    tuning: Array.from({ length: 21 }, (_, i) => 48 + i),
    label: "Kora — West African harp-lute",
    color: "#a0c8a0",
  },
  default_plucked: {
    strings: 6,
    frets: 12,
    tuning: [40, 45, 50, 55, 59, 64], // standard guitar
    label: "Plucked string",
    color: "#c8a060",
  },
};

function getStringConfig(instrument: Instrument): StringConfig {
  const name = instrument.name.toLowerCase();
  if (name.includes("krar")) return STRING_CONFIGS.krar;
  if (name.includes("oud")) return STRING_CONFIGS.oud;
  if (name.includes("sitar")) return STRING_CONFIGS.sitar;
  if (name.includes("kora")) return STRING_CONFIGS.kora;
  return STRING_CONFIGS.default_plucked;
}

// ── Percussion pad layouts ────────────────────────────────────────────────────

interface DrumPad {
  label: string;
  midi: number;
  color: string;
  size?: "large" | "medium" | "small";
  key?: string;
}

interface DrumConfig {
  pads: DrumPad[];
  layout: "circle" | "grid" | "tabla";
  label: string;
}

const DRUM_CONFIGS: Record<string, DrumConfig> = {
  djembe: {
    layout: "circle",
    label: "Djembe — West African drum",
    pads: [
      { label: "Bass", midi: 36, color: "#ef5350", size: "large", key: "F" },
      { label: "Tone", midi: 38, color: "#ff8c42", size: "medium", key: "D" },
      { label: "Slap", midi: 40, color: "#ffd54f", size: "medium", key: "S" },
      { label: "Rim", midi: 37, color: "#a5d6a7", size: "small", key: "A" },
      { label: "Mute", midi: 39, color: "#80cbc4", size: "small", key: "G" },
    ],
  },
  tabla: {
    layout: "tabla",
    label: "Tabla — Indian hand drum pair",
    pads: [
      { label: "Na", midi: 38, color: "#ce93d8", size: "small", key: "A" },
      { label: "Tin", midi: 40, color: "#b39ddb", size: "small", key: "S" },
      { label: "Te", midi: 42, color: "#9fa8da", size: "small", key: "D" },
      { label: "Ge", midi: 36, color: "#ef9a9a", size: "large", key: "F" },
      { label: "Ka", midi: 37, color: "#f48fb1", size: "medium", key: "G" },
      { label: "Dha", midi: 41, color: "#80cbc4", size: "medium", key: "H" },
    ],
  },
  kpanlogo: {
    layout: "grid",
    label: "Kpanlogo — Ghanaian drum",
    pads: [
      { label: "Low", midi: 36, color: "#ef5350", size: "large", key: "A" },
      { label: "Mid", midi: 38, color: "#ff8c42", size: "medium", key: "S" },
      { label: "High", midi: 40, color: "#ffd54f", size: "medium", key: "D" },
      { label: "Open", midi: 42, color: "#a5d6a7", size: "small", key: "F" },
    ],
  },
  default_percussion: {
    layout: "grid",
    label: "Percussion",
    pads: [
      { label: "1", midi: 36, color: "#ef5350", size: "large", key: "A" },
      { label: "2", midi: 38, color: "#ff8c42", size: "medium", key: "S" },
      { label: "3", midi: 40, color: "#ffd54f", size: "medium", key: "D" },
      { label: "4", midi: 42, color: "#a5d6a7", size: "small", key: "F" },
      { label: "5", midi: 44, color: "#80cbc4", size: "small", key: "G" },
      { label: "6", midi: 46, color: "#9fa8da", size: "small", key: "H" },
    ],
  },
};

function getDrumConfig(instrument: Instrument): DrumConfig {
  const name = instrument.name.toLowerCase();
  if (name.includes("djembe")) return DRUM_CONFIGS.djembe;
  if (name.includes("tabla")) return DRUM_CONFIGS.tabla;
  if (name.includes("kpanlogo")) return DRUM_CONFIGS.kpanlogo;
  return DRUM_CONFIGS.default_percussion;
}

// ── String fretboard interface ────────────────────────────────────────────────

function StringInterface({
  instrument,
  octave,
  activeNotes,
  onNoteOn,
  onNoteOff,
}: InterfaceProps) {
  const config = getStringConfig(instrument);
  const strings = Math.min(config.strings, 12); // cap for display
  const frets = Math.min(config.frets, 16);

  return (
    <div style={{ padding: "16px 20px" }}>
      <div
        style={{
          fontSize: 11,
          color: "#4a6a8a",
          marginBottom: 12,
          letterSpacing: 1,
        }}
      >
        {config.label.toUpperCase()}
      </div>
      <div
        style={{
          background: "#0a1520",
          border: "1px solid #1a2a3a",
          borderRadius: 8,
          overflow: "hidden",
          position: "relative",
        }}
      >
        {/* Fret position markers */}
        <div style={{ display: "flex", borderBottom: "1px solid #1a2a3a" }}>
          <div style={{ width: 48, flexShrink: 0 }} />
          {Array.from({ length: frets }, (_, f) => (
            <div
              key={f}
              style={{
                flex: 1,
                textAlign: "center",
                fontSize: 9,
                color: [3, 5, 7, 9, 12].includes(f + 1) ? "#4a6a8a" : "#1a2a3a",
                padding: "3px 0",
                borderLeft: "1px solid #1a2a3a",
              }}
            >
              {[3, 5, 7, 9, 12].includes(f + 1) ? "●" : f + 1}
            </div>
          ))}
        </div>

        {/* Strings */}
        {Array.from({ length: strings }, (_, s) => {
          const openNote = (config.tuning[s] ?? 48) + octave * 12 - 36;
          return (
            <div
              key={s}
              style={{
                display: "flex",
                alignItems: "center",
                borderBottom: s < strings - 1 ? "1px solid #0f1e2e" : "none",
                position: "relative",
              }}
            >
              {/* Open string button */}
              <div
                style={{
                  width: 48,
                  flexShrink: 0,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: "6px 0",
                  cursor: "pointer",
                  background: activeNotes.has(openNote)
                    ? "rgba(212,160,23,0.2)"
                    : "transparent",
                }}
                onMouseDown={() => onNoteOn(openNote)}
                onMouseUp={() => onNoteOff(openNote)}
                onMouseLeave={() =>
                  activeNotes.has(openNote) && onNoteOff(openNote)
                }
              >
                <div
                  style={{
                    width: 20,
                    height: 20,
                    borderRadius: "50%",
                    background: activeNotes.has(openNote)
                      ? config.color
                      : "#1a2a3a",
                    border: `1px solid ${activeNotes.has(openNote) ? config.color : "#2a3a4a"}`,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontSize: 8,
                    color: activeNotes.has(openNote) ? "#000" : "#4a6a8a",
                    transition: "all 0.08s",
                  }}
                >
                  O
                </div>
              </div>

              {/* String line */}
              <div
                style={{
                  position: "absolute",
                  left: 48,
                  right: 0,
                  top: "50%",
                  height: Math.max(1, 3 - s * 0.3),
                  background: `rgba(${200 - s * 15}, ${180 - s * 10}, ${100 - s * 5}, 0.4)`,
                  pointerEvents: "none",
                }}
              />

              {/* Fret buttons */}
              {Array.from({ length: frets }, (_, f) => {
                const midi = openNote + f + 1;
                const isActive = activeNotes.has(midi);
                return (
                  <div
                    key={f}
                    style={{
                      flex: 1,
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      padding: "6px 2px",
                      borderLeft: "1px solid #1a2a3a",
                      cursor: "pointer",
                      position: "relative",
                      zIndex: 1,
                    }}
                    onMouseDown={() => onNoteOn(midi)}
                    onMouseUp={() => onNoteOff(midi)}
                    onMouseLeave={() => isActive && onNoteOff(midi)}
                  >
                    {isActive && (
                      <div
                        style={{
                          width: 22,
                          height: 22,
                          borderRadius: "50%",
                          background: config.color,
                          boxShadow: `0 0 10px ${config.color}`,
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "center",
                          fontSize: 8,
                          color: "#000",
                          fontWeight: "bold",
                        }}
                      >
                        {midi % 12 === 0 ? "C" : ""}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          );
        })}
      </div>
      <div style={{ marginTop: 8, fontSize: 10, color: "#2a3a4a" }}>
        Click frets to play · Open string = leftmost column
      </div>
    </div>
  );
}

// ── Drum pad interface ────────────────────────────────────────────────────────

function DrumInterface({
  instrument,
  activeNotes,
  onNoteOn,
  onNoteOff,
}: InterfaceProps) {
  const config = getDrumConfig(instrument);

  const padSize = (size?: string) => {
    switch (size) {
      case "large":
        return 100;
      case "medium":
        return 80;
      default:
        return 64;
    }
  };

  if (config.layout === "circle") {
    // Djembe-style: large center pad + smaller pads around it
    const [center, ...rest] = config.pads;
    return (
      <div
        style={{
          padding: "20px",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 16,
        }}
      >
        <div style={{ fontSize: 11, color: "#4a6a8a", letterSpacing: 1 }}>
          {config.label.toUpperCase()}
        </div>
        {/* Center pad */}
        <DrumPadButton
          pad={center}
          activeNotes={activeNotes}
          onNoteOn={onNoteOn}
          onNoteOff={onNoteOff}
          size={120}
        />
        {/* Surrounding pads */}
        <div
          style={{
            display: "flex",
            gap: 12,
            flexWrap: "wrap",
            justifyContent: "center",
          }}
        >
          {rest.map((pad) => (
            <DrumPadButton
              key={pad.midi}
              pad={pad}
              activeNotes={activeNotes}
              onNoteOn={onNoteOn}
              onNoteOff={onNoteOff}
              size={padSize(pad.size)}
            />
          ))}
        </div>
      </div>
    );
  }

  if (config.layout === "tabla") {
    // Tabla: two drum heads side by side
    const dayan = config.pads.slice(0, 3); // right hand (treble)
    const bayan = config.pads.slice(3); // left hand (bass)
    return (
      <div
        style={{
          padding: "20px",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 16,
        }}
      >
        <div style={{ fontSize: 11, color: "#4a6a8a", letterSpacing: 1 }}>
          {config.label.toUpperCase()}
        </div>
        <div style={{ display: "flex", gap: 32, alignItems: "flex-end" }}>
          {/* Dayan (right, treble) */}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 8,
            }}
          >
            <div style={{ fontSize: 10, color: "#4a6a8a" }}>Dayan (treble)</div>
            <div
              style={{
                width: 120,
                height: 120,
                borderRadius: "50%",
                background: "#0d1a26",
                border: "3px solid #2a3a4a",
                display: "flex",
                flexWrap: "wrap",
                alignItems: "center",
                justifyContent: "center",
                gap: 4,
                padding: 8,
                position: "relative",
              }}
            >
              {dayan.map((pad) => (
                <DrumPadButton
                  key={pad.midi}
                  pad={pad}
                  activeNotes={activeNotes}
                  onNoteOn={onNoteOn}
                  onNoteOff={onNoteOff}
                  size={32}
                  compact
                />
              ))}
            </div>
          </div>
          {/* Bayan (left, bass) */}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 8,
            }}
          >
            <div style={{ fontSize: 10, color: "#4a6a8a" }}>Bayan (bass)</div>
            <div
              style={{
                width: 150,
                height: 150,
                borderRadius: "50%",
                background: "#0d1a26",
                border: "3px solid #2a3a4a",
                display: "flex",
                flexWrap: "wrap",
                alignItems: "center",
                justifyContent: "center",
                gap: 4,
                padding: 12,
              }}
            >
              {bayan.map((pad) => (
                <DrumPadButton
                  key={pad.midi}
                  pad={pad}
                  activeNotes={activeNotes}
                  onNoteOn={onNoteOn}
                  onNoteOff={onNoteOff}
                  size={40}
                  compact
                />
              ))}
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Default grid layout
  return (
    <div
      style={{
        padding: "20px",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 16,
      }}
    >
      <div style={{ fontSize: 11, color: "#4a6a8a", letterSpacing: 1 }}>
        {config.label.toUpperCase()}
      </div>
      <div
        style={{
          display: "flex",
          gap: 12,
          flexWrap: "wrap",
          justifyContent: "center",
        }}
      >
        {config.pads.map((pad) => (
          <DrumPadButton
            key={pad.midi}
            pad={pad}
            activeNotes={activeNotes}
            onNoteOn={onNoteOn}
            onNoteOff={onNoteOff}
            size={padSize(pad.size)}
          />
        ))}
      </div>
    </div>
  );
}

interface DrumPadButtonProps {
  pad: DrumPad;
  activeNotes: Set<number>;
  onNoteOn: (midi: number) => void;
  onNoteOff: (midi: number) => void;
  size: number;
  compact?: boolean;
}

function DrumPadButton({
  pad,
  activeNotes,
  onNoteOn,
  onNoteOff,
  size,
  compact,
}: DrumPadButtonProps) {
  const isActive = activeNotes.has(pad.midi);
  return (
    <div
      onMouseDown={() => onNoteOn(pad.midi)}
      onMouseUp={() => onNoteOff(pad.midi)}
      onMouseLeave={() => isActive && onNoteOff(pad.midi)}
      onTouchStart={(e) => {
        e.preventDefault();
        onNoteOn(pad.midi);
      }}
      onTouchEnd={(e) => {
        e.preventDefault();
        onNoteOff(pad.midi);
      }}
      style={{
        width: size,
        height: size,
        borderRadius: compact ? "50%" : 12,
        background: isActive ? pad.color : `rgba(${hexToRgb(pad.color)}, 0.12)`,
        border: `2px solid ${isActive ? pad.color : `rgba(${hexToRgb(pad.color)}, 0.3)`}`,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        cursor: "pointer",
        userSelect: "none",
        transition: "all 0.06s",
        boxShadow: isActive ? `0 0 20px ${pad.color}60` : "none",
        transform: isActive ? "scale(0.96)" : "scale(1)",
      }}
    >
      {!compact && (
        <>
          <span
            style={{
              fontSize: size > 80 ? 14 : 11,
              fontWeight: "bold",
              color: isActive ? "#000" : pad.color,
            }}
          >
            {pad.label}
          </span>
          {pad.key && (
            <span
              style={{
                fontSize: 9,
                color: isActive ? "rgba(0,0,0,0.5)" : "#2a3a4a",
                fontFamily: "monospace",
                marginTop: 2,
              }}
            >
              {pad.key}
            </span>
          )}
        </>
      )}
      {compact && (
        <span
          style={{
            fontSize: 9,
            fontWeight: "bold",
            color: isActive ? "#000" : pad.color,
          }}
        >
          {pad.label}
        </span>
      )}
    </div>
  );
}

function hexToRgb(hex: string): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `${r}, ${g}, ${b}`;
}

// ── Wind breath pad ───────────────────────────────────────────────────────────

function WindInterface({
  instrument,
  octave,
  velocity,
  activeNotes,
  onNoteOn,
  onNoteOff,
}: InterfaceProps) {
  const [heldNote, setHeldNote] = useState<number | null>(null);
  const pressRef = useRef<number>(0);

  // Pentatonic scale for wind instruments
  const scale = [0, 2, 4, 7, 9, 12, 14, 16, 19, 21];
  const rootMidi = 12 * (octave + 1) + 48;

  const handlePress = useCallback(
    (midi: number) => {
      if (heldNote !== null) onNoteOff(heldNote);
      setHeldNote(midi);
      onNoteOn(midi);
      pressRef.current = Date.now();
    },
    [heldNote, onNoteOn, onNoteOff],
  );

  const handleRelease = useCallback(
    (midi: number) => {
      setHeldNote(null);
      onNoteOff(midi);
    },
    [onNoteOff],
  );

  const noteNames = ["C", "D", "E", "G", "A", "C", "D", "E", "G", "A"];
  const colors = [
    "#4fc3f7",
    "#81d4fa",
    "#b3e5fc",
    "#4fc3f7",
    "#81d4fa",
    "#4fc3f7",
    "#81d4fa",
    "#b3e5fc",
    "#4fc3f7",
    "#81d4fa",
  ];

  return (
    <div
      style={{
        padding: "20px",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 16,
      }}
    >
      <div style={{ fontSize: 11, color: "#4a6a8a", letterSpacing: 1 }}>
        {instrument.name.toUpperCase()} — BREATH PAD
      </div>
      <div style={{ fontSize: 10, color: "#2a3a4a" }}>
        Hold to sustain · Pentatonic scale · Velocity {velocity}
      </div>
      <div
        style={{
          display: "flex",
          gap: 8,
          flexWrap: "wrap",
          justifyContent: "center",
          maxWidth: 480,
        }}
      >
        {scale.map((offset, i) => {
          const midi = rootMidi + offset;
          const isActive = activeNotes.has(midi);
          return (
            <div
              key={i}
              onMouseDown={() => handlePress(midi)}
              onMouseUp={() => handleRelease(midi)}
              onMouseLeave={() => isActive && handleRelease(midi)}
              onTouchStart={(e) => {
                e.preventDefault();
                handlePress(midi);
              }}
              onTouchEnd={(e) => {
                e.preventDefault();
                handleRelease(midi);
              }}
              style={{
                width: 72,
                height: 72,
                borderRadius: "50%",
                background: isActive
                  ? colors[i]
                  : `rgba(79, 195, 247, ${0.05 + i * 0.01})`,
                border: `2px solid ${isActive ? colors[i] : "rgba(79,195,247,0.2)"}`,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                cursor: "pointer",
                userSelect: "none",
                transition: "all 0.08s",
                boxShadow: isActive ? `0 0 24px ${colors[i]}80` : "none",
                transform: isActive ? "scale(1.08)" : "scale(1)",
              }}
            >
              <span
                style={{
                  fontSize: 16,
                  fontWeight: "bold",
                  color: isActive ? "#000" : colors[i],
                }}
              >
                {noteNames[i]}
              </span>
              <span
                style={{
                  fontSize: 9,
                  color: isActive ? "rgba(0,0,0,0.5)" : "#2a3a4a",
                }}
              >
                {midi}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

// ── Main dispatcher ───────────────────────────────────────────────────────────

/**
 * Returns true if this instrument should use a custom interface
 * instead of the standard piano keyboard.
 */
export function hasCustomInterface(instrument: Instrument): boolean {
  return (
    instrument.family === "plucked-string" ||
    instrument.family === "percussion" ||
    instrument.family === "wind"
  );
}

/**
 * Render the appropriate custom interface for an instrument.
 * Falls back to null if no custom interface is defined (caller shows piano).
 */
export function InstrumentInterface(props: InterfaceProps) {
  const { instrument } = props;

  switch (instrument.family) {
    case "plucked-string":
      return <StringInterface {...props} />;
    case "percussion":
      return <DrumInterface {...props} />;
    case "wind":
      return <WindInterface {...props} />;
    default:
      return null;
  }
}
