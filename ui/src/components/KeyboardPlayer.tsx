/**
 * Aether Studio — Keyboard Player
 *
 * Full-screen immersive instrument player.
 * Each instrument family gets a custom playing interface.
 * Plays through the Rust engine via WebSocket MIDI injection,
 * falls back to Web Audio when host is not connected.
 *
 * Microtonal visualization: when the instrument uses a non-12-TET tuning,
 * each key shows a cent-deviation indicator (colored bar + tooltip).
 * Keys that are flat show a blue tint; sharp keys show a warm amber tint.
 */

import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import type { Instrument } from "../catalog/types";
import { useInstrumentEngine } from "../catalog/useInstrumentEngine";
import { getCountryFlag } from "../catalog/countryFlags";
import { InstrumentInterface, hasCustomInterface } from "./InstrumentInterface";
import "./KeyboardPlayer.css";

interface KeyboardPlayerProps {
  instrument: Instrument;
  onClose: () => void;
  onAddToCanvas?: () => void;
}

// PC keyboard → MIDI note mapping
const WHITE_KEYS = ["A", "S", "D", "F", "G", "H", "J", "K", "L"];
const BLACK_KEYS = ["W", "E", "T", "Y", "U", "O", "P"];
const WHITE_NOTE_OFFSETS = [0, 2, 4, 5, 7, 9, 11, 12, 14];
const BLACK_NOTE_OFFSETS = [1, 3, 6, 8, 10, 13, 15];

// ── Microtonal tuning utilities ───────────────────────────────────────────────

/**
 * Tuning systems with their cent deviations from 12-TET per pitch class.
 * Index 0 = C, 1 = C#, ..., 11 = B.
 */
const TUNING_CENTS: Record<string, number[]> = {
  "arabic-maqam-rast": [0, 0, 0, -50, 0, 0, 0, 0, 0, 0, -50, 0],
  "arabic-maqam-bayati": [0, -50, 0, -30, 0, 0, 0, 0, 0, 0, -50, 0],
  "ethiopian-tizita": [0, -50, 0, -30, 0, 0, -20, 0, -40, 0, -30, 0],
  "ethiopian-bati": [0, 0, -20, 0, 0, 0, -30, 0, 0, -20, 0, 0],
  "indian-raga-yaman": [0, 0, 3.9, 0, -13.7, 0, -9.8, 2.0, 0, -15.6, 0, -11.7],
  "gamelan-slendro": [0, 0, -40, 0, -20, 0, 0, -30, 0, -10, 0, 0],
  "gamelan-pelog": [0, -80, 0, -30, 0, 0, -60, 0, -20, 0, 0, -50],
  "just-intonation": [
    0, 11.7, 3.9, 15.6, -13.7, -2.0, -9.8, 2.0, 13.7, -15.6, -17.6, -11.7,
  ],
};

/**
 * Get cent deviation for a MIDI note given a tuning name.
 * Returns 0 for 12-TET or unknown tunings.
 */
function getCentDeviation(
  midiNote: number,
  tuningName: string | undefined,
): number {
  if (!tuningName) return 0;
  const key = tuningName.toLowerCase().replace(/\s+/g, "-");
  const cents = TUNING_CENTS[key];
  if (!cents) return 0;
  return cents[midiNote % 12];
}

/**
 * Color for a cent deviation indicator.
 * Flat (negative) → blue; sharp (positive) → amber; near-zero → transparent.
 */
function centDeviationColor(cents: number): string {
  const abs = Math.abs(cents);
  if (abs < 5) return "transparent";
  const alpha = Math.min(abs / 50, 1) * 0.85;
  if (cents < 0) {
    // Flat: blue
    return `rgba(56, 189, 248, ${alpha})`;
  } else {
    // Sharp: amber
    return `rgba(251, 191, 36, ${alpha})`;
  }
}

export function KeyboardPlayer({
  instrument,
  onClose,
  onAddToCanvas,
}: KeyboardPlayerProps) {
  const [octave, setOctave] = useState(3);
  const [velocity, setVelocity] = useState(90);
  const [activeNotes, setActiveNotes] = useState<Set<number>>(new Set());
  const audioCtxRef = useRef<AudioContext | null>(null);
  const { playNote, stopNote, isEngineConnected } = useInstrumentEngine();

  // Precompute cent deviations for all MIDI notes in the visible range
  const centDeviations = useMemo(() => {
    const map: Record<number, number> = {};
    for (let midi = 0; midi < 128; midi++) {
      map[midi] = getCentDeviation(midi, instrument.tuning);
    }
    return map;
  }, [instrument.tuning]);

  const hasMicrotonalTuning = useMemo(() => {
    return Object.values(centDeviations).some((c) => Math.abs(c) > 5);
  }, [centDeviations]);

  const keyToMidi = useCallback(
    (key: string): number | null => {
      if (WHITE_KEYS.includes(key)) {
        return 12 * (octave + 1) + WHITE_NOTE_OFFSETS[WHITE_KEYS.indexOf(key)];
      }
      if (BLACK_KEYS.includes(key)) {
        return 12 * (octave + 1) + BLACK_NOTE_OFFSETS[BLACK_KEYS.indexOf(key)];
      }
      return null;
    },
    [octave],
  );

  const playWebAudio = useCallback(
    (midi: number) => {
      if (!audioCtxRef.current) audioCtxRef.current = new AudioContext();
      const ctx = audioCtxRef.current;
      const freq = 440 * Math.pow(2, (midi - 69) / 12);
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      const filter = ctx.createBiquadFilter();

      // Shape the sound based on instrument family
      const family = instrument.family;
      if (family === "bowed-string") {
        osc.type = "sawtooth";
        filter.type = "lowpass";
        filter.frequency.value = 2000;
      } else if (family === "wind") {
        osc.type = "sine";
        filter.type = "bandpass";
        filter.frequency.value = freq * 2;
        filter.Q.value = 2;
      } else if (family === "percussion") {
        osc.type = "triangle";
        filter.type = "highpass";
        filter.frequency.value = 200;
      } else {
        osc.type = "triangle";
        filter.type = "lowpass";
        filter.frequency.value = 3000;
      }

      osc.connect(filter);
      filter.connect(gain);
      gain.connect(ctx.destination);
      osc.frequency.value = freq;

      const vel = velocity / 127;
      gain.gain.setValueAtTime(0, ctx.currentTime);
      gain.gain.linearRampToValueAtTime(vel * 0.4, ctx.currentTime + 0.02);
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 1.2);

      osc.start();
      osc.stop(ctx.currentTime + 1.2);
    },
    [velocity, instrument.family],
  );

  const handleNoteOn = useCallback(
    (midi: number) => {
      setActiveNotes((n) => new Set(n).add(midi));
      if (isEngineConnected) {
        playNote(instrument, midi, velocity);
      } else {
        playWebAudio(midi);
      }
    },
    [isEngineConnected, playNote, instrument, velocity, playWebAudio],
  );

  const handleNoteOff = useCallback(
    (midi: number) => {
      setActiveNotes((n) => {
        const s = new Set(n);
        s.delete(midi);
        return s;
      });
      if (isEngineConnected) {
        stopNote(instrument, midi);
      }
    },
    [isEngineConnected, stopNote, instrument],
  );

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.repeat) return;
      const key = e.key.toUpperCase();
      if (key === "ESCAPE") {
        onClose();
        return;
      }
      if (key === "Z") {
        setOctave((o) => Math.max(0, o - 1));
        return;
      }
      if (key === "X") {
        setOctave((o) => Math.min(8, o + 1));
        return;
      }
      if (WHITE_KEYS.includes(key) || BLACK_KEYS.includes(key)) {
        const midi = keyToMidi(key);
        if (midi !== null) {
          handleNoteOn(midi);
        }
      }
    };
    const up = (e: KeyboardEvent) => {
      const key = e.key.toUpperCase();
      if (WHITE_KEYS.includes(key) || BLACK_KEYS.includes(key)) {
        const midi = keyToMidi(key);
        if (midi !== null) handleNoteOff(midi);
      }
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
      audioCtxRef.current?.close();
    };
  }, [keyToMidi, handleNoteOn, handleNoteOff, onClose]);

  const flag = getCountryFlag(instrument.country);
  const showCustomInterface = hasCustomInterface(instrument);
  const [useCustomUI, setUseCustomUI] = useState(showCustomInterface);

  return (
    <div className="kp-overlay">
      <div className="kp-backdrop" onClick={onClose} />
      <div className="kp-panel animate-scale-in">
        {/* Header */}
        <div className="kp-header">
          <div className="kp-instrument-info">
            <span className="kp-flag">{flag}</span>
            <div>
              <div className="kp-instrument-name">{instrument.name}</div>
              <div className="kp-instrument-meta">
                {instrument.country} · {instrument.tuning?.replace(/-/g, " ")}
              </div>
            </div>
          </div>
          <div className="kp-header-right">
            <div
              className={`kp-engine-badge ${isEngineConnected ? "connected" : "fallback"}`}
            >
              <span className="kp-engine-dot" />
              {isEngineConnected ? "Engine" : "Web Audio"}
            </div>
            {showCustomInterface && (
              <button
                onClick={() => setUseCustomUI((v) => !v)}
                style={{
                  padding: "4px 10px",
                  background: useCustomUI
                    ? "rgba(212,160,23,0.15)"
                    : "rgba(255,255,255,0.05)",
                  border: `1px solid ${useCustomUI ? "rgba(212,160,23,0.4)" : "#1e2d3d"}`,
                  borderRadius: 6,
                  color: useCustomUI ? "#d4a017" : "#64748b",
                  fontSize: 11,
                  cursor: "pointer",
                  fontFamily: "'Inter', system-ui, sans-serif",
                  fontWeight: 600,
                  transition: "all 0.15s",
                }}
                title="Toggle between custom instrument interface and piano keyboard"
              >
                {useCustomUI ? "🎸 Custom" : "🎹 Piano"}
              </button>
            )}
            <button className="kp-close" onClick={onClose} title="Close (ESC)">
              ✕
            </button>
          </div>
        </div>

        {/* Custom instrument interface OR piano keyboard */}
        {useCustomUI && showCustomInterface ? (
          <InstrumentInterface
            instrument={instrument}
            octave={octave}
            velocity={velocity}
            activeNotes={activeNotes}
            onNoteOn={handleNoteOn}
            onNoteOff={handleNoteOff}
          />
        ) : (
          <div className="kp-keyboard-container">
            {hasMicrotonalTuning && (
              <div className="kp-tuning-legend">
                <span className="kp-legend-item">
                  <span
                    className="kp-legend-dot"
                    style={{ background: "rgba(56,189,248,0.7)" }}
                  />
                  Flat (↓ cents)
                </span>
                <span className="kp-legend-item">
                  <span
                    className="kp-legend-dot"
                    style={{ background: "rgba(251,191,36,0.7)" }}
                  />
                  Sharp (↑ cents)
                </span>
                <span className="kp-legend-label">
                  {instrument.tuning?.replace(/-/g, " ")} tuning
                </span>
              </div>
            )}
            <div className="kp-keyboard">
              {/* White keys */}
              {Array.from({ length: 14 }).map((_, i) => {
                const noteOffset =
                  WHITE_NOTE_OFFSETS[i % WHITE_NOTE_OFFSETS.length];
                const midi = 12 * (octave + 1) + noteOffset + (i >= 9 ? 12 : 0);
                const keyBinding = i < WHITE_KEYS.length ? WHITE_KEYS[i] : "";
                const isActive = activeNotes.has(midi);
                const cents = centDeviations[midi] ?? 0;
                const deviationColor = centDeviationColor(cents);
                const showDeviation = Math.abs(cents) > 5;
                return (
                  <div
                    key={i}
                    className={`kp-white-key ${isActive ? "active" : ""}`}
                    onMouseDown={() => handleNoteOn(midi)}
                    onMouseUp={() => handleNoteOff(midi)}
                    onMouseLeave={() =>
                      activeNotes.has(midi) && handleNoteOff(midi)
                    }
                    title={
                      showDeviation
                        ? `${cents > 0 ? "+" : ""}${cents.toFixed(1)}¢`
                        : undefined
                    }
                  >
                    {/* Microtonal deviation indicator */}
                    {showDeviation && (
                      <div
                        className="kp-cent-bar"
                        style={{
                          background: deviationColor,
                          height: `${Math.min((Math.abs(cents) / 50) * 60, 60)}%`,
                        }}
                      />
                    )}
                    {isActive && <div className="key-ripple" />}
                    <span className="key-note">
                      {["C", "D", "E", "F", "G", "A", "B"][i % 7]}
                    </span>
                    <span className="key-binding">{keyBinding}</span>
                  </div>
                );
              })}
              {/* Black keys */}
              {[0, 1, 3, 4, 5, 7, 8, 10, 11, 12].map((pos, idx) => {
                const noteOffset =
                  BLACK_NOTE_OFFSETS[idx % BLACK_NOTE_OFFSETS.length];
                const midi =
                  12 * (octave + 1) + noteOffset + (idx >= 7 ? 12 : 0);
                const keyBinding =
                  idx < BLACK_KEYS.length ? BLACK_KEYS[idx] : "";
                const isActive = activeNotes.has(midi);
                const cents = centDeviations[midi] ?? 0;
                const deviationColor = centDeviationColor(cents);
                const showDeviation = Math.abs(cents) > 5;
                return (
                  <div
                    key={`b${pos}`}
                    className={`kp-black-key ${isActive ? "active" : ""}`}
                    style={{ left: `${(pos + 0.7) * (100 / 14)}%` }}
                    onMouseDown={() => handleNoteOn(midi)}
                    onMouseUp={() => handleNoteOff(midi)}
                    onMouseLeave={() =>
                      activeNotes.has(midi) && handleNoteOff(midi)
                    }
                    title={
                      showDeviation
                        ? `${cents > 0 ? "+" : ""}${cents.toFixed(1)}¢`
                        : undefined
                    }
                  >
                    {/* Microtonal deviation indicator on black key */}
                    {showDeviation && (
                      <div
                        className="kp-cent-bar-black"
                        style={{ background: deviationColor }}
                      />
                    )}
                    {isActive && <div className="key-ripple" />}
                    <span className="key-binding">{keyBinding}</span>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Controls */}
        <div className="kp-controls">
          <div className="kp-control-group">
            <label className="kp-label">Octave</label>
            <div className="kp-octave">
              <button onClick={() => setOctave((o) => Math.max(0, o - 1))}>
                ◀
              </button>
              <span className="kp-octave-value">{octave}</span>
              <button onClick={() => setOctave((o) => Math.min(8, o + 1))}>
                ▶
              </button>
            </div>
            <span className="kp-hint">Z / X</span>
          </div>

          <div className="kp-control-group">
            <label className="kp-label">Velocity</label>
            <input
              type="range"
              min="1"
              max="127"
              value={velocity}
              onChange={(e) => setVelocity(Number(e.target.value))}
              className="kp-slider"
            />
            <span className="kp-value">{velocity}</span>
          </div>

          <div className="kp-control-group kp-keyboard-map">
            <label className="kp-label">Keyboard Map</label>
            <div className="kp-key-map">
              <span className="kp-map-row">
                White: <code>A S D F G H J K L</code>
              </span>
              <span className="kp-map-row">
                Black: <code>W E · T Y U · O P</code>
              </span>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="kp-actions">
          {onAddToCanvas && (
            <button className="kp-action-btn primary" onClick={onAddToCanvas}>
              Add to Canvas
            </button>
          )}
          <button className="kp-action-btn secondary" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
