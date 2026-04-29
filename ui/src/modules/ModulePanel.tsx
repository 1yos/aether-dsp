/**
 * Module System — ModulePanel
 * A resizable, draggable floating panel that wraps any studio module.
 */
import React, { useCallback, useRef } from "react";
import type { ModuleInstance } from "./types";
import { useModuleStore } from "./useModuleStore";

interface ModulePanelProps {
  module: ModuleInstance;
  children: React.ReactNode;
}

const TITLE_HEIGHT = 36;
const MIN_WIDTH = 200;
const MIN_HEIGHT = 150;

export function ModulePanel({ module, children }: ModulePanelProps) {
  const removeModule = useModuleStore((s) => s.removeModule);
  const moveModule = useModuleStore((s) => s.moveModule);
  const resizeModule = useModuleStore((s) => s.resizeModule);

  // ── Drag (title bar) ──────────────────────────────────────────────────────
  const dragState = useRef<{
    startX: number;
    startY: number;
    origX: number;
    origY: number;
  } | null>(null);

  const onTitleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      // Ignore clicks on the close button
      if ((e.target as HTMLElement).closest("[data-close]")) return;
      e.preventDefault();
      dragState.current = {
        startX: e.clientX,
        startY: e.clientY,
        origX: module.x,
        origY: module.y,
      };

      const onMouseMove = (ev: MouseEvent) => {
        if (!dragState.current) return;
        const dx = ev.clientX - dragState.current.startX;
        const dy = ev.clientY - dragState.current.startY;
        moveModule(
          module.id,
          dragState.current.origX + dx,
          dragState.current.origY + dy,
        );
      };

      const onMouseUp = () => {
        dragState.current = null;
        window.removeEventListener("mousemove", onMouseMove);
        window.removeEventListener("mouseup", onMouseUp);
      };

      window.addEventListener("mousemove", onMouseMove);
      window.addEventListener("mouseup", onMouseUp);
    },
    [module.id, module.x, module.y, moveModule],
  );

  // ── Resize (bottom-right handle) ─────────────────────────────────────────
  const resizeState = useRef<{
    startX: number;
    startY: number;
    origW: number;
    origH: number;
  } | null>(null);

  const onResizeMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      resizeState.current = {
        startX: e.clientX,
        startY: e.clientY,
        origW: module.width,
        origH: module.height,
      };

      const onMouseMove = (ev: MouseEvent) => {
        if (!resizeState.current) return;
        const dx = ev.clientX - resizeState.current.startX;
        const dy = ev.clientY - resizeState.current.startY;
        const newW = Math.max(MIN_WIDTH, resizeState.current.origW + dx);
        const newH = Math.max(MIN_HEIGHT, resizeState.current.origH + dy);
        resizeModule(module.id, newW, newH);
      };

      const onMouseUp = () => {
        resizeState.current = null;
        window.removeEventListener("mousemove", onMouseMove);
        window.removeEventListener("mouseup", onMouseUp);
      };

      window.addEventListener("mousemove", onMouseMove);
      window.addEventListener("mouseup", onMouseUp);
    },
    [module.id, module.width, module.height, resizeModule],
  );

  return (
    <div
      style={{
        position: "absolute",
        left: module.x,
        top: module.y,
        width: module.width,
        height: module.height,
        display: "flex",
        flexDirection: "column",
        background: "#0a1520",
        border: "1px solid #1a2a3a",
        borderRadius: 8,
        boxShadow: "0 8px 32px rgba(0,0,0,0.6)",
        overflow: "hidden",
        zIndex: 100,
        userSelect: "none",
      }}
    >
      {/* Title bar */}
      <div
        onMouseDown={onTitleMouseDown}
        style={{
          height: TITLE_HEIGHT,
          minHeight: TITLE_HEIGHT,
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "0 12px",
          background: "#0f1e2e",
          borderBottom: "1px solid #1a2a3a",
          cursor: "grab",
          flexShrink: 0,
        }}
      >
        <span
          style={{
            color: "#a0c8e8",
            fontSize: 13,
            fontWeight: 600,
            letterSpacing: "0.04em",
            fontFamily: "monospace",
          }}
        >
          {module.type}
        </span>
        <button
          data-close="true"
          onClick={() => removeModule(module.id)}
          title="Close module"
          style={{
            background: "none",
            border: "none",
            color: "#4a6a8a",
            cursor: "pointer",
            fontSize: 16,
            lineHeight: 1,
            padding: "2px 4px",
            borderRadius: 4,
            transition: "color 0.15s",
          }}
          onMouseEnter={(e) =>
            ((e.target as HTMLButtonElement).style.color = "#e05050")
          }
          onMouseLeave={(e) =>
            ((e.target as HTMLButtonElement).style.color = "#4a6a8a")
          }
        >
          ✕
        </button>
      </div>

      {/* Module content */}
      <div
        style={{
          flex: 1,
          overflow: "auto",
          color: "#e0e8f0",
          position: "relative",
        }}
      >
        {children}
      </div>

      {/* Resize handle — bottom-right corner */}
      <div
        onMouseDown={onResizeMouseDown}
        title="Resize"
        style={{
          position: "absolute",
          bottom: 0,
          right: 0,
          width: 16,
          height: 16,
          cursor: "nwse-resize",
          display: "flex",
          alignItems: "flex-end",
          justifyContent: "flex-end",
          padding: "2px",
          color: "#2a4a6a",
          fontSize: 10,
          lineHeight: 1,
          userSelect: "none",
        }}
      >
        ◢
      </div>
    </div>
  );
}
