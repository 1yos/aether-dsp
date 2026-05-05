/**
 * SongView — the main timeline / playlist.
 *
 * Shows all tracks with their clips arranged on a timeline.
 * Click empty space to add a clip. Click a clip to select it.
 * Double-click a clip to open it in the Piano Roll.
 * Drag clips to move them. Drag the right edge to resize.
 */

import { useCallback, useRef, useState } from "react";
import { useDawStore } from "../store/dawStore";
import type { Track, Clip } from "../store/dawStore";

const TRACK_H = 56;
const BEAT_W = 32; // pixels per beat at zoom 1
const HEADER_W = 180;
const RULER_H = 28;
const TOTAL_BEATS = 128;

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

  const [zoom, setZoom] = useState(1);
  const beatW = BEAT_W * zoom;
  const scrollRef = useRef<HTMLDivElement>(null);

  // Drag state
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

  const beatFromX = useCallback(
    (x: number) => Math.max(0, Math.floor(x / beatW)),
    [beatW],
  );

  const handleLaneClick = useCallback(
    (e: React.MouseEvent, trackId: string) => {
      if (e.target !== e.currentTarget) return; // only bare lane clicks
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const beat = beatFromX(e.clientX - rect.left);
      addClip(trackId, beat);
      selectTrack(trackId);
    },
    [addClip, beatFromX, selectTrack],
  );

  const handleClipMouseDown = useCallback(
    (e: React.MouseEvent, clip: Clip) => {
      e.stopPropagation();
      selectClip(clip.id);
      selectTrack(clip.trackId);

      // Check if clicking resize handle (right 8px)
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      if (e.clientX > rect.right - 8) {
        resizing.current = {
          clipId: clip.id,
          startX: e.clientX,
          startLen: clip.lengthBeats,
        };
        const onMove = (ev: MouseEvent) => {
          if (!resizing.current) return;
          const dx = ev.clientX - resizing.current.startX;
          resizeClip(
            resizing.current.clipId,
            resizing.current.startLen + dx / beatW,
          );
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
          const dx = ev.clientX - dragging.current.startX;
          moveClip(
            dragging.current.clipId,
            dragging.current.startBeat + dx / beatW,
          );
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
    [beatW, moveClip, resizeClip, selectClip, selectTrack],
  );

  const handleClipDoubleClick = useCallback(
    (e: React.MouseEvent, clip: Clip) => {
      e.stopPropagation();
      openPianoRoll(clip.id, clip.trackId);
    },
    [openPianoRoll],
  );

  const border = "#0f1e2e";
  const dim = "#475569";

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: "#060c12",
        overflow: "hidden",
      }}
    >
      {/* Toolbar */}
      <div
        style={{
          height: 36,
          background: "#080e18",
          borderBottom: `1px solid ${border}`,
          display: "flex",
          alignItems: "center",
          padding: "0 12px",
          gap: 8,
          flexShrink: 0,
        }}
      >
        <button
          onClick={() => addTrack("instrument")}
          style={{
            padding: "3px 10px",
            background: "rgba(77,184,255,0.08)",
            border: "1px solid rgba(77,184,255,0.2)",
            borderRadius: 4,
            color: "#4db8ff",
            fontSize: 11,
            cursor: "pointer",
            fontFamily: "var(--font-sans)",
          }}
        >
          + Track
        </button>
        <button
          onClick={() => addTrack("audio", "Audio")}
          style={{
            padding: "3px 10px",
            background: "transparent",
            border: `1px solid ${border}`,
            borderRadius: 4,
            color: dim,
            fontSize: 11,
            cursor: "pointer",
            fontFamily: "var(--font-sans)",
          }}
        >
          + Audio
        </button>

        <div style={{ flex: 1 }} />

        {/* Zoom */}
        <span style={{ fontSize: 10, color: dim }}>Zoom</span>
        <input
          type="range"
          min={0.25}
          max={4}
          step={0.25}
          value={zoom}
          onChange={(e) => setZoom(parseFloat(e.target.value))}
          style={{ width: 80 }}
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
            overflow: "hidden",
          }}
        >
          {/* Ruler spacer */}
          <div
            style={{ height: RULER_H, borderBottom: `1px solid ${border}` }}
          />

          {tracks.map((track) => (
            <TrackHeader
              key={track.id}
              track={track}
              selected={selectedTrackId === track.id}
              onSelect={() => selectTrack(track.id)}
              onRemove={() => removeTrack(track.id)}
              onUpdate={(patch) => updateTrack(track.id, patch)}
            />
          ))}
        </div>

        {/* Timeline */}
        <div
          ref={scrollRef}
          style={{ flex: 1, overflow: "auto", position: "relative" }}
        >
          <div style={{ width: TOTAL_BEATS * beatW, minHeight: "100%" }}>
            {/* Ruler */}
            <div
              style={{
                height: RULER_H,
                background: "#080e18",
                borderBottom: `1px solid ${border}`,
                position: "sticky",
                top: 0,
                zIndex: 2,
                display: "flex",
                alignItems: "flex-end",
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
                      height: isBar ? "100%" : "40%",
                      borderLeft: `1px solid ${isBar ? "#1a2a3a" : "#0a1520"}`,
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
                          color: "#334155",
                          fontFamily: "monospace",
                        }}
                      >
                        {barNum}
                      </span>
                    )}
                  </div>
                );
              })}

              {/* Playhead */}
              <div
                style={{
                  position: "absolute",
                  left: transport.playheadBeat * beatW,
                  top: 0,
                  bottom: 0,
                  width: 1,
                  background: "#4db8ff",
                  pointerEvents: "none",
                  zIndex: 3,
                }}
              >
                <div
                  style={{
                    width: 8,
                    height: 8,
                    background: "#4db8ff",
                    clipPath: "polygon(0 0, 100% 0, 50% 100%)",
                    marginLeft: -3.5,
                  }}
                />
              </div>
            </div>

            {/* Track lanes */}
            {tracks.map((track) => (
              <div
                key={track.id}
                style={{
                  height: TRACK_H,
                  borderBottom: `1px solid ${border}`,
                  position: "relative",
                  background:
                    selectedTrackId === track.id
                      ? "rgba(77,184,255,0.02)"
                      : "transparent",
                  cursor: "crosshair",
                }}
                onClick={(e) => handleLaneClick(e, track.id)}
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

                {/* Clips */}
                {track.clips.map((clip) => (
                  <ClipBlock
                    key={clip.id}
                    clip={clip}
                    beatW={beatW}
                    selected={selectedClipId === clip.id}
                    onMouseDown={(e) => handleClipMouseDown(e, clip)}
                    onDoubleClick={(e) => handleClipDoubleClick(e, clip)}
                    onDelete={() => removeClip(clip.id)}
                  />
                ))}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Track header ──────────────────────────────────────────────────────────────

function TrackHeader({
  track,
  selected,
  onSelect,
  onRemove,
  onUpdate,
}: {
  track: Track;
  selected: boolean;
  onSelect: () => void;
  onRemove: () => void;
  onUpdate: (patch: Partial<Track>) => void;
}) {
  const border = "#0f1e2e";
  const dim = "#475569";

  return (
    <div
      onClick={onSelect}
      style={{
        height: TRACK_H,
        borderBottom: `1px solid ${border}`,
        padding: "6px 10px",
        display: "flex",
        flexDirection: "column",
        justifyContent: "space-between",
        background: selected ? "rgba(77,184,255,0.04)" : "transparent",
        cursor: "pointer",
        borderLeft: `3px solid ${selected ? track.color : "transparent"}`,
        transition: "all 0.1s",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
        <div
          style={{
            width: 8,
            height: 8,
            borderRadius: 2,
            background: track.color,
            flexShrink: 0,
          }}
        />
        <span
          style={{
            fontSize: 11,
            fontWeight: 600,
            color: selected ? "#e0e8f0" : "#94a3b8",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            flex: 1,
          }}
        >
          {track.name}
        </span>
        <button
          onClick={(e) => {
            e.stopPropagation();
            onRemove();
          }}
          style={{
            background: "none",
            border: "none",
            color: "#1e2d3a",
            cursor: "pointer",
            fontSize: 10,
            padding: 0,
            lineHeight: 1,
          }}
        >
          ✕
        </button>
      </div>

      <div style={{ display: "flex", gap: 4 }}>
        <button
          onClick={(e) => {
            e.stopPropagation();
            onUpdate({ muted: !track.muted });
          }}
          style={{
            padding: "1px 5px",
            background: track.muted ? "rgba(239,83,80,0.15)" : "transparent",
            border: `1px solid ${track.muted ? "rgba(239,83,80,0.3)" : border}`,
            borderRadius: 3,
            color: track.muted ? "#ef5350" : dim,
            fontSize: 9,
            cursor: "pointer",
            fontFamily: "var(--font-sans)",
          }}
        >
          M
        </button>
        <button
          onClick={(e) => {
            e.stopPropagation();
            onUpdate({ solo: !track.solo });
          }}
          style={{
            padding: "1px 5px",
            background: track.solo ? "rgba(255,213,79,0.15)" : "transparent",
            border: `1px solid ${track.solo ? "rgba(255,213,79,0.3)" : border}`,
            borderRadius: 3,
            color: track.solo ? "#ffd54f" : dim,
            fontSize: 9,
            cursor: "pointer",
            fontFamily: "var(--font-sans)",
          }}
        >
          S
        </button>
        <div style={{ flex: 1 }}>
          <input
            type="range"
            min={0}
            max={1}
            step={0.01}
            value={track.volume}
            onClick={(e) => e.stopPropagation()}
            onChange={(e) => onUpdate({ volume: parseFloat(e.target.value) })}
            style={{ width: "100%", height: 12 }}
          />
        </div>
      </div>
    </div>
  );
}

// ── Clip block ────────────────────────────────────────────────────────────────

function ClipBlock({
  clip,
  beatW,
  selected,
  onMouseDown,
  onDoubleClick,
  onDelete,
}: {
  clip: Clip;
  beatW: number;
  selected: boolean;
  onMouseDown: (e: React.MouseEvent) => void;
  onDoubleClick: (e: React.MouseEvent) => void;
  onDelete: () => void;
}) {
  const noteCount = clip.notes.length;

  return (
    <div
      onMouseDown={onMouseDown}
      onDoubleClick={onDoubleClick}
      style={{
        position: "absolute",
        left: clip.startBeat * beatW + 1,
        width: Math.max(clip.lengthBeats * beatW - 2, 8),
        top: 4,
        height: TRACK_H - 8,
        background: selected ? `${clip.color}30` : `${clip.color}18`,
        border: `1px solid ${selected ? clip.color : `${clip.color}60`}`,
        borderRadius: 4,
        cursor: "grab",
        overflow: "hidden",
        userSelect: "none",
        boxShadow: selected ? `0 0 8px ${clip.color}40` : "none",
        transition: "box-shadow 0.1s",
      }}
    >
      {/* Clip name */}
      <div
        style={{
          padding: "2px 5px",
          fontSize: 9,
          fontWeight: 600,
          color: clip.color,
          fontFamily: "var(--font-sans)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
        }}
      >
        {clip.name}
        {noteCount > 0 && (
          <span style={{ marginLeft: 4, opacity: 0.6 }}>{noteCount}n</span>
        )}
      </div>

      {/* Mini note preview */}
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
                width: `${(note.duration / clip.lengthBeats) * 100}%`,
                bottom: `${((note.pitch - 36) / 60) * 100}%`,
                height: 2,
                background: clip.color,
                borderRadius: 1,
                opacity: 0.8,
              }}
            />
          ))}
        </div>
      )}

      {/* Resize handle */}
      <div
        style={{
          position: "absolute",
          right: 0,
          top: 0,
          bottom: 0,
          width: 8,
          cursor: "ew-resize",
          background: "transparent",
        }}
      />

      {/* Delete on right-click */}
      <div
        onContextMenu={(e) => {
          e.preventDefault();
          e.stopPropagation();
          onDelete();
        }}
        style={{ position: "absolute", inset: 0 }}
      />
    </div>
  );
}
