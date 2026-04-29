/**
 * Timeline Module — horizontal scrollable timeline with playhead.
 * Uses useTransportStore for shared transport state.
 * Playhead advances via requestAnimationFrame when playing.
 */
import React, { useRef, useEffect, useCallback } from "react";
import { useTransportStore } from "../useTransportStore";

const THEME = {
  bg: "#060c12",
  panel: "#0d1a26",
  border: "#1a2a3a",
  text: "#e0e8f0",
  textDim: "#4a6a8a",
  accent: "#4fc3f7",
  stop: "#ef5350",
  playhead: "#ef5350",
  bar: "#1a2a3a",
  beat: "#0d1a26",
};

const RULER_HEIGHT = 28;
const BEAT_WIDTH = 48; // pixels per beat
const TOTAL_BEATS = 64;

export const TimelineModule: React.FC = () => {
  const { bpm, isPlaying, currentBeat, setBpm, play, stop } =
    useTransportStore();

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rafRef = useRef<number | null>(null);
  const playStartTimeRef = useRef<number>(0);
  const playStartBeatRef = useRef<number>(0);
  const scrollRef = useRef<number>(0);

  const drawTimeline = useCallback(
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

      const scroll = scrollRef.current;

      // Draw ruler
      ctx.fillStyle = THEME.panel;
      ctx.fillRect(0, 0, W, RULER_HEIGHT);
      ctx.strokeStyle = THEME.border;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, RULER_HEIGHT);
      ctx.lineTo(W, RULER_HEIGHT);
      ctx.stroke();

      // Draw beat/bar markers
      const firstBeat = Math.floor(scroll / BEAT_WIDTH);
      const lastBeat = firstBeat + Math.ceil(W / BEAT_WIDTH) + 1;

      for (
        let beat = firstBeat;
        beat <= Math.min(lastBeat, TOTAL_BEATS);
        beat++
      ) {
        const x = beat * BEAT_WIDTH - scroll;
        const isBar = beat % 4 === 0;

        // Ruler tick
        ctx.strokeStyle = isBar ? THEME.accent : THEME.textDim;
        ctx.lineWidth = isBar ? 1 : 0.5;
        ctx.beginPath();
        ctx.moveTo(x, isBar ? 0 : RULER_HEIGHT / 2);
        ctx.lineTo(x, RULER_HEIGHT);
        ctx.stroke();

        // Bar number
        if (isBar) {
          ctx.fillStyle = THEME.text;
          ctx.font = "10px monospace";
          ctx.fillText(`${Math.floor(beat / 4) + 1}`, x + 3, 14);
        }

        // Grid line into timeline body
        ctx.strokeStyle = isBar ? THEME.bar : THEME.beat;
        ctx.lineWidth = isBar ? 1 : 0.5;
        ctx.beginPath();
        ctx.moveTo(x, RULER_HEIGHT);
        ctx.lineTo(x, H);
        ctx.stroke();
      }

      // Track lanes (visual only)
      const laneHeight = (H - RULER_HEIGHT) / 4;
      for (let i = 1; i < 4; i++) {
        const y = RULER_HEIGHT + i * laneHeight;
        ctx.strokeStyle = THEME.border;
        ctx.lineWidth = 0.5;
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(W, y);
        ctx.stroke();
      }

      // Playhead
      const px = playheadBeat * BEAT_WIDTH - scroll;
      if (px >= 0 && px <= W) {
        ctx.strokeStyle = THEME.playhead;
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(px, 0);
        ctx.lineTo(px, H);
        ctx.stroke();

        // Playhead triangle
        ctx.fillStyle = THEME.playhead;
        ctx.beginPath();
        ctx.moveTo(px - 6, 0);
        ctx.lineTo(px + 6, 0);
        ctx.lineTo(px, 10);
        ctx.closePath();
        ctx.fill();
      }

      // BPM display
      ctx.fillStyle = THEME.textDim;
      ctx.font = "10px monospace";
      ctx.fillText(`${bpm} BPM`, W - 60, 14);
    },
    [bpm],
  );

  // Animation loop
  useEffect(() => {
    if (!isPlaying) {
      drawTimeline(currentBeat);
      return;
    }

    playStartTimeRef.current = performance.now();
    playStartBeatRef.current = currentBeat;

    const tick = () => {
      const elapsed = (performance.now() - playStartTimeRef.current) / 1000;
      const beat = playStartBeatRef.current + elapsed * (bpm / 60);

      // Auto-scroll to keep playhead visible
      const canvas = canvasRef.current;
      if (canvas) {
        const px = beat * BEAT_WIDTH - scrollRef.current;
        if (px > canvas.width * 0.75) {
          scrollRef.current = beat * BEAT_WIDTH - canvas.width * 0.25;
        }
      }

      drawTimeline(beat);
      rafRef.current = requestAnimationFrame(tick);
    };

    rafRef.current = requestAnimationFrame(tick);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    };
  }, [isPlaying, bpm, currentBeat, drawTimeline]);

  // Draw on mount and when not playing
  useEffect(() => {
    if (!isPlaying) drawTimeline(currentBeat);
  }, [currentBeat, drawTimeline, isPlaying]);

  const handlePlayStop = () => {
    if (isPlaying) {
      stop();
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    } else {
      play();
    }
  };

  const handleBpmChange = (v: number) => {
    setBpm(v);
  };

  // Click on ruler to seek
  const handleRulerClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    if (y > RULER_HEIGHT) return;
    const beat = (x + scrollRef.current) / BEAT_WIDTH;
    playStartBeatRef.current = beat;
    playStartTimeRef.current = performance.now();
    drawTimeline(beat);
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
          Timeline
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
            value={bpm}
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

        <span style={{ fontSize: 11, color: THEME.textDim }}>
          Beat {Math.floor(currentBeat) + 1} | Bar{" "}
          {Math.floor(currentBeat / 4) + 1}
        </span>
      </div>

      {/* Timeline canvas */}
      <div style={{ flex: 1, overflow: "hidden" }}>
        <canvas
          ref={canvasRef}
          width={900}
          height={200}
          onClick={handleRulerClick}
          style={{
            display: "block",
            width: "100%",
            height: "100%",
            cursor: "pointer",
          }}
        />
      </div>
    </div>
  );
};

export default TimelineModule;
