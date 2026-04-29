/**
 * Aether Studio v2.0 - Keyboard Player Overlay
 * Full-screen instrument player with PC keyboard support
 */

import { useState, useEffect, useRef } from "react";
import type { Instrument } from "../catalog/types";
import "./KeyboardPlayer.css";

interface KeyboardPlayerProps {
  instrument: Instrument;
  onClose: () => void;
  onAddToCanvas?: () => void;
}

const WHITE_KEYS = ["A", "S", "D", "F", "G", "H", "J", "K", "L"];
const BLACK_KEYS = ["W", "E", "T", "Y", "U", "O", "P"];

export function KeyboardPlayer({
  instrument,
  onClose,
  onAddToCanvas,
}: KeyboardPlayerProps) {
  const [octave, setOctave] = useState(3);
  const [velocity, setVelocity] = useState(80);
  const [activeKeys, setActiveKeys] = useState<Set<string>>(new Set());
  const audioContextRef = useRef<AudioContext | null>(null);

  useEffect(() => {
    // Initialize Web Audio
    audioContextRef.current = new AudioContext();

    // Keyboard event handlers
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.repeat) return;

      const key = e.key.toUpperCase();

      // Octave controls
      if (key === "Z") {
        setOctave((o) => Math.max(0, o - 1));
        return;
      }
      if (key === "X") {
        setOctave((o) => Math.min(8, o + 1));
        return;
      }

      // Close on ESC
      if (key === "ESCAPE") {
        onClose();
        return;
      }

      // Play note
      if (WHITE_KEYS.includes(key) || BLACK_KEYS.includes(key)) {
        playNote(key);
        setActiveKeys((keys) => new Set(keys).add(key));
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      const key = e.key.toUpperCase();
      setActiveKeys((keys) => {
        const newKeys = new Set(keys);
        newKeys.delete(key);
        return newKeys;
      });
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      audioContextRef.current?.close();
    };
  }, [octave, onClose]);

  const playNote = (key: string) => {
    const ctx = audioContextRef.current;
    if (!ctx) return;

    // Map key to MIDI note
    let noteOffset = 0;
    if (WHITE_KEYS.includes(key)) {
      const whiteNoteOffsets = [0, 2, 4, 5, 7, 9, 11, 12, 14]; // C D E F G A B C D
      noteOffset = whiteNoteOffsets[WHITE_KEYS.indexOf(key)];
    } else if (BLACK_KEYS.includes(key)) {
      const blackNoteOffsets = [1, 3, 6, 8, 10, 13, 15]; // C# D# F# G# A# C# D#
      noteOffset = blackNoteOffsets[BLACK_KEYS.indexOf(key)];
    }

    const midiNote = 12 * (octave + 1) + noteOffset;
    const frequency = 440 * Math.pow(2, (midiNote - 69) / 12);

    // Simple synth (will be replaced with actual instrument samples)
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.connect(gain);
    gain.connect(ctx.destination);

    osc.frequency.value = frequency;
    osc.type = "sine";

    const vel = velocity / 127;
    gain.gain.setValueAtTime(vel * 0.3, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.5);

    osc.start();
    osc.stop(ctx.currentTime + 0.5);
  };

  const renderKeyboard = () => {
    const keys = [];
    const numWhiteKeys = 14; // 2 octaves

    for (let i = 0; i < numWhiteKeys; i++) {
      const noteIndex = i % 7;
      const noteName = ["C", "D", "E", "F", "G", "A", "B"][noteIndex];
      const keyBinding = i < WHITE_KEYS.length ? WHITE_KEYS[i] : "";
      const isActive = activeKeys.has(keyBinding);

      keys.push(
        <div
          key={`white-${i}`}
          className={`piano-key white-key ${isActive ? "active" : ""}`}
          onClick={() => keyBinding && playNote(keyBinding)}
        >
          <span className="key-label">{noteName}</span>
          <span className="key-binding">{keyBinding}</span>
        </div>,
      );
    }

    // Black keys
    const blackKeyPositions = [0, 1, 3, 4, 5, 7, 8, 10, 11, 12]; // Positions between white keys
    blackKeyPositions.forEach((pos, idx) => {
      const keyBinding = idx < BLACK_KEYS.length ? BLACK_KEYS[idx] : "";
      const isActive = activeKeys.has(keyBinding);

      keys.push(
        <div
          key={`black-${pos}`}
          className={`piano-key black-key ${isActive ? "active" : ""}`}
          style={{ left: `${(pos + 0.7) * (100 / numWhiteKeys)}%` }}
          onClick={() => keyBinding && playNote(keyBinding)}
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
          <button className="close-btn" onClick={onClose} title="Close (ESC)">
            ✕
          </button>
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
              {/* Placeholder waveform */}
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
