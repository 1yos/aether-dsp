/**
 * InstrumentRecorder — record your own instrument samples.
 *
 * Workflow:
 * 1. User selects a note range and velocity layers
 * 2. For each note, the app prompts: "Play note X now"
 * 3. Records from microphone for a configurable duration
 * 4. Auto-detects the fundamental frequency via autocorrelation
 * 5. Assigns the recording to the correct zone in a SamplerInstrument JSON
 * 6. Exports as .aether-instrument file ready for the sampler
 *
 * The exported instrument can be loaded into any SamplerNode.
 */

import { useState, useRef, useCallback, useEffect } from "react";

// ── Types ─────────────────────────────────────────────────────────────────────

interface RecordedZone {
  noteNumber: number;
  noteName: string;
  audioBlob: Blob;
  audioUrl: string;
  detectedFreq: number | null;
  duration: number;
}

interface RecorderConfig {
  name: string;
  origin: string;
  description: string;
  noteStart: number;
  noteEnd: number;
  noteStep: number;
  recordDuration: number; // seconds
  attack: number;
  decay: number;
  sustain: number;
  release: number;
  maxVoices: number;
}

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

function midiToName(midi: number): string {
  return `${NOTE_NAMES[midi % 12]}${Math.floor(midi / 12) - 1}`;
}

function midiToFreq(midi: number): number {
  return 440 * Math.pow(2, (midi - 69) / 12);
}

// ── Pitch detection via autocorrelation ──────────────────────────────────────

function detectPitch(buffer: Float32Array, sampleRate: number): number | null {
  const SIZE = buffer.length;
  const MAX_SAMPLES = Math.floor(SIZE / 2);
  let bestOffset = -1;
  let bestCorrelation = 0;
  let rms = 0;

  for (let i = 0; i < SIZE; i++) rms += buffer[i] * buffer[i];
  rms = Math.sqrt(rms / SIZE);
  if (rms < 0.01) return null; // too quiet

  let lastCorrelation = 1;
  for (let offset = 0; offset < MAX_SAMPLES; offset++) {
    let correlation = 0;
    for (let i = 0; i < MAX_SAMPLES; i++) {
      correlation += Math.abs(buffer[i] - buffer[i + offset]);
    }
    correlation = 1 - correlation / MAX_SAMPLES;
    if (correlation > 0.9 && correlation > lastCorrelation) {
      bestCorrelation = correlation;
      bestOffset = offset;
    }
    lastCorrelation = correlation;
  }

  if (bestCorrelation > 0.9 && bestOffset > 0) {
    return sampleRate / bestOffset;
  }
  return null;
}

// ── Main component ────────────────────────────────────────────────────────────

interface InstrumentRecorderProps {
  onClose: () => void;
  onExport?: (instrumentJson: string) => void;
}

export function InstrumentRecorder({
  onClose,
  onExport,
}: InstrumentRecorderProps) {
  const [config, setConfig] = useState<RecorderConfig>({
    name: "My Instrument",
    origin: "",
    description: "",
    noteStart: 48, // C3
    noteEnd: 72, // C5
    noteStep: 3, // every 3 semitones
    recordDuration: 3,
    attack: 0.005,
    decay: 0.1,
    sustain: 0.8,
    release: 0.3,
    maxVoices: 16,
  });

  const [phase, setPhase] = useState<"setup" | "recording" | "review" | "done">(
    "setup",
  );
  const [currentNoteIdx, setCurrentNoteIdx] = useState(0);
  const [countdown, setCountdown] = useState(0);
  const [isRecording, setIsRecording] = useState(false);
  const [recordedZones, setRecordedZones] = useState<RecordedZone[]>([]);
  const [micError, setMicError] = useState<string | null>(null);

  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const streamRef = useRef<MediaStream | null>(null);

  // Compute note list from config
  const noteList: number[] = [];
  for (let n = config.noteStart; n <= config.noteEnd; n += config.noteStep) {
    noteList.push(n);
  }

  const currentNote = noteList[currentNoteIdx];

  // Request microphone access
  const initMic = useCallback(async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      streamRef.current = stream;

      const ctx = new AudioContext();
      audioContextRef.current = ctx;
      const source = ctx.createMediaStreamSource(stream);
      const analyser = ctx.createAnalyser();
      analyser.fftSize = 2048;
      source.connect(analyser);
      analyserRef.current = analyser;

      setMicError(null);
      return true;
    } catch (e) {
      setMicError(`Microphone access denied: ${e}`);
      return false;
    }
  }, []);

  const startRecording = useCallback(async () => {
    if (!streamRef.current) {
      const ok = await initMic();
      if (!ok) return;
    }

    chunksRef.current = [];
    const mr = new MediaRecorder(streamRef.current!);
    mediaRecorderRef.current = mr;

    mr.ondataavailable = (e) => {
      if (e.data.size > 0) chunksRef.current.push(e.data);
    };

    mr.onstop = async () => {
      const blob = new Blob(chunksRef.current, { type: "audio/webm" });
      const url = URL.createObjectURL(blob);

      // Detect pitch from the recorded audio
      let detectedFreq: number | null = null;
      try {
        const arrayBuffer = await blob.arrayBuffer();
        const ctx = audioContextRef.current ?? new AudioContext();
        const decoded = await ctx.decodeAudioData(arrayBuffer);
        const channelData = decoded.getChannelData(0);
        detectedFreq = detectPitch(channelData, decoded.sampleRate);
      } catch {
        /* ignore pitch detection errors */
      }

      const zone: RecordedZone = {
        noteNumber: currentNote,
        noteName: midiToName(currentNote),
        audioBlob: blob,
        audioUrl: url,
        detectedFreq,
        duration: config.recordDuration,
      };

      setRecordedZones((prev) => [...prev, zone]);
      setIsRecording(false);

      // Advance to next note or finish
      if (currentNoteIdx + 1 < noteList.length) {
        setCurrentNoteIdx((i) => i + 1);
      } else {
        setPhase("review");
      }
    };

    mr.start();
    setIsRecording(true);

    // Auto-stop after recordDuration
    setTimeout(() => {
      if (mr.state === "recording") mr.stop();
    }, config.recordDuration * 1000);
  }, [
    currentNote,
    currentNoteIdx,
    noteList.length,
    config.recordDuration,
    initMic,
  ]);

  // Countdown before recording
  const startCountdown = useCallback(() => {
    setCountdown(3);
    const interval = setInterval(() => {
      setCountdown((c) => {
        if (c <= 1) {
          clearInterval(interval);
          startRecording();
          return 0;
        }
        return c - 1;
      });
    }, 1000);
  }, [startRecording]);

  // Export as .aether-instrument JSON
  const exportInstrument = useCallback(() => {
    const zones = recordedZones.map((zone) => ({
      id: `zone-${zone.noteNumber}`,
      file_path: `${zone.noteName}.wav`,
      root_note: zone.noteNumber,
      note_low: Math.max(0, zone.noteNumber - Math.floor(config.noteStep / 2)),
      note_high: Math.min(
        127,
        zone.noteNumber + Math.floor(config.noteStep / 2),
      ),
      velocity_low: 0,
      velocity_high: 127,
      articulation: "OneShot",
      volume_db: 0.0,
      tune_cents: zone.detectedFreq
        ? 1200 * Math.log2(zone.detectedFreq / midiToFreq(zone.noteNumber))
        : 0.0,
      release_file: null,
    }));

    const instrument = {
      name: config.name,
      origin: config.origin,
      description: config.description,
      author: "Recorded with Aether Studio",
      tuning: {
        name: "12-TET",
        description: "Standard equal temperament",
        frequencies: Array.from(
          { length: 128 },
          (_, n) => 440 * Math.pow(2, (n - 69) / 12),
        ),
      },
      zones,
      attack: config.attack,
      decay: config.decay,
      sustain: config.sustain,
      release: config.release,
      max_voices: config.maxVoices,
    };

    const json = JSON.stringify(instrument, null, 2);

    // Download as .aether-instrument
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${config.name.toLowerCase().replace(/\s+/g, "-")}.aether-instrument`;
    a.click();
    URL.revokeObjectURL(url);

    onExport?.(json);
    setPhase("done");
  }, [recordedZones, config, onExport]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      streamRef.current?.getTracks().forEach((t) => t.stop());
      audioContextRef.current?.close();
      recordedZones.forEach((z) => URL.revokeObjectURL(z.audioUrl));
    };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const bg = "#060c12";
  const border = "#0f1e2e";
  const text = "#e0e8f0";
  const dim = "#4a6a8a";
  const accent = "#38bdf8";

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 2000,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "rgba(2,5,10,0.92)",
        backdropFilter: "blur(12px)",
      }}
    >
      <div
        style={{
          width: 560,
          maxHeight: "85vh",
          overflow: "auto",
          background: bg,
          border: `1px solid ${border}`,
          borderRadius: 12,
          boxShadow: "0 24px 64px rgba(0,0,0,0.7)",
        }}
      >
        {/* Header */}
        <div
          style={{
            padding: "16px 20px",
            borderBottom: `1px solid ${border}`,
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <div>
            <div style={{ fontSize: 16, fontWeight: 700, color: text }}>
              Instrument Recorder
            </div>
            <div style={{ fontSize: 11, color: dim, marginTop: 2 }}>
              Record your own instrument samples
            </div>
          </div>
          <button
            onClick={onClose}
            style={{
              background: "none",
              border: "none",
              color: dim,
              cursor: "pointer",
              fontSize: 18,
            }}
          >
            ✕
          </button>
        </div>

        <div style={{ padding: 20 }}>
          {/* Setup phase */}
          {phase === "setup" && (
            <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
              {[
                { label: "Instrument Name", key: "name", type: "text" },
                { label: "Origin / Region", key: "origin", type: "text" },
                { label: "Description", key: "description", type: "text" },
              ].map(({ label, key, type }) => (
                <div key={key}>
                  <label
                    style={{
                      fontSize: 11,
                      color: dim,
                      display: "block",
                      marginBottom: 4,
                    }}
                  >
                    {label}
                  </label>
                  <input
                    type={type}
                    value={
                      (config as unknown as Record<string, unknown>)[
                        key
                      ] as string
                    }
                    onChange={(e) =>
                      setConfig((c) => ({ ...c, [key]: e.target.value }))
                    }
                    style={{
                      width: "100%",
                      background: "#0a1520",
                      border: `1px solid ${border}`,
                      borderRadius: 6,
                      color: text,
                      padding: "6px 10px",
                      fontSize: 12,
                      boxSizing: "border-box",
                    }}
                  />
                </div>
              ))}

              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "1fr 1fr 1fr",
                  gap: 10,
                }}
              >
                {[
                  { label: "Start Note", key: "noteStart", min: 0, max: 127 },
                  { label: "End Note", key: "noteEnd", min: 0, max: 127 },
                  {
                    label: "Step (semitones)",
                    key: "noteStep",
                    min: 1,
                    max: 12,
                  },
                ].map(({ label, key, min, max }) => (
                  <div key={key}>
                    <label
                      style={{
                        fontSize: 11,
                        color: dim,
                        display: "block",
                        marginBottom: 4,
                      }}
                    >
                      {label}{" "}
                      {key === "noteStart"
                        ? `(${midiToName(config.noteStart)})`
                        : key === "noteEnd"
                          ? `(${midiToName(config.noteEnd)})`
                          : ""}
                    </label>
                    <input
                      type="number"
                      min={min}
                      max={max}
                      value={
                        (config as unknown as Record<string, unknown>)[
                          key
                        ] as number
                      }
                      onChange={(e) =>
                        setConfig((c) => ({
                          ...c,
                          [key]: parseInt(e.target.value) || 0,
                        }))
                      }
                      style={{
                        width: "100%",
                        background: "#0a1520",
                        border: `1px solid ${border}`,
                        borderRadius: 6,
                        color: text,
                        padding: "6px 10px",
                        fontSize: 12,
                        boxSizing: "border-box",
                      }}
                    />
                  </div>
                ))}
              </div>

              <div>
                <label
                  style={{
                    fontSize: 11,
                    color: dim,
                    display: "block",
                    marginBottom: 4,
                  }}
                >
                  Record Duration: {config.recordDuration}s per note
                </label>
                <input
                  type="range"
                  min={1}
                  max={10}
                  value={config.recordDuration}
                  onChange={(e) =>
                    setConfig((c) => ({
                      ...c,
                      recordDuration: parseInt(e.target.value),
                    }))
                  }
                  style={{ width: "100%" }}
                />
              </div>

              <div
                style={{
                  padding: "10px 12px",
                  background: "rgba(56,189,248,0.06)",
                  border: `1px solid rgba(56,189,248,0.15)`,
                  borderRadius: 6,
                  fontSize: 11,
                  color: dim,
                }}
              >
                Will record{" "}
                <strong style={{ color: accent }}>
                  {noteList.length} notes
                </strong>{" "}
                from {midiToName(config.noteStart)} to{" "}
                {midiToName(config.noteEnd)}, every {config.noteStep} semitone
                {config.noteStep > 1 ? "s" : ""}. Total time: ~
                {Math.ceil(
                  (noteList.length * (config.recordDuration + 3)) / 60,
                )}{" "}
                minutes.
              </div>

              {micError && (
                <div
                  style={{
                    color: "#ef5350",
                    fontSize: 11,
                    padding: "8px 12px",
                    background: "rgba(239,83,80,0.08)",
                    borderRadius: 6,
                  }}
                >
                  {micError}
                </div>
              )}

              <button
                onClick={async () => {
                  const ok = await initMic();
                  if (ok) setPhase("recording");
                }}
                style={{
                  padding: "10px 20px",
                  background: accent,
                  border: "none",
                  borderRadius: 8,
                  color: "#000",
                  fontWeight: 700,
                  fontSize: 13,
                  cursor: "pointer",
                }}
              >
                Start Recording
              </button>
            </div>
          )}

          {/* Recording phase */}
          {phase === "recording" && (
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: 20,
                padding: "20px 0",
              }}
            >
              <div
                style={{
                  fontSize: 48,
                  fontWeight: 800,
                  color: accent,
                  fontFamily: "monospace",
                }}
              >
                {midiToName(currentNote)}
              </div>
              <div style={{ fontSize: 14, color: dim }}>
                Note {currentNoteIdx + 1} of {noteList.length} · MIDI{" "}
                {currentNote} · {midiToFreq(currentNote).toFixed(1)} Hz
              </div>

              {countdown > 0 && (
                <div
                  style={{
                    fontSize: 64,
                    fontWeight: 900,
                    color: "#ffd54f",
                    fontFamily: "monospace",
                  }}
                >
                  {countdown}
                </div>
              )}

              {isRecording && (
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    gap: 8,
                  }}
                >
                  <div
                    style={{
                      width: 16,
                      height: 16,
                      borderRadius: "50%",
                      background: "#ef5350",
                      animation: "recPulse 0.5s ease-in-out infinite",
                    }}
                  />
                  <div
                    style={{ fontSize: 13, color: "#ef5350", fontWeight: 600 }}
                  >
                    Recording… {config.recordDuration}s
                  </div>
                </div>
              )}

              {!isRecording && countdown === 0 && (
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    gap: 12,
                  }}
                >
                  <div style={{ fontSize: 13, color: dim }}>
                    Play{" "}
                    <strong style={{ color: text }}>
                      {midiToName(currentNote)}
                    </strong>{" "}
                    on your instrument, then click Record
                  </div>
                  <button
                    onClick={startCountdown}
                    style={{
                      padding: "12px 32px",
                      background: "#ef5350",
                      border: "none",
                      borderRadius: 8,
                      color: "#fff",
                      fontWeight: 700,
                      fontSize: 14,
                      cursor: "pointer",
                    }}
                  >
                    ● Record
                  </button>
                  <button
                    onClick={() => {
                      if (currentNoteIdx + 1 < noteList.length) {
                        setCurrentNoteIdx((i) => i + 1);
                      } else {
                        setPhase("review");
                      }
                    }}
                    style={{
                      padding: "6px 16px",
                      background: "transparent",
                      border: `1px solid ${border}`,
                      borderRadius: 6,
                      color: dim,
                      fontSize: 11,
                      cursor: "pointer",
                    }}
                  >
                    Skip this note
                  </button>
                </div>
              )}

              <div
                style={{
                  width: "100%",
                  background: "#0a1520",
                  borderRadius: 6,
                  height: 6,
                }}
              >
                <div
                  style={{
                    width: `${(currentNoteIdx / noteList.length) * 100}%`,
                    height: "100%",
                    background: accent,
                    borderRadius: 6,
                    transition: "width 0.3s",
                  }}
                />
              </div>
              <div style={{ fontSize: 10, color: dim }}>
                {recordedZones.length} notes recorded
              </div>
            </div>
          )}

          {/* Review phase */}
          {phase === "review" && (
            <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              <div style={{ fontSize: 13, color: text, fontWeight: 600 }}>
                {recordedZones.length} notes recorded
              </div>
              <div
                style={{
                  maxHeight: 300,
                  overflowY: "auto",
                  display: "flex",
                  flexDirection: "column",
                  gap: 4,
                }}
              >
                {recordedZones.map((zone) => (
                  <div
                    key={zone.noteNumber}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 10,
                      padding: "8px 10px",
                      background: "#0a1520",
                      borderRadius: 6,
                      border: `1px solid ${border}`,
                    }}
                  >
                    <span
                      style={{
                        fontSize: 12,
                        color: accent,
                        fontFamily: "monospace",
                        minWidth: 32,
                      }}
                    >
                      {zone.noteName}
                    </span>
                    <audio
                      src={zone.audioUrl}
                      controls
                      style={{ height: 28, flex: 1 }}
                    />
                    {zone.detectedFreq && (
                      <span
                        style={{
                          fontSize: 10,
                          color: dim,
                          fontFamily: "monospace",
                          minWidth: 60,
                        }}
                      >
                        {zone.detectedFreq.toFixed(1)} Hz
                      </span>
                    )}
                  </div>
                ))}
              </div>
              <div style={{ display: "flex", gap: 8 }}>
                <button
                  onClick={exportInstrument}
                  style={{
                    flex: 1,
                    padding: "10px",
                    background: accent,
                    border: "none",
                    borderRadius: 8,
                    color: "#000",
                    fontWeight: 700,
                    fontSize: 13,
                    cursor: "pointer",
                  }}
                >
                  Export .aether-instrument
                </button>
                <button
                  onClick={() => {
                    setPhase("recording");
                    setCurrentNoteIdx(0);
                    setRecordedZones([]);
                  }}
                  style={{
                    padding: "10px 16px",
                    background: "transparent",
                    border: `1px solid ${border}`,
                    borderRadius: 8,
                    color: dim,
                    fontSize: 12,
                    cursor: "pointer",
                  }}
                >
                  Re-record
                </button>
              </div>
            </div>
          )}

          {/* Done phase */}
          {phase === "done" && (
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: 16,
                padding: "20px 0",
              }}
            >
              <div style={{ fontSize: 32 }}>✓</div>
              <div style={{ fontSize: 14, color: text, fontWeight: 600 }}>
                Instrument exported!
              </div>
              <div style={{ fontSize: 12, color: dim, textAlign: "center" }}>
                Load the .aether-instrument file into a SamplerNode to play it.
              </div>
              <button
                onClick={onClose}
                style={{
                  padding: "10px 24px",
                  background: accent,
                  border: "none",
                  borderRadius: 8,
                  color: "#000",
                  fontWeight: 700,
                  cursor: "pointer",
                }}
              >
                Close
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
