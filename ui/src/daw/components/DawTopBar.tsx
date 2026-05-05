/**
 * DawTopBar — transport + view switcher + status.
 * Single 48px bar. No redundancy.
 */

import { useCallback, useRef, useEffect } from "react";
import { useDawStore, DawView } from "../store/dawStore";
import { useEngineStore } from "../../studio/store/engineStore";
import { useState } from "react";

const VIEWS: Array<{
  id: DawView;
  label: string;
  icon: string;
  shortcut: string;
}> = [
  { id: "song", label: "Song", icon: "≡", shortcut: "F1" },
  { id: "piano-roll", label: "Piano Roll", icon: "♩", shortcut: "F2" },
  { id: "mixer", label: "Mixer", icon: "⊟", shortcut: "F3" },
  { id: "patcher", label: "Patcher", icon: "⬡", shortcut: "F4" },
  { id: "perform", label: "Perform", icon: "🎚", shortcut: "F5" },
];

interface DawTopBarProps {
  onOpenRecorder: () => void;
}

export function DawTopBar({ onOpenRecorder }: DawTopBarProps) {
  const activeView = useDawStore((s) => s.activeView);
  const setView = useDawStore((s) => s.setView);
  const transport = useDawStore((s) => s.transport);
  const play = useDawStore((s) => s.play);
  const stop = useDawStore((s) => s.stop);
  const toggleRecord = useDawStore((s) => s.toggleRecord);
  const setBpm = useDawStore((s) => s.setBpm);
  const browserOpen = useDawStore((s) => s.browserOpen);
  const setBrowserOpen = useDawStore((s) => s.setBrowserOpen);
  const propertiesOpen = useDawStore((s) => s.propertiesOpen);
  const setPropertiesOpen = useDawStore((s) => s.setPropertiesOpen);

  const wsStatus = useEngineStore((s) => s.wsStatus);
  const audioActive = useEngineStore((s) => s.audioActive);

  const [bpmEditing, setBpmEditing] = useState(false);
  const [bpmInput, setBpmInput] = useState(String(transport.bpm));
  const tapTimesRef = useRef<number[]>([]);

  const handleTap = useCallback(() => {
    const now = performance.now();
    tapTimesRef.current.push(now);
    if (tapTimesRef.current.length > 4) tapTimesRef.current.shift();
    if (tapTimesRef.current.length >= 2) {
      const intervals = tapTimesRef.current
        .slice(1)
        .map((t, i) => t - tapTimesRef.current[i]);
      const avg = intervals.reduce((a, b) => a + b, 0) / intervals.length;
      setBpm(Math.round(60000 / avg));
    }
  }, [setBpm]);

  // Wire transport play/stop to engine mute toggle
  const toggleMute = useEngineStore((s) => s.toggleMute);
  const sendIntent = useEngineStore((s) => s.sendIntent);

  // Sync DAW play state → engine (unmute when playing, mute when stopped)
  useEffect(() => {
    sendIntent?.({ type: "set_mute", muted: !transport.isPlaying });
  }, [transport.isPlaying, sendIntent]);

  // BPM → engine
  useEffect(() => {
    sendIntent?.({ type: "set_bpm", bpm: transport.bpm });
  }, [transport.bpm, sendIntent]);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      )
        return;
      if (e.ctrlKey || e.metaKey) return;
      switch (e.key) {
        case "F1":
          e.preventDefault();
          setView("song");
          break;
        case "F2":
          e.preventDefault();
          setView("piano-roll");
          break;
        case "F3":
          e.preventDefault();
          setView("mixer");
          break;
        case "F4":
          e.preventDefault();
          setView("patcher");
          break;
        case "F5":
          e.preventDefault();
          setView("perform");
          break;
        case " ":
          e.preventDefault();
          if (transport.isPlaying) stop();
          else play();
          break;
        case "r":
        case "R":
          e.preventDefault();
          toggleRecord();
          break;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [transport.isPlaying, play, stop, toggleRecord, setView]);

  void toggleMute; // used via sendIntent above

  const statusColor =
    wsStatus === "connected"
      ? "#00e5a0"
      : wsStatus === "connecting"
        ? "#ffd54f"
        : "#ef5350";
  const statusLabel =
    wsStatus === "connected"
      ? "Live"
      : wsStatus === "connecting"
        ? "Connecting"
        : "Offline";

  const btn = (
    content: React.ReactNode,
    onClick: () => void,
    active = false,
    title = "",
    color = "#94a3b8",
    danger = false,
  ) => (
    <button
      onClick={onClick}
      title={title}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        gap: 5,
        padding: "4px 10px",
        background: active
          ? danger
            ? "rgba(239,83,80,0.18)"
            : "rgba(77,184,255,0.12)"
          : "transparent",
        border: `1px solid ${active ? (danger ? "rgba(239,83,80,0.4)" : "rgba(77,184,255,0.3)") : "transparent"}`,
        borderRadius: 5,
        color: active ? (danger ? "#ef5350" : "#4db8ff") : color,
        fontSize: 12,
        fontFamily: "var(--font-sans)",
        fontWeight: active ? 600 : 400,
        cursor: "pointer",
        transition: "all 0.1s",
        whiteSpace: "nowrap",
        height: 28,
      }}
    >
      {content}
    </button>
  );

  return (
    <div
      style={{
        height: 48,
        background: "linear-gradient(180deg, #0d1825 0%, #080e18 100%)",
        borderBottom: "1px solid #0f1e2e",
        display: "flex",
        alignItems: "center",
        padding: "0 12px",
        gap: 4,
        flexShrink: 0,
        userSelect: "none",
      }}
    >
      {/* Logo */}
      <div
        style={{
          display: "flex",
          alignItems: "baseline",
          gap: 1,
          marginRight: 12,
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontSize: 15,
            fontWeight: 800,
            letterSpacing: "-0.03em",
            background: "linear-gradient(135deg, #4db8ff, #a78bfa)",
            WebkitBackgroundClip: "text",
            WebkitTextFillColor: "transparent",
          }}
        >
          Aether
        </span>
        <span
          style={{
            fontSize: 15,
            fontWeight: 300,
            color: "#334155",
            letterSpacing: "-0.02em",
          }}
        >
          Studio
        </span>
      </div>

      {/* View tabs */}
      <div style={{ display: "flex", gap: 2, marginRight: 8 }}>
        {VIEWS.map((v) => (
          <button
            key={v.id}
            onClick={() => setView(v.id)}
            title={`${v.label} (${v.shortcut})`}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 5,
              padding: "4px 12px",
              background:
                activeView === v.id ? "rgba(77,184,255,0.1)" : "transparent",
              border: `1px solid ${activeView === v.id ? "rgba(77,184,255,0.25)" : "transparent"}`,
              borderRadius: 5,
              color: activeView === v.id ? "#4db8ff" : "#475569",
              fontSize: 12,
              fontFamily: "var(--font-sans)",
              fontWeight: activeView === v.id ? 600 : 400,
              cursor: "pointer",
              transition: "all 0.1s",
              height: 28,
            }}
          >
            <span style={{ fontSize: 13 }}>{v.icon}</span>
            {v.label}
          </button>
        ))}
      </div>

      {/* Divider */}
      <div
        style={{
          width: 1,
          height: 20,
          background: "#0f1e2e",
          flexShrink: 0,
          margin: "0 6px",
        }}
      />

      {/* Transport */}
      <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
        {/* Stop */}
        {btn("■", stop, false, "Stop (Space)", "#94a3b8")}

        {/* Play */}
        {btn(
          <>
            <span style={{ fontSize: 10 }}>▶</span>{" "}
            {transport.isPlaying ? "Playing" : "Play"}
          </>,
          play,
          transport.isPlaying,
          "Play (Space)",
          "#00e5a0",
        )}

        {/* Record */}
        {btn(
          <>
            <span
              style={{
                fontSize: 10,
                color: transport.isRecording ? "#ef5350" : undefined,
              }}
            >
              ●
            </span>{" "}
            Rec
          </>,
          toggleRecord,
          transport.isRecording,
          "Record",
          "#ef5350",
          true,
        )}
      </div>

      {/* Divider */}
      <div
        style={{
          width: 1,
          height: 20,
          background: "#0f1e2e",
          flexShrink: 0,
          margin: "0 6px",
        }}
      />

      {/* BPM */}
      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
        <span
          style={{ fontSize: 10, color: "#334155", letterSpacing: "0.1em" }}
        >
          BPM
        </span>
        {bpmEditing ? (
          <input
            autoFocus
            type="number"
            value={bpmInput}
            onChange={(e) => setBpmInput(e.target.value)}
            onBlur={() => {
              const v = parseInt(bpmInput, 10);
              if (!isNaN(v)) setBpm(v);
              setBpmEditing(false);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                const v = parseInt(bpmInput, 10);
                if (!isNaN(v)) setBpm(v);
                setBpmEditing(false);
              }
              if (e.key === "Escape") setBpmEditing(false);
            }}
            style={{
              width: 52,
              background: "#0a1520",
              border: "1px solid #4db8ff",
              borderRadius: 4,
              color: "#4db8ff",
              fontSize: 14,
              fontWeight: 700,
              fontFamily: "monospace",
              textAlign: "center",
              padding: "2px 4px",
              outline: "none",
            }}
          />
        ) : (
          <button
            onClick={() => {
              setBpmInput(String(transport.bpm));
              setBpmEditing(true);
            }}
            style={{
              background: "transparent",
              border: "1px solid transparent",
              borderRadius: 4,
              color: "#e0e8f0",
              fontSize: 14,
              fontWeight: 700,
              fontFamily: "monospace",
              cursor: "text",
              padding: "2px 6px",
              minWidth: 52,
              textAlign: "center",
            }}
          >
            {transport.bpm}
          </button>
        )}
        <button
          onClick={handleTap}
          style={{
            padding: "3px 8px",
            background: "transparent",
            border: "1px solid #1a2a3a",
            borderRadius: 4,
            color: "#475569",
            fontSize: 10,
            cursor: "pointer",
            fontFamily: "var(--font-sans)",
          }}
        >
          TAP
        </button>
      </div>

      {/* Time signature */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 2,
          padding: "2px 8px",
          background: "#0a1520",
          border: "1px solid #1a2a3a",
          borderRadius: 4,
          fontFamily: "monospace",
          fontSize: 12,
          color: "#475569",
        }}
      >
        {transport.timeSignatureNum}/{transport.timeSignatureDen}
      </div>

      {/* Spacer */}
      <div style={{ flex: 1 }} />

      {/* Panel toggles */}
      {btn(
        <>
          <span style={{ fontSize: 11 }}>☰</span> Browser
        </>,
        () => setBrowserOpen(!browserOpen),
        browserOpen,
        "Toggle browser panel",
      )}
      {btn(
        <>
          <span style={{ fontSize: 11 }}>⊟</span> Props
        </>,
        () => setPropertiesOpen(!propertiesOpen),
        propertiesOpen,
        "Toggle properties panel",
      )}

      {/* Divider */}
      <div
        style={{
          width: 1,
          height: 20,
          background: "#0f1e2e",
          flexShrink: 0,
          margin: "0 6px",
        }}
      />

      {/* Recorder */}
      {btn(
        <>
          <span>🎙</span> Record Instrument
        </>,
        onOpenRecorder,
        false,
        "Record your own instrument samples",
        "#a78bfa",
      )}

      {/* Divider */}
      <div
        style={{
          width: 1,
          height: 20,
          background: "#0f1e2e",
          flexShrink: 0,
          margin: "0 6px",
        }}
      />

      {/* VU */}
      <div
        style={{ display: "flex", alignItems: "center", gap: 3, height: 16 }}
      >
        {[0.4, 0.7, 1, 0.6, 0.3].map((h, i) => (
          <div
            key={i}
            style={{
              width: 3,
              height: audioActive ? `${h * 16}px` : "3px",
              background: audioActive
                ? `hsl(${160 - i * 15}, 80%, 55%)`
                : "#1a2a3a",
              borderRadius: 1,
              transition: "height 0.08s",
            }}
          />
        ))}
      </div>

      {/* Status */}
      <div
        style={{ display: "flex", alignItems: "center", gap: 5, marginLeft: 8 }}
      >
        <div
          style={{
            width: 7,
            height: 7,
            borderRadius: "50%",
            background: statusColor,
            boxShadow:
              wsStatus === "connected" ? `0 0 6px ${statusColor}` : "none",
          }}
        />
        <span
          style={{
            fontSize: 11,
            color: wsStatus === "connected" ? statusColor : "#475569",
            fontWeight: 500,
          }}
        >
          {statusLabel}
        </span>
      </div>
    </div>
  );
}
