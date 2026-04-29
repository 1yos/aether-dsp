import { useRef, useState, useEffect } from "react";
import { useEngineStore, WsStatus } from "../store/engineStore";
import { useModuleStore } from "../../modules/useModuleStore";

const STATUS_COLOR: Record<WsStatus, string> = {
  disconnected: "#ef5350",
  connecting: "#ffd54f",
  connected: "#00e5a0",
  error: "#ff7043",
};

const STATUS_LABEL: Record<WsStatus, string> = {
  disconnected: "Offline",
  connecting: "Connecting",
  connected: "Live",
  error: "Error",
};

export function TransportBar({
  onOpenInstrumentMaker,
}: {
  onOpenInstrumentMaker?: () => void;
}) {
  const addModule = useModuleStore((s) => s.addModule);

  const muted = useEngineStore((s) => s.muted);
  const toggleMute = useEngineStore((s) => s.toggleMute);
  const clearGraph = useEngineStore((s) => s.clearGraph);
  const loadPatch = useEngineStore((s) => s.loadPatch);
  const wsStatus = useEngineStore((s) => s.wsStatus);
  const audioActive = useEngineStore((s) => s.audioActive);
  const selectedNodeId = useEngineStore((s) => s.selectedNodeId);
  const removeNode = useEngineStore((s) => s.removeNode);
  const fileRef = useRef<HTMLInputElement>(null);

  const canUndo = useEngineStore((s) => s.canUndo);
  const canRedo = useEngineStore((s) => s.canRedo);
  const undo = useEngineStore((s) => s.undo);
  const redo = useEngineStore((s) => s.redo);

  const midiPorts = useEngineStore((s) => s.midiPorts);
  const connectedMidiPort = useEngineStore((s) => s.connectedMidiPort);
  const listMidiPorts = useEngineStore((s) => s.listMidiPorts);
  const connectMidiPort = useEngineStore((s) => s.connectMidiPort);
  const setConnectedMidiPort = useEngineStore((s) => s.setConnectedMidiPort);
  const [midiDropdownOpen, setMidiDropdownOpen] = useState(false);

  const isRecording = useEngineStore((s) => s.isRecording);
  const startRecording = useEngineStore((s) => s.startRecording);
  const stopRecording = useEngineStore((s) => s.stopRecording);
  const [elapsedSecs, setElapsedSecs] = useState(0);

  // Tick elapsed timer while recording
  useEffect(() => {
    if (!isRecording) {
      setElapsedSecs(0);
      return;
    }
    setElapsedSecs(0);
    const id = setInterval(() => setElapsedSecs((s) => s + 1), 1000);
    return () => clearInterval(id);
  }, [isRecording]);

  const handleLoadFile = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      try {
        const json = JSON.parse(ev.target?.result as string);
        if (json.nodes && json.connections !== undefined) {
          loadPatch({
            nodes: json.nodes.map(
              (n: {
                id: string;
                type: string;
                params?: Record<string, number>;
              }) => ({
                id: n.id,
                type: n.type,
                params: n.params ?? {},
              }),
            ),
            connections: json.connections,
            output_node: json.output_node,
          });
        }
      } catch {
        alert("Could not parse patch file.");
      }
    };
    reader.readAsText(file);
    e.target.value = "";
  };

  const statusColor = STATUS_COLOR[wsStatus];
  const isConnected = wsStatus === "connected";

  return (
    <div
      style={{
        height: 52,
        background: "linear-gradient(180deg, #0c1420 0%, #080e18 100%)",
        borderBottom: "1px solid #0f1e2e",
        display: "flex",
        alignItems: "center",
        padding: "0 20px",
        gap: 8,
        flexShrink: 0,
        position: "relative",
      }}
    >
      <style>{`@keyframes recPulse { 0%,100% { opacity:1; } 50% { opacity:0.55; } }`}</style>
      {/* Logo */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 0,
          marginRight: 16,
        }}
      >
        <span
          style={{
            fontFamily: "'Inter', system-ui, sans-serif",
            fontSize: 16,
            fontWeight: 800,
            letterSpacing: "-0.03em",
            background: "linear-gradient(135deg, #38bdf8, #818cf8)",
            WebkitBackgroundClip: "text",
            WebkitTextFillColor: "transparent",
          }}
        >
          Aether
        </span>
        <span
          style={{
            fontFamily: "'Inter', system-ui, sans-serif",
            fontSize: 16,
            fontWeight: 300,
            letterSpacing: "-0.02em",
            color: "#475569",
            marginLeft: 1,
          }}
        >
          Studio
        </span>
        <span
          style={{
            fontSize: 8,
            color: "#1e2d3d",
            marginLeft: 6,
            fontFamily: "monospace",
          }}
        >
          v2.1-catalog
        </span>
      </div>

      <Divider />

      {/* Instrument Maker */}
      {onOpenInstrumentMaker && (
        <TBtn
          onClick={onOpenInstrumentMaker}
          accent="#a78bfa"
          icon="🎹"
          label="Instrument Maker"
        />
      )}

      {/* Catalog — PROMINENT */}
      <button
        onClick={() => addModule("InstrumentBrowser")}
        title="Open Instrument Catalog (60 world instruments)"
        style={{
          display: "flex",
          alignItems: "center",
          gap: 7,
          padding: "6px 16px",
          background:
            "linear-gradient(135deg, rgba(56,189,248,0.15), rgba(129,140,248,0.12))",
          border: "1px solid rgba(56,189,248,0.4)",
          borderRadius: 6,
          color: "#38bdf8",
          fontFamily: "'Inter', system-ui, sans-serif",
          fontSize: 12,
          fontWeight: 700,
          cursor: "pointer",
          letterSpacing: "0.02em",
          transition: "all 0.15s",
          boxShadow: "0 0 12px rgba(56,189,248,0.2)",
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.background =
            "linear-gradient(135deg, rgba(56,189,248,0.25), rgba(129,140,248,0.18))";
          e.currentTarget.style.boxShadow = "0 0 20px rgba(56,189,248,0.35)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background =
            "linear-gradient(135deg, rgba(56,189,248,0.15), rgba(129,140,248,0.12))";
          e.currentTarget.style.boxShadow = "0 0 12px rgba(56,189,248,0.2)";
        }}
      >
        <span style={{ fontSize: 14 }}>🌍</span>
        Catalog
        <span
          style={{
            fontSize: 9,
            background: "rgba(56,189,248,0.2)",
            padding: "1px 5px",
            borderRadius: 3,
            fontWeight: 600,
            marginLeft: 2,
          }}
        >
          60
        </span>
      </button>

      <Divider />

      {/* Play / Mute */}
      <button
        onClick={toggleMute}
        title={muted ? "Resume audio" : "Mute audio"}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 7,
          padding: "5px 14px",
          background: muted ? "rgba(239,83,80,0.12)" : "rgba(0,229,160,0.08)",
          border: `1px solid ${muted ? "rgba(239,83,80,0.4)" : "rgba(0,229,160,0.25)"}`,
          borderRadius: 6,
          color: muted ? "#ef5350" : "#00e5a0",
          fontFamily: "'Inter', system-ui, sans-serif",
          fontSize: 12,
          fontWeight: 600,
          cursor: "pointer",
          letterSpacing: "0.02em",
          transition: "all 0.15s",
        }}
      >
        <span style={{ fontSize: 10 }}>{muted ? "⏸" : "▶"}</span>
        {muted ? "Paused" : "Playing"}
      </button>

      {/* Load patch */}
      <TBtn
        onClick={() => fileRef.current?.click()}
        icon="📂"
        label="Load Patch"
      />
      <input
        ref={fileRef}
        type="file"
        accept=".json"
        style={{ display: "none" }}
        onChange={handleLoadFile}
      />

      {/* Delete selected */}
      {selectedNodeId && (
        <TBtn
          onClick={() => removeNode(selectedNodeId)}
          icon="⌫"
          label="Delete"
          accent="#ef5350"
          danger
        />
      )}

      {/* Clear */}
      <TBtn
        onClick={() => {
          if (confirm("Clear the entire graph?")) clearGraph();
        }}
        icon="✕"
        label="Clear"
        accent="#ff7043"
        danger
      />

      {/* Undo */}
      <TBtn onClick={undo} icon="↩" label="Undo" disabled={!canUndo} />

      {/* Redo */}
      <TBtn onClick={redo} icon="↪" label="Redo" disabled={!canRedo} />

      <Divider />

      {/* Record */}
      {!isRecording ? (
        <button
          onClick={async () => {
            let path = "";
            if ("showSaveFilePicker" in window) {
              try {
                const handle = await (
                  window as Window & {
                    showSaveFilePicker: (
                      opts: object,
                    ) => Promise<{ name: string }>;
                  }
                ).showSaveFilePicker({
                  suggestedName: "recording.wav",
                  types: [
                    {
                      description: "WAV Audio",
                      accept: { "audio/wav": [".wav"] },
                    },
                  ],
                });
                path = handle.name;
              } catch {
                return; // user cancelled
              }
            } else {
              const p = prompt("Output file path:", "recording.wav");
              if (!p) return;
              path = p;
            }
            startRecording(path);
          }}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            padding: "5px 14px",
            background: "rgba(239,83,80,0.08)",
            border: "1px solid rgba(239,83,80,0.3)",
            borderRadius: 6,
            color: "#ef5350",
            fontFamily: "'Inter', system-ui, sans-serif",
            fontSize: 12,
            fontWeight: 600,
            cursor: "pointer",
            letterSpacing: "0.02em",
          }}
        >
          <span style={{ fontSize: 10 }}>●</span>
          REC
        </button>
      ) : (
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <button
            onClick={stopRecording}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 6,
              padding: "5px 14px",
              background: "rgba(239,83,80,0.18)",
              border: "1px solid rgba(239,83,80,0.5)",
              borderRadius: 6,
              color: "#ef5350",
              fontFamily: "'Inter', system-ui, sans-serif",
              fontSize: 12,
              fontWeight: 600,
              cursor: "pointer",
              letterSpacing: "0.02em",
              animation: "recPulse 1s ease-in-out infinite",
            }}
          >
            <span style={{ fontSize: 10 }}>■</span>
            Stop
          </button>
          <span
            style={{
              color: "#ef5350",
              fontFamily: "monospace",
              fontSize: 12,
              minWidth: 32,
            }}
          >
            {Math.floor(elapsedSecs / 60)}:
            {String(elapsedSecs % 60).padStart(2, "0")}
          </span>
        </div>
      )}

      <div style={{ flex: 1 }} />

      {/* Audio VU */}
      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
        <div
          style={{
            display: "flex",
            gap: 2,
            alignItems: "flex-end",
            height: 16,
          }}
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
                transition: "height 0.08s, background 0.1s",
              }}
            />
          ))}
        </div>
        <span
          style={{ color: "#334155", fontSize: 10, fontFamily: "monospace" }}
        >
          {audioActive ? "AUDIO" : "SILENT"}
        </span>
      </div>

      <Divider />

      {/* Connection status */}
      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
        <div
          style={{
            width: 7,
            height: 7,
            borderRadius: "50%",
            background: statusColor,
            boxShadow: isConnected ? `0 0 8px ${statusColor}` : "none",
            transition: "all 0.3s",
          }}
        />
        <span
          style={{
            color: isConnected ? statusColor : "#475569",
            fontSize: 11,
            fontFamily: "'Inter', system-ui, sans-serif",
            fontWeight: 500,
          }}
        >
          {STATUS_LABEL[wsStatus]}
        </span>
      </div>

      <Divider />

      {/* MIDI selector */}
      <div style={{ position: "relative" }}>
        <button
          onClick={() => {
            listMidiPorts();
            setMidiDropdownOpen((o) => !o);
          }}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            padding: "4px 10px",
            background: connectedMidiPort
              ? "rgba(0,229,160,0.06)"
              : "transparent",
            border: `1px solid ${connectedMidiPort ? "rgba(0,229,160,0.2)" : "#1a2a3a"}`,
            borderRadius: 6,
            color: connectedMidiPort ? "#00e5a0" : "#475569",
            fontSize: 11,
            fontFamily: "'Inter', system-ui, sans-serif",
            cursor: "pointer",
          }}
        >
          <span style={{ fontSize: 12 }}>♪</span>
          <span>
            {connectedMidiPort
              ? connectedMidiPort.slice(0, 16) +
                (connectedMidiPort.length > 16 ? "…" : "")
              : "No MIDI"}
          </span>
          <span style={{ opacity: 0.4, fontSize: 9 }}>▾</span>
        </button>

        {midiDropdownOpen && (
          <div
            style={{
              position: "absolute",
              top: "calc(100% + 6px)",
              right: 0,
              background: "#0c1420",
              border: "1px solid #1e2d3d",
              borderRadius: 8,
              minWidth: 220,
              zIndex: 1000,
              boxShadow: "0 8px 32px rgba(0,0,0,0.6)",
              overflow: "hidden",
            }}
          >
            {midiPorts.length === 0 ? (
              <div
                style={{ padding: "10px 14px", color: "#475569", fontSize: 12 }}
              >
                No MIDI ports found
              </div>
            ) : (
              midiPorts.map((port, i) => (
                <button
                  key={i}
                  onClick={() => {
                    connectMidiPort(i);
                    setConnectedMidiPort(port);
                    setMidiDropdownOpen(false);
                  }}
                  style={{
                    display: "block",
                    width: "100%",
                    textAlign: "left",
                    background:
                      connectedMidiPort === port
                        ? "rgba(0,229,160,0.08)"
                        : "transparent",
                    color: connectedMidiPort === port ? "#00e5a0" : "#94a3b8",
                    border: "none",
                    borderBottom: "1px solid #0f1e2e",
                    padding: "9px 14px",
                    cursor: "pointer",
                    fontSize: 12,
                  }}
                >
                  {connectedMidiPort === port ? "✓  " : "    "}
                  {port}
                </button>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function Divider() {
  return (
    <div
      style={{ width: 1, height: 20, background: "#0f1e2e", flexShrink: 0 }}
    />
  );
}

function TBtn({
  onClick,
  icon,
  label,
  accent,
  danger,
  disabled,
}: {
  onClick: () => void;
  icon: string;
  label: string;
  accent?: string;
  danger?: boolean;
  disabled?: boolean;
}) {
  const [hovered, setHovered] = useState(false);
  const color = accent ?? "#94a3b8";
  return (
    <button
      onClick={disabled ? undefined : onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 6,
        padding: "5px 12px",
        background:
          !disabled && hovered
            ? danger
              ? `rgba(${accent === "#ef5350" ? "239,83,80" : "255,112,67"},0.12)`
              : "rgba(255,255,255,0.05)"
            : "transparent",
        border: `1px solid ${!disabled && hovered ? (danger ? `${color}60` : "#1e2d3d") : "transparent"}`,
        borderRadius: 6,
        color: disabled ? "#64748b" : hovered ? color : "#64748b",
        fontFamily: "'Inter', system-ui, sans-serif",
        fontSize: 12,
        cursor: disabled ? "not-allowed" : "pointer",
        transition: "all 0.12s",
        whiteSpace: "nowrap",
        opacity: disabled ? 0.4 : 1,
      }}
    >
      <span style={{ fontSize: 11 }}>{icon}</span>
      {label}
    </button>
  );
}
