/**
 * SettingsPanel — audio device, buffer size, sample rate, MIDI, theme.
 *
 * Communicates with aether-host via sendIntent for audio/MIDI settings.
 * UI preferences (theme, shortcuts) are stored in localStorage.
 */

import { useState, useEffect, useCallback } from "react";
import { useEngineStore } from "../studio/store/engineStore";

interface AudioDevice {
  id: string;
  name: string;
  isDefault: boolean;
}

interface SettingsState {
  audioDevices: AudioDevice[];
  selectedDevice: string;
  bufferSize: number;
  sampleRate: number;
  theme: "dark" | "light";
  midiDevices: string[];
  selectedMidiDevice: string | null;
}

const BUFFER_SIZES = [64, 128, 256, 512, 1024];
const SAMPLE_RATES = [44100, 48000, 88200, 96000];

const PREFS_KEY = "aether-settings";

function loadPrefs(): Partial<SettingsState> {
  try {
    const raw = localStorage.getItem(PREFS_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function savePrefs(prefs: Partial<SettingsState>) {
  try {
    localStorage.setItem(PREFS_KEY, JSON.stringify(prefs));
  } catch {
    /* ignore */
  }
}

interface SettingsPanelProps {
  onClose: () => void;
}

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const midiPorts = useEngineStore((s) => s.midiPorts);
  const wsStatus = useEngineStore((s) => s.wsStatus);

  const prefs = loadPrefs();

  const [bufferSize, setBufferSize] = useState<number>(prefs.bufferSize ?? 128);
  const [sampleRate, setSampleRate] = useState<number>(
    prefs.sampleRate ?? 48000,
  );
  const [theme, setTheme] = useState<"dark" | "light">(prefs.theme ?? "dark");
  const [selectedMidi, setSelectedMidi] = useState<string | null>(
    prefs.selectedMidiDevice ?? null,
  );
  const [restartRequired, setRestartRequired] = useState(false);

  // Request MIDI port list on mount
  useEffect(() => {
    sendIntent?.({ type: "midi_list_ports" });
  }, [sendIntent]);

  const handleBufferSize = useCallback(
    (size: number) => {
      setBufferSize(size);
      savePrefs({
        bufferSize: size,
        sampleRate,
        theme,
        selectedMidiDevice: selectedMidi,
      });
      sendIntent?.({ type: "set_buffer_size", buffer_size: size });
      setRestartRequired(true);
    },
    [sampleRate, theme, selectedMidi, sendIntent],
  );

  const handleSampleRate = useCallback(
    (rate: number) => {
      setSampleRate(rate);
      savePrefs({
        bufferSize,
        sampleRate: rate,
        theme,
        selectedMidiDevice: selectedMidi,
      });
      sendIntent?.({ type: "set_sample_rate", sample_rate: rate });
      setRestartRequired(true);
    },
    [bufferSize, theme, selectedMidi, sendIntent],
  );

  const handleTheme = useCallback(
    (t: "dark" | "light") => {
      setTheme(t);
      savePrefs({
        bufferSize,
        sampleRate,
        theme: t,
        selectedMidiDevice: selectedMidi,
      });
      document.documentElement.setAttribute("data-theme", t);
    },
    [bufferSize, sampleRate, selectedMidi],
  );

  const handleMidiConnect = useCallback(
    (portName: string, index: number) => {
      setSelectedMidi(portName);
      savePrefs({
        bufferSize,
        sampleRate,
        theme,
        selectedMidiDevice: portName,
      });
      sendIntent?.({ type: "midi_connect", port_index: index });
    },
    [bufferSize, sampleRate, theme, sendIntent],
  );

  const bg = "#060c12";
  const border = "#0f1e2e";
  const text = "#e0e8f0";
  const dim = "#4a6a8a";
  const accent = "#38bdf8";
  const surface = "#0a1520";

  const latencyMs = ((bufferSize / sampleRate) * 1000).toFixed(1);

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 2000,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "rgba(2,5,10,0.88)",
        backdropFilter: "blur(12px)",
      }}
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div
        style={{
          width: 520,
          maxHeight: "80vh",
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
            <div style={{ fontSize: 15, fontWeight: 700, color: text }}>
              Settings
            </div>
            <div style={{ fontSize: 11, color: dim, marginTop: 2 }}>
              Audio, MIDI, and display preferences
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

        <div
          style={{
            padding: 20,
            display: "flex",
            flexDirection: "column",
            gap: 24,
          }}
        >
          {/* Connection status */}
          <div
            style={{
              padding: "10px 14px",
              background:
                wsStatus === "connected"
                  ? "rgba(0,229,160,0.06)"
                  : "rgba(239,83,80,0.06)",
              border: `1px solid ${wsStatus === "connected" ? "rgba(0,229,160,0.2)" : "rgba(239,83,80,0.2)"}`,
              borderRadius: 8,
              display: "flex",
              alignItems: "center",
              gap: 8,
              fontSize: 12,
              color: wsStatus === "connected" ? "#00e5a0" : "#ef5350",
            }}
          >
            <span
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: wsStatus === "connected" ? "#00e5a0" : "#ef5350",
                flexShrink: 0,
              }}
            />
            {wsStatus === "connected"
              ? "Engine connected — ws://127.0.0.1:9001"
              : "Engine offline — start aether-host to change audio settings"}
          </div>

          {/* Audio section */}
          <section>
            <div
              style={{
                fontSize: 11,
                color: accent,
                fontWeight: 600,
                letterSpacing: "0.08em",
                marginBottom: 12,
                textTransform: "uppercase",
              }}
            >
              Audio
            </div>

            {/* Buffer size */}
            <div style={{ marginBottom: 16 }}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  marginBottom: 8,
                }}
              >
                <label style={{ fontSize: 12, color: text }}>Buffer Size</label>
                <span
                  style={{ fontSize: 11, color: dim, fontFamily: "monospace" }}
                >
                  {bufferSize} samples · {latencyMs}ms latency
                </span>
              </div>
              <div style={{ display: "flex", gap: 6 }}>
                {BUFFER_SIZES.map((size) => (
                  <button
                    key={size}
                    onClick={() => handleBufferSize(size)}
                    style={{
                      flex: 1,
                      padding: "7px 0",
                      background: bufferSize === size ? accent : surface,
                      border: `1px solid ${bufferSize === size ? accent : border}`,
                      borderRadius: 6,
                      color: bufferSize === size ? "#000" : dim,
                      fontSize: 11,
                      fontWeight: bufferSize === size ? 700 : 400,
                      cursor: "pointer",
                    }}
                  >
                    {size}
                  </button>
                ))}
              </div>
            </div>

            {/* Sample rate */}
            <div>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  marginBottom: 8,
                }}
              >
                <label style={{ fontSize: 12, color: text }}>Sample Rate</label>
                <span
                  style={{ fontSize: 11, color: dim, fontFamily: "monospace" }}
                >
                  {(sampleRate / 1000).toFixed(1)} kHz
                </span>
              </div>
              <div style={{ display: "flex", gap: 6 }}>
                {SAMPLE_RATES.map((rate) => (
                  <button
                    key={rate}
                    onClick={() => handleSampleRate(rate)}
                    style={{
                      flex: 1,
                      padding: "7px 0",
                      background: sampleRate === rate ? accent : surface,
                      border: `1px solid ${sampleRate === rate ? accent : border}`,
                      borderRadius: 6,
                      color: sampleRate === rate ? "#000" : dim,
                      fontSize: 11,
                      fontWeight: sampleRate === rate ? 700 : 400,
                      cursor: "pointer",
                    }}
                  >
                    {rate >= 1000 ? `${rate / 1000}k` : rate}
                  </button>
                ))}
              </div>
            </div>

            {restartRequired && (
              <div
                style={{
                  marginTop: 10,
                  padding: "8px 12px",
                  background: "rgba(255,213,79,0.08)",
                  border: "1px solid rgba(255,213,79,0.2)",
                  borderRadius: 6,
                  fontSize: 11,
                  color: "#ffd54f",
                }}
              >
                ⚠ Restart aether-host to apply audio device changes
              </div>
            )}
          </section>

          {/* MIDI section */}
          <section>
            <div
              style={{
                fontSize: 11,
                color: accent,
                fontWeight: 600,
                letterSpacing: "0.08em",
                marginBottom: 12,
                textTransform: "uppercase",
              }}
            >
              MIDI
            </div>

            {midiPorts.length === 0 ? (
              <div style={{ fontSize: 12, color: dim, padding: "10px 0" }}>
                No MIDI devices detected. Connect a device and click Refresh.
              </div>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                {midiPorts.map((port, i) => (
                  <button
                    key={port}
                    onClick={() => handleMidiConnect(port, i)}
                    style={{
                      padding: "10px 14px",
                      background:
                        selectedMidi === port
                          ? "rgba(56,189,248,0.08)"
                          : surface,
                      border: `1px solid ${selectedMidi === port ? accent : border}`,
                      borderRadius: 8,
                      color: selectedMidi === port ? accent : text,
                      fontSize: 12,
                      textAlign: "left",
                      cursor: "pointer",
                      display: "flex",
                      alignItems: "center",
                      gap: 8,
                    }}
                  >
                    <span style={{ fontSize: 14 }}>🎹</span>
                    <span style={{ flex: 1 }}>{port}</span>
                    {selectedMidi === port && (
                      <span style={{ fontSize: 10, color: accent }}>
                        ● Active
                      </span>
                    )}
                  </button>
                ))}
              </div>
            )}

            <button
              onClick={() => sendIntent?.({ type: "midi_list_ports" })}
              style={{
                marginTop: 10,
                padding: "7px 14px",
                background: "transparent",
                border: `1px solid ${border}`,
                borderRadius: 6,
                color: dim,
                fontSize: 11,
                cursor: "pointer",
              }}
            >
              ↻ Refresh MIDI devices
            </button>
          </section>

          {/* Display section */}
          <section>
            <div
              style={{
                fontSize: 11,
                color: accent,
                fontWeight: 600,
                letterSpacing: "0.08em",
                marginBottom: 12,
                textTransform: "uppercase",
              }}
            >
              Display
            </div>

            <div style={{ display: "flex", gap: 8 }}>
              {(["dark", "light"] as const).map((t) => (
                <button
                  key={t}
                  onClick={() => handleTheme(t)}
                  style={{
                    flex: 1,
                    padding: "10px",
                    background: theme === t ? "rgba(56,189,248,0.08)" : surface,
                    border: `1px solid ${theme === t ? accent : border}`,
                    borderRadius: 8,
                    color: theme === t ? accent : dim,
                    fontSize: 12,
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: 6,
                  }}
                >
                  <span>{t === "dark" ? "🌙" : "☀️"}</span>
                  <span style={{ textTransform: "capitalize" }}>{t}</span>
                </button>
              ))}
            </div>
          </section>

          {/* Keyboard shortcuts reference */}
          <section>
            <div
              style={{
                fontSize: 11,
                color: accent,
                fontWeight: 600,
                letterSpacing: "0.08em",
                marginBottom: 12,
                textTransform: "uppercase",
              }}
            >
              Keyboard Shortcuts
            </div>
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "1fr 1fr",
                gap: 6,
              }}
            >
              {[
                ["1", "Explore mode"],
                ["2", "Create mode"],
                ["3", "Arrange mode"],
                ["4", "Perform mode"],
                ["Ctrl+S", "Save project"],
                ["Ctrl+O", "Load project"],
                ["Ctrl+Z", "Undo"],
                ["Ctrl+Y", "Redo"],
                ["Esc", "Close panel"],
                ["Space", "Play / Stop"],
              ].map(([key, desc]) => (
                <div
                  key={key}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    padding: "6px 10px",
                    background: surface,
                    borderRadius: 6,
                    border: `1px solid ${border}`,
                  }}
                >
                  <kbd
                    style={{
                      fontSize: 10,
                      fontFamily: "monospace",
                      background: "#0f1e2e",
                      border: `1px solid ${border}`,
                      borderRadius: 4,
                      padding: "2px 6px",
                      color: accent,
                      whiteSpace: "nowrap",
                    }}
                  >
                    {key}
                  </kbd>
                  <span style={{ fontSize: 11, color: dim }}>{desc}</span>
                </div>
              ))}
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
