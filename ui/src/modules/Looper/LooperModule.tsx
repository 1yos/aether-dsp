/**
 * Looper Module — record audio from microphone, loop it back.
 * Uses MediaRecorder API for capture and AudioBufferSourceNode for playback.
 */
import React, { useState, useRef, useEffect, useCallback } from "react";
import { useTransportStore } from "../useTransportStore";

const THEME = {
  bg: "#060c12",
  panel: "#0d1a26",
  border: "#1a2a3a",
  text: "#e0e8f0",
  textDim: "#4a6a8a",
  accent: "#4fc3f7",
  record: "#ef5350",
  stop: "#ef5350",
  play: "#a5d6a7",
  waveform: "#4fc3f7",
};

type LooperState = "idle" | "recording" | "playing" | "stopped";

export const LooperModule: React.FC = () => {
  const { bpm } = useTransportStore();

  const [state, setState] = useState<LooperState>("idle");
  const [loopDuration, setLoopDuration] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const audioCtxRef = useRef<AudioContext | null>(null);
  const audioBufferRef = useRef<AudioBuffer | null>(null);
  const sourceNodeRef = useRef<AudioBufferSourceNode | null>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const streamRef = useRef<MediaStream | null>(null);

  const getAudioCtx = () => {
    if (!audioCtxRef.current) {
      audioCtxRef.current = new AudioContext();
    }
    return audioCtxRef.current;
  };

  const drawWaveform = useCallback((buffer: AudioBuffer) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const W = canvas.width;
    const H = canvas.height;
    ctx.clearRect(0, 0, W, H);
    ctx.fillStyle = THEME.panel;
    ctx.fillRect(0, 0, W, H);

    const data = buffer.getChannelData(0);
    const step = Math.ceil(data.length / W);
    const mid = H / 2;

    ctx.strokeStyle = THEME.waveform;
    ctx.lineWidth = 1;
    ctx.beginPath();

    for (let x = 0; x < W; x++) {
      let min = 1;
      let max = -1;
      for (let j = 0; j < step; j++) {
        const sample = data[x * step + j] ?? 0;
        if (sample < min) min = sample;
        if (sample > max) max = sample;
      }
      ctx.moveTo(x, mid + min * mid * 0.9);
      ctx.lineTo(x, mid + max * mid * 0.9);
    }
    ctx.stroke();

    // Center line
    ctx.strokeStyle = THEME.border;
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.moveTo(0, mid);
    ctx.lineTo(W, mid);
    ctx.stroke();
  }, []);

  const startRecording = async () => {
    setError(null);
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      streamRef.current = stream;
      chunksRef.current = [];

      const recorder = new MediaRecorder(stream);
      mediaRecorderRef.current = recorder;

      recorder.ondataavailable = (e) => {
        if (e.data.size > 0) chunksRef.current.push(e.data);
      };

      recorder.onstop = async () => {
        const blob = new Blob(chunksRef.current, { type: "audio/webm" });
        const arrayBuffer = await blob.arrayBuffer();
        try {
          const audioCtx = getAudioCtx();
          const decoded = await audioCtx.decodeAudioData(arrayBuffer);
          audioBufferRef.current = decoded;
          setLoopDuration(decoded.duration);
          drawWaveform(decoded);
          setState("stopped");
        } catch (err) {
          setError("Failed to decode audio. Try a different browser.");
          setState("idle");
        }
        // Stop all tracks
        streamRef.current?.getTracks().forEach((t) => t.stop());
        streamRef.current = null;
      };

      recorder.start();
      setState("recording");
    } catch (err) {
      setError("Microphone access denied or unavailable.");
    }
  };

  const stopRecording = () => {
    mediaRecorderRef.current?.stop();
  };

  const startPlayback = () => {
    const buffer = audioBufferRef.current;
    if (!buffer) return;

    const audioCtx = getAudioCtx();
    if (audioCtx.state === "suspended") audioCtx.resume();

    // Stop any existing source
    sourceNodeRef.current?.stop();
    sourceNodeRef.current?.disconnect();

    const source = audioCtx.createBufferSource();
    source.buffer = buffer;
    source.loop = true;
    source.connect(audioCtx.destination);
    source.start();
    sourceNodeRef.current = source;
    setState("playing");
  };

  const stopPlayback = () => {
    sourceNodeRef.current?.stop();
    sourceNodeRef.current?.disconnect();
    sourceNodeRef.current = null;
    setState("stopped");
  };

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      sourceNodeRef.current?.stop();
      streamRef.current?.getTracks().forEach((t) => t.stop());
    };
  }, []);

  const barsAtBpm =
    loopDuration != null ? (loopDuration / ((60 / bpm) * 4)).toFixed(2) : null;

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
      {/* Header */}
      <div
        style={{
          padding: "6px 12px",
          borderBottom: `1px solid ${THEME.border}`,
          fontSize: 11,
          color: THEME.textDim,
          letterSpacing: 2,
          textTransform: "uppercase",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <span>Looper</span>
        <span
          style={{
            color:
              state === "recording"
                ? THEME.record
                : state === "playing"
                  ? THEME.play
                  : THEME.textDim,
          }}
        >
          {state.toUpperCase()}
        </span>
      </div>

      {/* Controls */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
          padding: "10px 12px",
          borderBottom: `1px solid ${THEME.border}`,
        }}
      >
        {/* Record button */}
        {state === "idle" || state === "stopped" ? (
          <button
            onClick={startRecording}
            style={{
              padding: "6px 16px",
              background: THEME.record,
              border: "none",
              borderRadius: 4,
              color: "#fff",
              fontWeight: "bold",
              fontSize: 13,
              cursor: "pointer",
              fontFamily: "monospace",
            }}
          >
            ● REC
          </button>
        ) : state === "recording" ? (
          <button
            onClick={stopRecording}
            style={{
              padding: "6px 16px",
              background: THEME.stop,
              border: "none",
              borderRadius: 4,
              color: "#fff",
              fontWeight: "bold",
              fontSize: 13,
              cursor: "pointer",
              fontFamily: "monospace",
              animation: "pulse 1s infinite",
            }}
          >
            ■ Stop Rec
          </button>
        ) : null}

        {/* Play/Stop loop */}
        {(state === "stopped" || state === "playing") &&
          audioBufferRef.current && (
            <button
              onClick={state === "playing" ? stopPlayback : startPlayback}
              style={{
                padding: "6px 16px",
                background: state === "playing" ? THEME.stop : THEME.play,
                border: "none",
                borderRadius: 4,
                color: "#000",
                fontWeight: "bold",
                fontSize: 13,
                cursor: "pointer",
                fontFamily: "monospace",
              }}
            >
              {state === "playing" ? "■ Stop" : "▶ Play Loop"}
            </button>
          )}

        {/* Re-record */}
        {(state === "stopped" || state === "playing") && (
          <button
            onClick={() => {
              stopPlayback();
              audioBufferRef.current = null;
              setLoopDuration(null);
              setState("idle");
              // Clear canvas
              const canvas = canvasRef.current;
              if (canvas) {
                const ctx = canvas.getContext("2d");
                ctx?.clearRect(0, 0, canvas.width, canvas.height);
              }
            }}
            style={{
              padding: "6px 12px",
              background: THEME.panel,
              border: `1px solid ${THEME.border}`,
              borderRadius: 4,
              color: THEME.textDim,
              fontSize: 12,
              cursor: "pointer",
              fontFamily: "monospace",
            }}
          >
            ↺ Clear
          </button>
        )}
      </div>

      {/* Loop info */}
      {loopDuration != null && (
        <div
          style={{
            padding: "6px 12px",
            fontSize: 12,
            color: THEME.textDim,
            borderBottom: `1px solid ${THEME.border}`,
            display: "flex",
            gap: 16,
          }}
        >
          <span>
            Length:{" "}
            <span style={{ color: THEME.text }}>
              {loopDuration.toFixed(2)}s
            </span>
          </span>
          <span>
            Bars @ {bpm} BPM:{" "}
            <span style={{ color: THEME.text }}>{barsAtBpm}</span>
          </span>
        </div>
      )}

      {/* Error */}
      {error && (
        <div
          style={{
            padding: "6px 12px",
            fontSize: 12,
            color: THEME.record,
            borderBottom: `1px solid ${THEME.border}`,
          }}
        >
          {error}
        </div>
      )}

      {/* Waveform canvas */}
      <div style={{ flex: 1, padding: 12 }}>
        <canvas
          ref={canvasRef}
          width={600}
          height={120}
          style={{
            width: "100%",
            height: "100%",
            background: THEME.panel,
            borderRadius: 4,
            border: `1px solid ${THEME.border}`,
            display: "block",
          }}
        />
        {!audioBufferRef.current && (
          <div
            style={{
              position: "absolute",
              top: "50%",
              left: "50%",
              transform: "translate(-50%, -50%)",
              color: THEME.textDim,
              fontSize: 12,
              pointerEvents: "none",
            }}
          >
            {state === "recording" ? "Recording…" : "No loop recorded"}
          </div>
        )}
      </div>
    </div>
  );
};

export default LooperModule;
