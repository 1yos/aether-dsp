/**
 * Aether Studio — Keyboard Player
 *
 * Full-screen immersive instrument player.
 * Each instrument family gets a custom playing interface.
 * Plays through the Rust engine via WebSocket MIDI injection,
 * falls back to Web Audio when host is not connected.
 */

import { useState, useEffect, useRef, useCallback } from "react";
import type { Instrument } from "../catalog/types";
import { useInstrumentEngine } from "../catalog/useInstrumentEngine";
import { getCountryFlag } from "../catalog/countryFlags";
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
            <button className="kp-close" onClick={onClose} title="Close (ESC)">
              ✕
            </button>
          </div>
        </div>

        {/* Piano keyboard */}
        <div className="kp-keyboard-container">
          <div className="kp-keyboard">
            {/* White keys */}
            {Array.from({ length: 14 }).map((_, i) => {
              const noteOffset =
                WHITE_NOTE_OFFSETS[i % WHITE_NOTE_OFFSETS.length];
              const midi = 12 * (octave + 1) + noteOffset + (i >= 9 ? 12 : 0);
              const keyBinding = i < WHITE_KEYS.length ? WHITE_KEYS[i] : "";
              const isActive = activeNotes.has(midi);
              return (
                <div
                  key={i}
                  className={`kp-white-key ${isActive ? "active" : ""}`}
                  onMouseDown={() => handleNoteOn(midi)}
                  onMouseUp={() => handleNoteOff(midi)}
                  onMouseLeave={() =>
                    activeNotes.has(midi) && handleNoteOff(midi)
                  }
                >
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
              const midi = 12 * (octave + 1) + noteOffset + (idx >= 7 ? 12 : 0);
              const keyBinding = idx < BLACK_KEYS.length ? BLACK_KEYS[idx] : "";
              const isActive = activeNotes.has(midi);
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
                >
                  {isActive && <div className="key-ripple" />}
                  <span className="key-binding">{keyBinding}</span>
                </div>
              );
            })}
          </div>
        </div>

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
