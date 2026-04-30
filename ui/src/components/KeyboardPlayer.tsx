/**
 * Aether Studio v2.0 - Keyboard Player Overlay
 * Full-screen instrument player with PC keyboard support.
 * Plays through the Rust audio engine via WebSocket MIDI injection,
 * with Web Audio fallback when the host is not connected.
 */

import { useState, useEffect, useRef, useCallback } from "react";
import type { Instrument } from "../catalog/types";
import { useInstrumentEngine } from "../catalog/useInstrumentEngine";
import "./KeyboardPlayer.css";

interface KeyboardPlayerProps {
  instrument: Instrument;
  onClose: () => void;
  onAddToCanvas?: () => void;
}

const WHITE_KEYS = ["A", "S", "D", "F", "G", "H", "J", "K", "L"];
const BLACK_KEYS = ["W", "E", "T", "Y", "U", "O", "P"];

// Maps keyboard key → MIDI note offset from octave root (C)
const WHITE_NOTE_OFFSETS = [0, 2, 4, 5, 7, 9, 11, 12, 14]; // C D E F G A B C D
const BLACK_NOTE_OFFSETS = [1, 3, 6, 8, 10, 13, 15]; // C# D# F# G# A# C# D#

export function KeyboardPlayer({
  instrument,
  onClose,
  onAddToCanvas,
}: KeyboardPlayerProps) {
  const [octave, setOctave] = useState(3);
  const [velocity, setVelocity] = useState(80);
  const [activeKeys, setActiveKeys] = useState<Set<string>>(new Set());
  const audioContextRef = useRef<AudioContext | null>(null);
  const {
    playNote: enginePlayNote,
    stopNote: engineStopNote,
    isEngineConnected,
  } = useInstrumentEngine();

  // Compute MIDI note number from key + current octave
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

  // Web Audio fallback — used when host is not connected
  const playWebAudioNote = useCallback(
    (midiNote: number) => {
      const ctx = audioContextRef.current;
      if (!ctx) return;
      const frequency = 440 * Math.pow(2, (midiNote - 69) / 12);
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.frequency.value = frequency;
      osc.type = "sine";
      const vel = velocity / 127;
      gain.gain.setValueAtTime(vel * 0.3, ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.8);
      osc.start();
      osc.stop(ctx.currentTime + 0.8);
    },
    [velocity],
  );

  const handleNoteOn = useCallback(
    (key: string) => {
      const midiNote = keyToMidi(key);
      if (midiNote === null) return;

      if (isEngineConnected) {
        // Play through Rust engine via WebSocket MIDI injection
        enginePlayNote(instrument, midiNote, velocity);
      } else {
        // Fallback to Web Audio
        playWebAudioNote(midiNote);
      }
    },
    [
      keyToMidi,
      isEngineConnected,
      enginePlayNote,
      instrument,
      velocity,
      playWebAudioNote,
    ],
  );

  const handleNoteOff = useCallback(
    (key: string) => {
      const midiNote = keyToMidi(key);
      if (midiNote === null) return;
      if (isEngineConnected) {
        engineStopNote(instrument, midiNote);
      }
    },
    [keyToMidi, isEngineConnected, engineStopNote, instrument],
  );

  useEffect(() => {
    audioContextRef.current = new AudioContext();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.repeat) return;
      const key = e.key.toUpperCase();

      if (key === "Z") {
        setOctave((o) => Math.max(0, o - 1));
        return;
      }
      if (key === "X") {
        setOctave((o) => Math.min(8, o + 1));
        return;
      }
      if (key === "ESCAPE") {
        onClose();
        return;
      }

      if (WHITE_KEYS.includes(key) || BLACK_KEYS.includes(key)) {
        handleNoteOn(key);
        setActiveKeys((keys) => new Set(keys).add(key));
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      const key = e.key.toUpperCase();
      if (WHITE_KEYS.includes(key) || BLACK_KEYS.includes(key)) {
        handleNoteOff(key);
        setActiveKeys((keys) => {
          const newKeys = new Set(keys);
          newKeys.delete(key);
          return newKeys;
        });
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      audioContextRef.current?.close();
    };
  }, [handleNoteOn, handleNoteOff, onClose]);

  const renderKeyboard = () => {
    const keys = [];
    const numWhiteKeys = 14;

    for (let i = 0; i < numWhiteKeys; i++) {
      const noteIndex = i % 7;
      const noteName = ["C", "D", "E", "F", "G", "A", "B"][noteIndex];
      const keyBinding = i < WHITE_KEYS.length ? WHITE_KEYS[i] : "";
      const isActive = activeKeys.has(keyBinding);

      keys.push(
        <div
          key={`white-${i}`}
          className={`piano-key white-key ${isActive ? "active" : ""}`}
          onMouseDown={() => keyBinding && handleNoteOn(keyBinding)}
          onMouseUp={() => keyBinding && handleNoteOff(keyBinding)}
          onMouseLeave={() =>
            keyBinding &&
            activeKeys.has(keyBinding) &&
            handleNoteOff(keyBinding)
          }
        >
          <span className="key-label">{noteName}</span>
          <span className="key-binding">{keyBinding}</span>
        </div>,
      );
    }

    const blackKeyPositions = [0, 1, 3, 4, 5, 7, 8, 10, 11, 12];
    blackKeyPositions.forEach((pos, idx) => {
      const keyBinding = idx < BLACK_KEYS.length ? BLACK_KEYS[idx] : "";
      const isActive = activeKeys.has(keyBinding);

      keys.push(
        <div
          key={`black-${pos}`}
          className={`piano-key black-key ${isActive ? "active" : ""}`}
          style={{ left: `${(pos + 0.7) * (100 / numWhiteKeys)}%` }}
          onMouseDown={() => keyBinding && handleNoteOn(keyBinding)}
          onMouseUp={() => keyBinding && handleNoteOff(keyBinding)}
          onMouseLeave={() =>
            keyBinding &&
            activeKeys.has(keyBinding) &&
            handleNoteOff(keyBinding)
          }
        >
          <span className="key-binding">{keyBinding}</span>
        </div>,
      );
    });

    return keys;
  };

  return (
    <div className="keyboard-player-overlay">
      <div className="keyboard-player-content">
        {/* Header */}
        <div className="keyboard-player-header">
          <div className="instrument-info">
            <span className="instrument-flag">{instrument.flag}</span>
            <span className="instrument-name">{instrument.name}</span>
            <span className="instrument-tuning">• {instrument.tuning}</span>
          </div>
          <div className="header-right">
            <span
              className={`engine-status ${isEngineConnected ? "connected" : "fallback"}`}
            >
              {isEngineConnected ? "🟢 Engine" : "🟡 Web Audio"}
            </span>
            <button className="close-btn" onClick={onClose} title="Close (ESC)">
              ✕
            </button>
          </div>
        </div>

        {/* Piano Keyboard */}
        <div className="piano-keyboard-container">
          <div className="piano-keyboard">{renderKeyboard()}</div>
        </div>

        {/* Controls */}
        <div className="keyboard-controls">
          <div className="control-group">
            <label>Octave</label>
            <div className="octave-control">
              <button onClick={() => setOctave((o) => Math.max(0, o - 1))}>
                ◀
              </button>
              <span className="octave-value">{octave}</span>
              <button onClick={() => setOctave((o) => Math.min(8, o + 1))}>
                ▶
              </button>
            </div>
            <span className="hint">Z / X</span>
          </div>

          <div className="control-group">
            <label>Velocity</label>
            <input
              type="range"
              min="0"
              max="127"
              value={velocity}
              onChange={(e) => setVelocity(Number(e.target.value))}
              className="velocity-slider"
            />
            <span className="velocity-value">{velocity}</span>
          </div>
        </div>

        {/* Visualizations */}
        <div className="visualizations">
          <div className="waveform">
            <div className="waveform-label">Waveform</div>
            <div className="waveform-canvas">
              <svg width="100%" height="60" viewBox="0 0 400 60">
                <path
                  d="M0,30 Q50,10 100,30 T200,30 T300,30 T400,30"
                  stroke="var(--accent-primary)"
                  strokeWidth="2"
                  fill="none"
                  opacity="0.6"
                />
              </svg>
            </div>
          </div>

          <div className="spectrum">
            <div className="spectrum-label">Spectrum</div>
            <div className="spectrum-bars">
              {Array.from({ length: 32 }).map((_, i) => (
                <div
                  key={i}
                  className="spectrum-bar"
                  style={{ height: `${Math.random() * 100}%` }}
                />
              ))}
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="keyboard-actions">
          {onAddToCanvas && (
            <button className="action-btn primary" onClick={onAddToCanvas}>
              Add to Canvas
            </button>
          )}
          <button className="action-btn secondary">Record Performance</button>
        </div>
      </div>
    </div>
  );
}
