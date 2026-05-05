/**
 * DawShell — the main studio layout.
 *
 * ┌─────────────────────────────────────────────────────┐
 * │  TopBar: transport + view tabs + status             │  48px
 * ├──────────┬──────────────────────────────────────────┤
 * │          │                                          │
 * │ Browser  │   Main Workspace                         │
 * │  220px   │   (Song / Piano Roll / Mixer / Patcher)  │
 * │          │                                          │
 * ├──────────┴──────────────────────────────────────────┤
 * │  Properties / Automation                            │  180px
 * └─────────────────────────────────────────────────────┘
 */

import { useCallback, useRef, useEffect, useState } from "react";
import { useDawStore } from "./store/dawStore";
import { DawTopBar } from "./components/DawTopBar";
import { DawBrowser } from "./components/DawBrowser";
import { SongView } from "./views/SongView";
import { PianoRollView } from "./views/PianoRollView";
import { MixerView } from "./views/MixerView";
import { PatcherView } from "./views/PatcherView";
import { PropertiesPanel } from "./components/PropertiesPanel";
import { useEngineSocket } from "../hooks/useEngineSocket";
import { useProjectSave } from "../hooks/useProjectSave";
import { InstrumentRecorder } from "../components/InstrumentRecorder";

export function DawShell() {
  useEngineSocket();
  useProjectSave();

  const activeView = useDawStore((s) => s.activeView);
  const browserOpen = useDawStore((s) => s.browserOpen);
  const propertiesOpen = useDawStore((s) => s.propertiesOpen);
  const browserWidth = useDawStore((s) => s.browserWidth);
  const propertiesHeight = useDawStore((s) => s.propertiesHeight);
  const setBrowserWidth = useDawStore((s) => s.setBrowserWidth);
  const setPropertiesHeight = useDawStore((s) => s.setPropertiesHeight);
  const transport = useDawStore((s) => s.transport);
  const setTransport = useDawStore((s) => s.setTransport);

  const [showRecorder, setShowRecorder] = useState(false);

  // Advance playhead while playing
  useEffect(() => {
    if (!transport.isPlaying) return;
    const interval = setInterval(() => {
      setTransport({
        playheadBeat: transport.playheadBeat + transport.bpm / 60 / 10,
      });
    }, 100);
    return () => clearInterval(interval);
  }, [
    transport.isPlaying,
    transport.bpm,
    transport.playheadBeat,
    setTransport,
  ]);

  // ── Resize handles ────────────────────────────────────────────────────────
  const browserResizing = useRef(false);
  const propsResizing = useRef(false);

  const onBrowserResizeStart = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      browserResizing.current = true;
      const startX = e.clientX;
      const startW = browserWidth;
      const onMove = (ev: MouseEvent) => {
        if (!browserResizing.current) return;
        setBrowserWidth(startW + ev.clientX - startX);
      };
      const onUp = () => {
        browserResizing.current = false;
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
      };
      window.addEventListener("mousemove", onMove);
      window.addEventListener("mouseup", onUp);
    },
    [browserWidth, setBrowserWidth],
  );

  const onPropsResizeStart = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      propsResizing.current = true;
      const startY = e.clientY;
      const startH = propertiesHeight;
      const onMove = (ev: MouseEvent) => {
        if (!propsResizing.current) return;
        setPropertiesHeight(startH - (ev.clientY - startY));
      };
      const onUp = () => {
        propsResizing.current = false;
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
      };
      window.addEventListener("mousemove", onMove);
      window.addEventListener("mouseup", onUp);
    },
    [propertiesHeight, setPropertiesHeight],
  );

  const renderView = () => {
    switch (activeView) {
      case "song":
        return <SongView />;
      case "piano-roll":
        return <PianoRollView />;
      case "mixer":
        return <MixerView />;
      case "patcher":
        return <PatcherView />;
    }
  };

  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        display: "flex",
        flexDirection: "column",
        background: "var(--bg-void)",
        overflow: "hidden",
        fontFamily: "var(--font-sans)",
      }}
    >
      {/* Top bar */}
      <DawTopBar onOpenRecorder={() => setShowRecorder(true)} />

      {/* Main area */}
      <div
        style={{ flex: 1, display: "flex", overflow: "hidden", minHeight: 0 }}
      >
        {/* Browser panel */}
        {browserOpen && (
          <>
            <DawBrowser width={browserWidth} />
            {/* Resize handle */}
            <div
              onMouseDown={onBrowserResizeStart}
              style={{
                width: 4,
                background: "transparent",
                cursor: "col-resize",
                flexShrink: 0,
                position: "relative",
                zIndex: 10,
              }}
              onMouseEnter={(e) =>
                (e.currentTarget.style.background = "rgba(77,184,255,0.3)")
              }
              onMouseLeave={(e) =>
                (e.currentTarget.style.background = "transparent")
              }
            />
          </>
        )}

        {/* Workspace + properties */}
        <div
          style={{
            flex: 1,
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
            minWidth: 0,
          }}
        >
          {/* Main workspace */}
          <div style={{ flex: 1, overflow: "hidden", minHeight: 0 }}>
            {renderView()}
          </div>

          {/* Properties resize handle */}
          {propertiesOpen && (
            <div
              onMouseDown={onPropsResizeStart}
              style={{
                height: 4,
                background: "transparent",
                cursor: "row-resize",
                flexShrink: 0,
              }}
              onMouseEnter={(e) =>
                (e.currentTarget.style.background = "rgba(77,184,255,0.3)")
              }
              onMouseLeave={(e) =>
                (e.currentTarget.style.background = "transparent")
              }
            />
          )}

          {/* Properties panel */}
          {propertiesOpen && <PropertiesPanel height={propertiesHeight} />}
        </div>
      </div>

      {/* Instrument recorder modal */}
      {showRecorder && (
        <InstrumentRecorder onClose={() => setShowRecorder(false)} />
      )}
    </div>
  );
}
