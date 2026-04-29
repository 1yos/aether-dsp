/**
 * Step Sequencer Module — 16-step grid with 4 tracks.
 * Emits InjectMidi intents during playback.
 * Uses Web Audio API lookahead scheduler for sample-accurate timing.
 */
import React, { useState, useCallback } from "react";
import { useEngineStore } from "../../studio/store/engineStore";
import { useTransportStore } from "../useTransportStore";
import {
  useLookaheadScheduler,
  type StepLength,
} from "../../hooks/useLookaheadScheduler";

const THEME = {
  bg: "#060c12",
  panel: "#0d1a26",
  border: "#1a2a3a",
  text: "#e0e8f0",
  textDim: "#4a6a8a",
  accent: "#4fc3f7",
  active: "#4fc3f7",
  inactive: "#0a1520",
  playing: "#ffb74d",
  stop: "#ef5350",
};

const STEPS = 16;
const TRACK_COUNT = 4;
// Default notes: C3, E3, G3, B3
const DEFAULT_NOTES = [48, 52, 55, 59];
const TRACK_NAMES = ["C3", "E3", "G3", "B3"];
const TRACK_COLORS = ["#4fc3f7", "#a5d6a7", "#ffb74d", "#ce93d8"];

type StepGrid = boolean[][];

const makeEmptyGrid = (): StepGrid =>
  Array.from({ length: TRACK_COUNT }, () => Array(STEPS).fill(false));

export const StepSequencerModule: React.FC = () => {
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const initAudioContext = useEngineStore((s) => s.initAudioContext);
  const { bpm, isPlaying, setBpm, play, stop } = useTransportStore();

  const [grid, setGrid] = useState<StepGrid>(makeEmptyGrid);
  const [stepLength, setStepLength] = useState<StepLength>("1/8");
  const [localBpm, setLocalBpm] = useState(bpm);
  const [audioContext, setAudioContext] = useState<AudioContext | null>(null);

  // Initialize lookahead scheduler
  const scheduler = useLookaheadScheduler({
    bpm: localBpm,
    stepLength,
    steps: grid,
    notes: DEFAULT_NOTES,
    sendIntent: sendIntent || (() => {}),
    audioContext: audioContext || new AudioContext(), // Fallback for initial render
  });

  const toggleStep = (track: number, step: number) => {
    setGrid((prev) => {
      const next = prev.map((row) => [...row]);
      next[track][step] = !next[track][step];
      return next;
    });
  };

  const startPlayback = useCallback(() => {
    // Create AudioContext lazily on first play (browser autoplay policy)
    if (!audioContext) {
      const ctx = initAudioContext();
      setAudioContext(ctx);
    }
    scheduler.start();
    play();
  }, [audioContext, initAudioContext, scheduler, play]);

  const stopPlayback = useCallback(() => {
    scheduler.stop();
    stop();
  }, [scheduler, stop]);

  const handleBpmChange = (v: number) => {
    setLocalBpm(v);
    setBpm(v);
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
      {/* Header controls */}
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
          Step Seq
        </span>

        {/* Play/Stop */}
        <button
          onClick={isPlaying ? stopPlayback : startPlayback}
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

        {/* BPM */}
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

        {/* Step length */}
        <label
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            fontSize: 12,
          }}
        >
          <span style={{ color: THEME.textDim }}>Step</span>
          <select
            value={stepLength}
            onChange={(e) => setStepLength(e.target.value as StepLength)}
            style={{
              background: THEME.panel,
              border: `1px solid ${THEME.border}`,
              borderRadius: 4,
              color: THEME.text,
              padding: "2px 6px",
              fontSize: 12,
              fontFamily: "monospace",
            }}
          >
            <option value="1/4">1/4</option>
            <option value="1/8">1/8</option>
            <option value="1/16">1/16</option>
          </select>
        </label>
      </div>

      {/* Grid */}
      <div style={{ padding: 12, overflowY: "auto", flex: 1 }}>
        {Array.from({ length: TRACK_COUNT }, (_, track) => (
          <div
            key={track}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 4,
              marginBottom: 6,
            }}
          >
            {/* Track label */}
            <div
              style={{
                width: 28,
                fontSize: 11,
                color: TRACK_COLORS[track],
                textAlign: "right",
                flexShrink: 0,
              }}
            >
              {TRACK_NAMES[track]}
            </div>

            {/* Steps */}
            {Array.from({ length: STEPS }, (_, step) => {
              const isActive = grid[track][step];
              const isCurrent =
                step === scheduler.currentStep && scheduler.isPlaying;
              return (
                <button
                  key={step}
                  onClick={() => toggleStep(track, step)}
                  style={{
                    width: 28,
                    height: 28,
                    background: isCurrent
                      ? THEME.playing
                      : isActive
                        ? TRACK_COLORS[track]
                        : THEME.inactive,
                    border: `1px solid ${isCurrent ? THEME.playing : isActive ? TRACK_COLORS[track] : THEME.border}`,
                    borderRadius: 3,
                    cursor: "pointer",
                    opacity: isActive || isCurrent ? 1 : 0.5,
                    transition: "background 0.05s",
                    // Beat grouping: slight gap every 4 steps
                    marginLeft: step > 0 && step % 4 === 0 ? 6 : 0,
                  }}
                  title={`Track ${track + 1}, Step ${step + 1}`}
                />
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
};

export default StepSequencerModule;
