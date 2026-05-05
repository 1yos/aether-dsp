import { useCallback, useRef, useState } from "react";
import { useDawStore } from "../store/dawStore";
import type { Clip } from "../store/dawStore";

const HEADER_W = 200;
const RULER_H = 32;
const BEAT_W_BASE = 32;
const TOTAL_BEATS = 256;

const TRACK_ICONS: Record<string, string> = {
  instrument: "🎹",
  audio: "🎵",
  bus: "🔀",
  master: "⊟",
};

const EFFECT_COLORS: Record<string, string> = {
  EQ: "#4db8ff",
  Compressor: "#a78bfa",
  Reverb: "#34d399",
  Delay: "#fbbf24",
  Filter: "#f97316",
};

export function SongView() {
  const tracks = useDawStore((s) => s.tracks);
  const transport = useDawStore((s) => s.transport);
  const selectedTrackId = useDawStore((s) => s.selectedTrackId);
  const selectedClipId = useDawStore((s) => s.selectedClipId);
  const addTrack = useDawStore((s) => s.addTrack);
  const removeTrack = useDawStore((s) => s.removeTrack);
  const updateTrack = useDawStore((s) => s.updateTrack);
  const selectTrack = useDawStore((s) => s.selectTrack);
  const addClip = useDawStore((s) => s.addClip);
  const removeClip = useDawStore((s) => s.removeClip);
  const selectClip = useDawStore((s) => s.selectClip);
  const moveClip = useDawStore((s) => s.moveClip);
  const resizeClip = useDawStore((s) => s.resizeClip);
  const openPianoRoll = useDawStore((s) => s.openPianoRoll);
  const addEffect = useDawStore((s) => s.addEffect);
  const removeEffect = useDawStore((s) => s.removeEffect);
  const toggleEffect = useDawStore((s) => s.toggleEffect);
  const setTransport = useDawStore((s) => s.setTransport);

  const [zoom, setZoom] = useState(1);
  const [snap, setSnap] = useState<
    "bar" | "beat" | "half" | "quarter" | "eighth" | "off"
  >("beat");
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    type: "track" | "clip";
    id: string;
  } | null>(null);
  const [renamingTrackId, setRenamingTrackId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const beatW = BEAT_W_BASE * zoom;
  const scrollRef = useRef<HTMLDivElement>(null);
  const dragging = useRef<{
    clipId: string;
    startX: number;
    startBeat: number;
  } | null>(null);
  const resizing = useRef<{
    clipId: string;
    startX: number;
    startLen: number;
  } | null>(null);
  const trackResizing = useRef<{
    trackId: string;
    startY: number;
    startH: number;
  } | null>(null);

  const snapBeat = useCallback(
    (beat: number) => {
      const snaps = {
        bar: transport.timeSignatureNum,
        beat: 1,
        half: 0.5,
        quarter: 0.25,
        eighth: 0.125,
        off: 0,
      };
      const s = snaps[snap];
      return s > 0 ? Math.round(beat / s) * s : beat;
    },
    [snap, transport.timeSignatureNum],
  );

  const handleLaneClick = useCallback(
    (e: React.MouseEvent, trackId: string) => {
      if (e.target !== e.currentTarget) return;
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const beat = snapBeat(Math.max(0, (e.clientX - rect.left) / beatW));
      addClip(trackId, beat);
      selectTrack(trackId);
      setContextMenu(null);
    },
    [addClip, beatW, snapBeat, selectTrack],
  );

  const handleClipMouseDown = useCallback(
    (e: React.MouseEvent, clip: Clip) => {
      e.stopPropagation();
      if (e.button === 2) return;
      selectClip(clip.id);
      selectTrack(clip.trackId);
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      if (e.clientX > rect.right - 10) {
        resizing.current = {
          clipId: clip.id,
          startX: e.clientX,
          startLen: clip.lengthBeats,
        };
        const onMove = (ev: MouseEvent) => {
          if (!resizing.current) return;
          const newLen = snapBeat(
            Math.max(
              0.25,
              resizing.current.startLen +
                (ev.clientX - resizing.current.startX) / beatW,
            ),
          );
          resizeClip(resizing.current.clipId, newLen);
        };
        const onUp = () => {
          resizing.current = null;
          window.removeEventListener("mousemove", onMove);
          window.removeEventListener("mouseup", onUp);
        };
        window.addEventListener("mousemove", onMove);
        window.addEventListener("mouseup", onUp);
      } else {
        dragging.current = {
          clipId: clip.id,
          startX: e.clientX,
          startBeat: clip.startBeat,
        };
        const onMove = (ev: MouseEvent) => {
          if (!dragging.current) return;
          const newBeat = snapBeat(
            Math.max(
              0,
              dragging.current.startBeat +
                (ev.clientX - dragging.current.startX) / beatW,
            ),
          );
          moveClip(dragging.current.clipId, newBeat);
        };
        const onUp = () => {
          dragging.current = null;
          window.removeEventListener("mousemove", onMove);
          window.removeEventListener("mouseup", onUp);
        };
        window.addEventListener("mousemove", onMove);
        window.addEventListener("mouseup", onUp);
      }
    },
    [beatW, moveClip, resizeClip, selectClip, selectTrack, snapBeat],
  );

  const handleTrackResizeStart = useCallback(
    (e: React.MouseEvent, trackId: string, currentH: number) => {
      e.stopPropagation();
      trackResizing.current = { trackId, startY: e.clientY, startH: currentH };
      const onMove = (ev: MouseEvent) => {
        if (!trackResizing.current) return;
        const newH = Math.max(
          48,
          Math.min(
            200,
            trackResizing.current.startH +
              ev.clientY -
              trackResizing.current.startY,
          ),
        );
        updateTrack(trackResizing.current.trackId, { height: newH });
      };
      const onUp = () => {
        trackResizing.current = null;
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
      };
      window.addEventListener("mousemove", onMove);
      window.addEventListener("mouseup", onUp);
    },
    [updateTrack],
  );

  const handleRulerClick = useCallback(
    (e: React.MouseEvent) => {
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const beat = Math.max(0, (e.clientX - rect.left) / beatW);
      setTransport({ playheadBeat: beat });
    },
    [beatW, setTransport],
  );

  const border = "#0f1e2e";
  const dim = "#475569";
  const text = "#e0e8f0";
  const accent = "#4db8ff";

  const selectedTrack = tracks.find((t) => t.id === selectedTrackId);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: "#060c12",
        overflow: "hidden",
        position: "relative",
      }}
      onClick={() => setContextMenu(null)}
    >
      {/* Toolbar */}
      <div
        style={{
          height: 36,
          background: "#080e18",
          borderBottom: `1px solid ${border}`,
          display: "flex",
          alignItems: "center",
          padding: "0 10px",
          gap: 6,
          flexShrink: 0,
        }}
      >
        {[
          ["instrument", "+ Instrument", accent],
          ["audio", "+ Audio", "#34d399"],
          ["bus", "+ Bus", "#a78bfa"],
        ].map(([type, label, color]) => (
          <button
            key={type}
            onClick={() => addTrack(type as "instrument" | "audio" | "bus")}
            style={{
              padding: "3px 10px",
              background: `${color}12`,
              border: `1px solid ${color}30`,
              borderRadius: 4,
              color,
              fontSize: 11,
              cursor: "pointer",
              fontFamily: "var(--font-sans)",
            }}
          >
            {label}
          </button>
        ))}
        <div
          style={{ width: 1, height: 16, background: border, margin: "0 4px" }}
        />
        <span style={{ fontSize: 10, color: dim }}>Snap:</span>
        <select
          value={snap}
          onChange={(e) => setSnap(e.target.value as typeof snap)}
          style={{
            background: "#0a1520",
            border: `1px solid ${border}`,
            borderRadius: 4,
            color: text,
            padding: "2px 6px",
            fontSize: 10,
            fontFamily: "var(--font-sans)",
            cursor: "pointer",
          }}
        >
          {["bar", "beat", "half", "quarter", "eighth", "off"].map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </select>
        <span style={{ fontSize: 10, color: dim }}>Zoom:</span>
        <input
          type="range"
          min={0.25}
          max={4}
          step={0.25}
          value={zoom}
          onChange={(e) => setZoom(parseFloat(e.target.value))}
          style={{ width: 70 }}
        />
        <span
          style={{
            fontSize: 10,
            color: dim,
            fontFamily: "monospace",
            minWidth: 28,
          }}
        >
          {zoom}×
        </span>
        <div style={{ flex: 1 }} />
        <button
          onClick={() => setTransport({ loopEnabled: !transport.loopEnabled })}
          style={{
            padding: "3px 10px",
            background: transport.loopEnabled
              ? "rgba(77,184,255,0.15)"
              : "transparent",
            border: `1px solid ${transport.loopEnabled ? accent : border}`,
            borderRadius: 4,
            color: transport.loopEnabled ? accent : dim,
            fontSize: 11,
            cursor: "pointer",
            fontFamily: "var(--font-sans)",
          }}
        >
          ⟳ Loop
        </button>
      </div>

      {/* Main area */}
      <div style={{ flex: 1, display: "flex", overflow: "hidden" }}>
        {/* Track headers */}
        <div
          style={{
            width: HEADER_W,
            flexShrink: 0,
            background: "#080e18",
            borderRight: `1px solid ${border}`,
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              height: RULER_H,
              borderBottom: `1px solid ${border}`,
              flexShrink: 0,
            }}
          />
          <div style={{ flex: 1, overflowY: "auto" }}>
            {tracks.map((track) => (
              <div key={track.id} style={{ position: "relative" }}>
                <div
                  onClick={() => selectTrack(track.id)}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    setContextMenu({
                      x: e.clientX,
                      y: e.clientY,
                      type: "track",
                      id: track.id,
                    });
                  }}
                  style={{
                    height: track.height,
                    borderBottom: `1px solid ${border}`,
                    padding: "5px 8px",
                    display: "flex",
                    flexDirection: "column",
                    justifyContent: "space-between",
                    background:
                      selectedTrackId === track.id
                        ? "rgba(77,184,255,0.04)"
                        : "transparent",
                    cursor: "pointer",
                    borderLeft: `3px solid ${selectedTrackId === track.id ? track.color : "transparent"}`,
                    transition: "all 0.1s",
                  }}
                >
                  {/* Row 1: icon + name + arm + close */}
                  <div
                    style={{ display: "flex", alignItems: "center", gap: 5 }}
                  >
                    <span style={{ fontSize: 12 }}>
                      {TRACK_ICONS[track.type] ?? "🎵"}
                    </span>
                    {renamingTrackId === track.id ? (
                      <input
                        autoFocus
                        value={renameValue}
                        onChange={(e) => setRenameValue(e.target.value)}
                        onBlur={() => {
                          updateTrack(track.id, { name: renameValue });
                          setRenamingTrackId(null);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            updateTrack(track.id, { name: renameValue });
                            setRenamingTrackId(null);
                          }
                        }}
                        onClick={(e) => e.stopPropagation()}
                        style={{
                          flex: 1,
                          background: "#0a1520",
                          border: `1px solid ${accent}`,
                          borderRadius: 3,
                          color: text,
                          fontSize: 11,
                          padding: "1px 4px",
                          outline: "none",
                        }}
                      />
                    ) : (
                      <span
                        onDoubleClick={(e) => {
                          e.stopPropagation();
                          setRenamingTrackId(track.id);
                          setRenameValue(track.name);
                        }}
                        style={{
                          flex: 1,
                          fontSize: 11,
                          fontWeight: 600,
                          color:
                            selectedTrackId === track.id ? text : "#94a3b8",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {track.name}
                      </span>
                    )}
                    {/* ARM */}
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        updateTrack(track.id, { armed: !track.armed });
                      }}
                      title="Record arm"
                      style={{
                        width: 14,
                        height: 14,
                        borderRadius: "50%",
                        background: track.armed ? "#ef5350" : "#1a2a3a",
                        border: `1px solid ${track.armed ? "#ef5350" : border}`,
                        cursor: "pointer",
                        flexShrink: 0,
                        boxShadow: track.armed ? "0 0 6px #ef5350" : "none",
                      }}
                    />
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        removeTrack(track.id);
                      }}
                      style={{
                        background: "none",
                        border: "none",
                        color: "#1e2d3a",
                        cursor: "pointer",
                        fontSize: 10,
                        padding: 0,
                        lineHeight: 1,
                        flexShrink: 0,
                      }}
                    >
                      ✕
                    </button>
                  </div>

                  {/* Row 2: M S vol pan */}
                  <div
                    style={{ display: "flex", alignItems: "center", gap: 4 }}
                  >
                    {[
                      [
                        "M",
                        track.muted,
                        "#ef5350",
                        () => updateTrack(track.id, { muted: !track.muted }),
                      ],
                      [
                        "S",
                        track.solo,
                        "#ffd54f",
                        () => updateTrack(track.id, { solo: !track.solo }),
                      ],
                    ].map(([label, active, color, fn]) => (
                      <button
                        key={label as string}
                        onClick={(e) => {
                          e.stopPropagation();
                          (fn as () => void)();
                        }}
                        style={{
                          padding: "1px 5px",
                          background: active
                            ? `${color as string}20`
                            : "transparent",
                          border: `1px solid ${active ? `${color as string}50` : border}`,
                          borderRadius: 3,
                          color: active ? (color as string) : dim,
                          fontSize: 9,
                          cursor: "pointer",
                          fontFamily: "var(--font-sans)",
                        }}
                      >
                        {label as string}
                      </button>
                    ))}
                    <input
                      type="range"
                      min={0}
                      max={1}
                      step={0.01}
                      value={track.volume}
                      onClick={(e) => e.stopPropagation()}
                      onChange={(e) =>
                        updateTrack(track.id, {
                          volume: parseFloat(e.target.value),
                        })
                      }
                      style={{ flex: 1, height: 10 }}
                      title={`Vol: ${Math.round(track.volume * 100)}%`}
                    />
                    <span
                      style={{
                        fontSize: 8,
                        color: dim,
                        fontFamily: "monospace",
                        minWidth: 20,
                      }}
                    >
                      {track.pan === 0
                        ? "C"
                        : track.pan > 0
                          ? `R${Math.round(track.pan * 100)}`
                          : `L${Math.round(-track.pan * 100)}`}
                    </span>
                  </div>

                  {/* Row 3: effects chain */}
                  {track.effects.length > 0 && (
                    <div style={{ display: "flex", gap: 3, flexWrap: "wrap" }}>
                      {track.effects.map((fx) => (
                        <span
                          key={fx.id}
                          onClick={(e) => {
                            e.stopPropagation();
                            toggleEffect(track.id, fx.id);
                          }}
                          style={{
                            fontSize: 8,
                            padding: "1px 5px",
                            borderRadius: 3,
                            background: fx.enabled
                              ? `${EFFECT_COLORS[fx.type] ?? "#4db8ff"}20`
                              : "#0a1520",
                            border: `1px solid ${fx.enabled ? `${EFFECT_COLORS[fx.type] ?? "#4db8ff"}50` : border}`,
                            color: fx.enabled
                              ? (EFFECT_COLORS[fx.type] ?? "#4db8ff")
                              : dim,
                            cursor: "pointer",
                            fontFamily: "var(--font-sans)",
                          }}
                        >
                          {fx.type}
                        </span>
                      ))}
                    </div>
                  )}
                </div>

                {/* Track resize handle */}
                <div
                  onMouseDown={(e) =>
                    handleTrackResizeStart(e, track.id, track.height)
                  }
                  style={{
                    position: "absolute",
                    bottom: 0,
                    left: 0,
                    right: 0,
                    height: 4,
                    cursor: "row-resize",
                    zIndex: 2,
                  }}
                  onMouseEnter={(e) =>
                    (e.currentTarget.style.background = "rgba(77,184,255,0.3)")
                  }
                  onMouseLeave={(e) =>
                    (e.currentTarget.style.background = "transparent")
                  }
                />
              </div>
            ))}
          </div>
        </div>

        {/* Timeline */}
        <div
          ref={scrollRef}
          style={{ flex: 1, overflow: "auto", position: "relative" }}
        >
          <div
            style={{
              width: TOTAL_BEATS * beatW,
              minHeight: "100%",
              position: "relative",
            }}
          >
            {/* Ruler */}
            <div
              onClick={handleRulerClick}
              style={{
                height: RULER_H,
                background: "#080e18",
                borderBottom: `1px solid ${border}`,
                position: "sticky",
                top: 0,
                zIndex: 3,
                display: "flex",
                alignItems: "flex-end",
                cursor: "pointer",
              }}
            >
              {Array.from({ length: TOTAL_BEATS }, (_, i) => {
                const isBar = i % transport.timeSignatureNum === 0;
                const barNum = Math.floor(i / transport.timeSignatureNum) + 1;
                return (
                  <div
                    key={i}
                    style={{
                      width: beatW,
                      flexShrink: 0,
                      height: isBar ? "100%" : "45%",
                      borderLeft: `1px solid ${isBar ? "#1e3a5f" : "#0a1520"}`,
                      display: "flex",
                      alignItems: "flex-start",
                      paddingTop: 4,
                      paddingLeft: 3,
                    }}
                  >
                    {isBar && (
                      <span
                        style={{
                          fontSize: 9,
                          color: "#4db8ff",
                          fontFamily: "monospace",
                          fontWeight: 700,
                        }}
                      >
                        {barNum}
                      </span>
                    )}
                  </div>
                );
              })}

              {/* Loop region */}
              {transport.loopEnabled && (
                <div
                  style={{
                    position: "absolute",
                    left: transport.loopStart * beatW,
                    width: (transport.loopEnd - transport.loopStart) * beatW,
                    top: 0,
                    height: "100%",
                    background: "rgba(255,213,79,0.08)",
                    borderLeft: "2px solid #ffd54f",
                    borderRight: "2px solid #ffd54f",
                    pointerEvents: "none",
                  }}
                />
              )}

              {/* Playhead triangle */}
              <div
                style={{
                  position: "absolute",
                  left: transport.playheadBeat * beatW - 0.5,
                  top: 0,
                  bottom: 0,
                  width: 1,
                  background: "#4db8ff",
                  pointerEvents: "none",
                  zIndex: 4,
                }}
              >
                <div
                  style={{
                    width: 10,
                    height: 10,
                    background: "#4db8ff",
                    clipPath: "polygon(0 0, 100% 0, 50% 100%)",
                    marginLeft: -4.5,
                  }}
                />
              </div>
            </div>

            {/* Track lanes */}
            {tracks.map((track) => (
              <div
                key={track.id}
                style={{
                  height: track.height,
                  borderBottom: `1px solid ${border}`,
                  position: "relative",
                  background:
                    selectedTrackId === track.id
                      ? "rgba(77,184,255,0.015)"
                      : "transparent",
                  cursor: "crosshair",
                }}
                onClick={(e) => handleLaneClick(e, track.id)}
                onContextMenu={(e) => {
                  e.preventDefault();
                  setContextMenu({
                    x: e.clientX,
                    y: e.clientY,
                    type: "track",
                    id: track.id,
                  });
                }}
              >
                {/* Beat grid */}
                {Array.from({ length: TOTAL_BEATS }, (_, i) => {
                  const isBar = i % transport.timeSignatureNum === 0;
                  return (
                    <div
                      key={i}
                      style={{
                        position: "absolute",
                        left: i * beatW,
                        top: 0,
                        bottom: 0,
                        width: 1,
                        background: isBar ? "#0f1e2e" : "#080e18",
                        pointerEvents: "none",
                      }}
                    />
                  );
                })}

                {/* Playhead line through lane */}
                <div
                  style={{
                    position: "absolute",
                    left: transport.playheadBeat * beatW,
                    top: 0,
                    bottom: 0,
                    width: 1,
                    background: "rgba(77,184,255,0.4)",
                    pointerEvents: "none",
                    zIndex: 2,
                  }}
                />

                {/* Clips */}
                {track.clips.map((clip) => (
                  <ClipBlock
                    key={clip.id}
                    clip={clip}
                    beatW={beatW}
                    trackH={track.height}
                    selected={selectedClipId === clip.id}
                    onMouseDown={(e) => handleClipMouseDown(e, clip)}
                    onDoubleClick={(e) => {
                      e.stopPropagation();
                      openPianoRoll(clip.id, clip.trackId);
                    }}
                    onContextMenu={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      setContextMenu({
                        x: e.clientX,
                        y: e.clientY,
                        type: "clip",
                        id: clip.id,
                      });
                    }}
                    onDelete={() => removeClip(clip.id)}
                  />
                ))}
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Effects panel for selected track */}
      {selectedTrack && (
        <div
          style={{
            height: 40,
            background: "#080e18",
            borderTop: `1px solid ${border}`,
            display: "flex",
            alignItems: "center",
            padding: "0 12px",
            gap: 8,
            flexShrink: 0,
          }}
        >
          <span
            style={{
              fontSize: 10,
              color: dim,
              fontWeight: 600,
              letterSpacing: "0.08em",
            }}
          >
            FX CHAIN:
          </span>
          {["EQ", "Compressor", "Reverb", "Delay", "Filter"].map((fx) => (
            <button
              key={fx}
              onClick={() => addEffect(selectedTrack.id, fx)}
              style={{
                padding: "2px 8px",
                background: `${EFFECT_COLORS[fx]}12`,
                border: `1px solid ${EFFECT_COLORS[fx]}30`,
                borderRadius: 4,
                color: EFFECT_COLORS[fx],
                fontSize: 10,
                cursor: "pointer",
                fontFamily: "var(--font-sans)",
              }}
            >
              + {fx}
            </button>
          ))}
          {selectedTrack.effects.map((fx) => (
            <div
              key={fx.id}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 3,
                padding: "2px 6px",
                background: `${EFFECT_COLORS[fx.type] ?? "#4db8ff"}15`,
                border: `1px solid ${EFFECT_COLORS[fx.type] ?? "#4db8ff"}40`,
                borderRadius: 4,
              }}
            >
              <span
                style={{
                  fontSize: 10,
                  color: EFFECT_COLORS[fx.type] ?? "#4db8ff",
                }}
              >
                {fx.type}
              </span>
              <button
                onClick={() => removeEffect(selectedTrack.id, fx.id)}
                style={{
                  background: "none",
                  border: "none",
                  color: dim,
                  cursor: "pointer",
                  fontSize: 9,
                  padding: 0,
                  lineHeight: 1,
                }}
              >
                ✕
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Context menu */}
      {contextMenu && (
        <div
          style={{
            position: "fixed",
            left: contextMenu.x,
            top: contextMenu.y,
            background: "#0c1420",
            border: `1px solid ${border}`,
            borderRadius: 6,
            zIndex: 9999,
            minWidth: 160,
            boxShadow: "0 8px 32px rgba(0,0,0,0.6)",
            overflow: "hidden",
          }}
        >
          {contextMenu.type === "track" ? (
            <>
              {["EQ", "Compressor", "Reverb", "Delay", "Filter"].map((fx) => (
                <button
                  key={fx}
                  onClick={() => {
                    addEffect(contextMenu.id, fx);
                    setContextMenu(null);
                  }}
                  style={{
                    display: "block",
                    width: "100%",
                    padding: "8px 14px",
                    background: "transparent",
                    border: "none",
                    color: EFFECT_COLORS[fx],
                    fontSize: 11,
                    cursor: "pointer",
                    textAlign: "left",
                    fontFamily: "var(--font-sans)",
                  }}
                >
                  + Add {fx}
                </button>
              ))}
              <div style={{ height: 1, background: border, margin: "4px 0" }} />
              <button
                onClick={() => {
                  const t = tracks.find((t) => t.id === contextMenu.id);
                  if (t) {
                    setRenamingTrackId(t.id);
                    setRenameValue(t.name);
                  }
                  setContextMenu(null);
                }}
                style={{
                  display: "block",
                  width: "100%",
                  padding: "8px 14px",
                  background: "transparent",
                  border: "none",
                  color: text,
                  fontSize: 11,
                  cursor: "pointer",
                  textAlign: "left",
                  fontFamily: "var(--font-sans)",
                }}
              >
                Rename
              </button>
              <button
                onClick={() => {
                  removeTrack(contextMenu.id);
                  setContextMenu(null);
                }}
                style={{
                  display: "block",
                  width: "100%",
                  padding: "8px 14px",
                  background: "transparent",
                  border: "none",
                  color: "#ef5350",
                  fontSize: 11,
                  cursor: "pointer",
                  textAlign: "left",
                  fontFamily: "var(--font-sans)",
                }}
              >
                Delete Track
              </button>
            </>
          ) : (
            <>
              <button
                onClick={() => {
                  const clip = tracks
                    .flatMap((t) => t.clips)
                    .find((c) => c.id === contextMenu.id);
                  if (clip) openPianoRoll(clip.id, clip.trackId);
                  setContextMenu(null);
                }}
                style={{
                  display: "block",
                  width: "100%",
                  padding: "8px 14px",
                  background: "transparent",
                  border: "none",
                  color: text,
                  fontSize: 11,
                  cursor: "pointer",
                  textAlign: "left",
                  fontFamily: "var(--font-sans)",
                }}
              >
                Open in Piano Roll
              </button>
              <button
                onClick={() => {
                  removeClip(contextMenu.id);
                  setContextMenu(null);
                }}
                style={{
                  display: "block",
                  width: "100%",
                  padding: "8px 14px",
                  background: "transparent",
                  border: "none",
                  color: "#ef5350",
                  fontSize: 11,
                  cursor: "pointer",
                  textAlign: "left",
                  fontFamily: "var(--font-sans)",
                }}
              >
                Delete Clip
              </button>
            </>
          )}
        </div>
      )}
    </div>
  );
}

function ClipBlock({
  clip,
  beatW,
  trackH,
  selected,
  onMouseDown,
  onDoubleClick,
  onContextMenu,
  onDelete,
}: {
  clip: Clip;
  beatW: number;
  trackH: number;
  selected: boolean;
  onMouseDown: (e: React.MouseEvent) => void;
  onDoubleClick: (e: React.MouseEvent) => void;
  onContextMenu: (e: React.MouseEvent) => void;
  onDelete: () => void;
}) {
  void onDelete;
  const noteCount = clip.notes.length;
  return (
    <div
      onMouseDown={onMouseDown}
      onDoubleClick={onDoubleClick}
      onContextMenu={onContextMenu}
      style={{
        position: "absolute",
        left: clip.startBeat * beatW + 1,
        width: Math.max(clip.lengthBeats * beatW - 2, 8),
        top: 4,
        height: trackH - 8,
        background: selected ? `${clip.color}28` : `${clip.color}14`,
        border: `1px solid ${selected ? clip.color : `${clip.color}50`}`,
        borderRadius: 3,
        cursor: "grab",
        overflow: "hidden",
        userSelect: "none",
        boxShadow: selected ? `0 0 10px ${clip.color}40` : "none",
      }}
    >
      <div
        style={{
          padding: "2px 5px",
          fontSize: 9,
          fontWeight: 700,
          color: clip.color,
          fontFamily: "var(--font-sans)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
        }}
      >
        {clip.name}
        {noteCount > 0 && (
          <span style={{ marginLeft: 4, opacity: 0.5 }}>{noteCount}n</span>
        )}
      </div>
      {noteCount > 0 && (
        <div
          style={{
            position: "absolute",
            inset: "14px 2px 2px",
            overflow: "hidden",
          }}
        >
          {clip.notes.map((note) => (
            <div
              key={note.id}
              style={{
                position: "absolute",
                left: `${(note.beat / clip.lengthBeats) * 100}%`,
                width: `${Math.max((note.duration / clip.lengthBeats) * 100, 1)}%`,
                bottom: `${((note.pitch - 36) / 60) * 100}%`,
                height: 2,
                background: clip.color,
                borderRadius: 1,
                opacity: 0.85,
              }}
            />
          ))}
        </div>
      )}
      <div
        style={{
          position: "absolute",
          right: 0,
          top: 0,
          bottom: 0,
          width: 8,
          cursor: "ew-resize",
        }}
      />
    </div>
  );
}
