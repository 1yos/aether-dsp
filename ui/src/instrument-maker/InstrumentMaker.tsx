/**
 * InstrumentMaker — the full instrument creation tool.
 *
 * Layout:
 *   ┌─────────────────────────────────────────────────────────┐
 *   │  Header: name, save/load/export buttons                 │
 *   ├──────────────┬──────────────────────────────────────────┤
 *   │  Left panel  │  Main area                               │
 *   │  - Meta      │  - Keyboard map (full width)             │
 *   │  - ADSR      │  - Zone list + Zone editor               │
 *   │  - Tuning    │                                          │
 *   ├──────────────┤                                          │
 *   │  Timbre      │                                          │
 *   │  Transfer    │                                          │
 *   └──────────────┴──────────────────────────────────────────┘
 */
import React, { useRef, useState, useCallback } from "react";
import { useInstrumentStore } from "./store/instrumentStore";
import { useEngineStore } from "../studio/store/engineStore";
import { KeyboardMap } from "./components/KeyboardMap";
import { ZoneEditor } from "./components/ZoneEditor";
import { ZoneList } from "./components/ZoneList";
import { InstrumentMeta } from "./components/InstrumentMeta";
import { TimbrePanel } from "./components/TimbrePanel";
import { SampleZone } from "./types";
import { loadAudioBuffer, useAudioPreview } from "./hooks/useAudioPreview";

type LeftTab = "meta" | "timbre";

export function InstrumentMaker({ onBack }: { onBack: () => void }) {
  const { instrument, addZone, loadInstrument, exportJson, isDirty } =
    useInstrumentStore();
  const { sendIntent } = useEngineStore();
  const [leftTab, setLeftTab] = useState<LeftTab>("meta");
  const [draggingOver, setDraggingOver] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const loadInputRef = useRef<HTMLInputElement>(null);
  const { previewNote: playNote } = useAudioPreview();

  // Handle audio file drop onto the keyboard area
  const handleFileDrop = useCallback(
    async (files: FileList, defaultNote = 60) => {
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        if (!file.type.startsWith("audio/")) continue;

        try {
          const audioBuffer = await loadAudioBuffer(file);
          const dataUrl = URL.createObjectURL(file);

          // Auto-detect root note from filename (e.g. "krar_A3.wav" → A3 = 57)
          const rootNote = detectNoteFromFilename(file.name) ?? defaultNote;

          const zone: SampleZone = {
            id: `zone-${Date.now()}-${i}`,
            file_path: file.name,
            fileName: file.name,
            root_note: rootNote,
            note_low: Math.max(0, rootNote - 6),
            note_high: Math.min(127, rootNote + 6),
            velocity_low: 0,
            velocity_high: 127,
            articulation: { type: "OneShot" },
            volume_db: 0,
            tune_cents: 0,
            release_file: null,
            audioDataUrl: dataUrl,
            audioBuffer,
          };

          addZone(zone);
        } catch (err) {
          console.error("Failed to load audio file:", err);
        }
      }
    },
    [addZone],
  );

  const handleKeyboardDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDraggingOver(false);
      handleFileDrop(e.dataTransfer.files);
    },
    [handleFileDrop],
  );

  const handleExport = useCallback(() => {
    // TODO (Tauri): When running inside Tauri (`window.__TAURI__` is defined),
    // replace this browser download link with the native file save dialog:
    //   import { save } from "@tauri-apps/api/dialog";
    //   import { writeTextFile } from "@tauri-apps/api/fs";
    //   const filePath = await save({ defaultPath: `${name}.aether-instrument`, filters: [{ name: "Instrument", extensions: ["aether-instrument", "json"] }] });
    //   if (filePath) await writeTextFile(filePath, json);
    const json = exportJson();
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${instrument.name.replace(/\s+/g, "-").toLowerCase()}.aether-instrument`;
    a.click();
    URL.revokeObjectURL(url);
  }, [exportJson, instrument.name]);

  const handleExportClap = useCallback(() => {
    // Task 20.6: Send ExportClap intent to the host.
    // The host will build the CLAP plugin and respond with clap_exported (success)
    // or error (failure).  Both are handled in useWebSocket.ts.
    const json = exportJson();
    // Prompt the user for an output path (browser fallback — Tauri uses native dialog).
    const outputPath = window.prompt(
      "Enter the output path for the CLAP plugin (e.g. C:/Plugins/MyInstrument.clap):",
      `${instrument.name.replace(/\s+/g, "-").toLowerCase()}.clap`,
    );
    if (!outputPath) return;
    sendIntent?.({
      type: "export_clap",
      node_id: 0,
      generation: 0,
      instrument_json: json,
      output_path: outputPath,
    });
  }, [exportJson, instrument.name, sendIntent]);

  const handleLoad = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = (ev) => {
        try {
          const inst = JSON.parse(ev.target?.result as string);
          // Validate required fields
          if (!inst.name || !inst.zones || !inst.tuning) {
            alert(
              "Invalid instrument file: missing required fields (name, zones, tuning).",
            );
            return;
          }
          loadInstrument(inst);
        } catch {
          alert("Invalid instrument file.");
        }
      };
      reader.readAsText(file);
    },
    [loadInstrument],
  );

  const handleNoteClick = useCallback(
    (note: number) => {
      playNote(instrument, note);
    },
    [instrument, playNote],
  );

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        width: "100vw",
        height: "100vh",
        background: "#060c12",
        color: "#e0e8f0",
        fontFamily: "system-ui, sans-serif",
      }}
    >
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 12,
          padding: "8px 16px",
          background: "#080e14",
          borderBottom: "1px solid #1a2a3a",
          flexShrink: 0,
        }}
      >
        <button
          onClick={onBack}
          style={{
            background: "none",
            border: "1px solid #1a2a3a",
            borderRadius: 4,
            color: "#7a9ab5",
            padding: "4px 10px",
            fontSize: 12,
            cursor: "pointer",
          }}
        >
          ← Studio
        </button>

        <div style={{ fontSize: 14, fontWeight: 700, color: "#4fc3f7" }}>
          🎹 Instrument Maker
        </div>

        <div
          style={{
            fontSize: 13,
            color: isDirty ? "#ffb74d" : "#4a6a8a",
            flex: 1,
          }}
        >
          {instrument.name}
          {isDirty && " •"}
        </div>

        <div style={{ display: "flex", gap: 8 }}>
          <button
            onClick={() => fileInputRef.current?.click()}
            style={btnStyle("#1a3a1a", "#2a6a2a", "#a5d6a7")}
          >
            + Add Sample
          </button>
          <input
            ref={fileInputRef}
            type="file"
            accept="audio/*"
            multiple
            style={{ display: "none" }}
            onChange={(e) => {
              if (e.target.files) handleFileDrop(e.target.files);
            }}
          />

          <button
            onClick={() => loadInputRef.current?.click()}
            style={btnStyle("#1a2a3a", "#2a4a6a", "#7a9ab5")}
          >
            Load
          </button>
          <input
            ref={loadInputRef}
            type="file"
            accept=".aether-instrument,.json"
            style={{ display: "none" }}
            onChange={handleLoad}
          />

          <button
            onClick={handleExport}
            style={btnStyle("#1a3a5a", "#2a5a8a", "#4fc3f7")}
          >
            Export
          </button>

          <button
            onClick={handleExportClap}
            style={btnStyle("#2a1a3a", "#4a2a6a", "#ce93d8")}
            title="Export this instrument as a CLAP plugin (coming in a future update)"
          >
            Export CLAP
          </button>
        </div>
      </div>

      {/* Body */}
      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        {/* Left panel */}
        <div
          style={{
            width: 280,
            flexShrink: 0,
            borderRight: "1px solid #1a2a3a",
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
          }}
        >
          {/* Tab bar */}
          <div
            style={{
              display: "flex",
              borderBottom: "1px solid #1a2a3a",
              flexShrink: 0,
            }}
          >
            {(["meta", "timbre"] as LeftTab[]).map((tab) => (
              <button
                key={tab}
                onClick={() => setLeftTab(tab)}
                style={{
                  flex: 1,
                  padding: "8px 0",
                  background: leftTab === tab ? "#0f1923" : "transparent",
                  border: "none",
                  borderBottom:
                    leftTab === tab
                      ? "2px solid #4fc3f7"
                      : "2px solid transparent",
                  color: leftTab === tab ? "#4fc3f7" : "#4a6a8a",
                  fontSize: 11,
                  cursor: "pointer",
                  textTransform: "capitalize",
                }}
              >
                {tab === "meta" ? "Instrument" : "Timbre Transfer"}
              </button>
            ))}
          </div>

          <div style={{ flex: 1, overflowY: "auto" }}>
            {leftTab === "meta" ? <InstrumentMeta /> : <TimbrePanel />}
          </div>
        </div>

        {/* Main area */}
        <div
          style={{
            flex: 1,
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
          }}
        >
          {/* Keyboard */}
          <div
            style={{
              padding: "12px 12px 8px",
              borderBottom: "1px solid #1a2a3a",
              flexShrink: 0,
              background: draggingOver ? "#0a1a2a" : "transparent",
              transition: "background 0.15s",
            }}
            onDragOver={(e) => {
              e.preventDefault();
              setDraggingOver(true);
            }}
            onDragLeave={() => setDraggingOver(false)}
            onDrop={handleKeyboardDrop}
          >
            <div
              style={{
                fontSize: 11,
                color: "#4a6a8a",
                marginBottom: 6,
                display: "flex",
                justifyContent: "space-between",
              }}
            >
              <span>
                {instrument.zones.length} zone
                {instrument.zones.length !== 1 ? "s" : ""} •{" "}
                {instrument.tuning.name}
              </span>
              <span>Drop audio files here or click keys to preview</span>
            </div>
            <KeyboardMap onNoteClick={handleNoteClick} />
          </div>

          {/* Zone list + editor */}
          <div
            style={{
              flex: 1,
              display: "flex",
              overflow: "hidden",
            }}
          >
            {/* Zone list */}
            <div
              style={{
                width: 240,
                borderRight: "1px solid #1a2a3a",
                display: "flex",
                flexDirection: "column",
                overflow: "hidden",
              }}
            >
              <div
                style={{
                  padding: "8px 10px",
                  fontSize: 11,
                  color: "#7a9ab5",
                  borderBottom: "1px solid #1a2a3a",
                  flexShrink: 0,
                }}
              >
                Zones ({instrument.zones.length})
              </div>
              <div style={{ flex: 1, overflowY: "auto" }}>
                <ZoneList />
              </div>
            </div>

            {/* Zone editor */}
            <div style={{ flex: 1, overflowY: "auto" }}>
              <ZoneEditor />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function btnStyle(
  bg: string,
  border: string,
  color: string,
): React.CSSProperties {
  return {
    background: bg,
    border: `1px solid ${border}`,
    borderRadius: 4,
    color,
    padding: "4px 10px",
    fontSize: 11,
    cursor: "pointer",
  };
}

const NOTE_NAMES: Record<string, number> = {
  C: 0,
  D: 2,
  E: 4,
  F: 5,
  G: 7,
  A: 9,
  B: 11,
};

function detectNoteFromFilename(filename: string): number | null {
  // Match patterns like A3, C#4, Bb2, etc.
  const match = filename.match(/([A-G])(#|b)?(\d)/i);
  if (!match) return null;
  const noteName = match[1].toUpperCase();
  const accidental = match[2] ?? "";
  const octave = parseInt(match[3]);
  const base = NOTE_NAMES[noteName];
  if (base === undefined) return null;
  const semitone =
    base + (accidental === "#" ? 1 : accidental === "b" ? -1 : 0);
  return (octave + 1) * 12 + semitone;
}
