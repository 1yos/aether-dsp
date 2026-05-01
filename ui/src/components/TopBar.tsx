/**
 * Aether Studio — Top Bar
 * Minimal, focused. The mode switcher is the center of gravity.
 */

import { useModeStore } from "../store/useModeStore";
import { STUDIO_MODES } from "../types/modes";
import { useProjectSave } from "../hooks/useProjectSave";
import { useEngineStore } from "../studio/store/engineStore";
import { ModulationMatrix } from "./ModulationMatrix";
import { useState } from "react";
import "./TopBar.css";

export function TopBar() {
  const { currentMode, setMode } = useModeStore();
  const { saveProject, loadProject } = useProjectSave();
  const wsStatus = useEngineStore((s) => s.wsStatus);
  const audioActive = useEngineStore((s) => s.audioActive);
  const [showModMatrix, setShowModMatrix] = useState(false);

  const statusColor =
    wsStatus === "connected"
      ? "var(--accent-success)"
      : wsStatus === "connecting"
        ? "var(--accent-warning)"
        : "var(--accent-error)";
  const statusLabel =
    wsStatus === "connected"
      ? "Live"
      : wsStatus === "connecting"
        ? "Connecting"
        : "Offline";

  return (
    <header className="top-bar">
      {/* Left: Logo */}
      <div className="top-bar-left">
        <div className="logo">
          <div className="logo-mark">◉</div>
          <div className="logo-text">
            <span className="logo-name">Aether</span>
            <span className="logo-version">Studio</span>
          </div>
        </div>
      </div>

      {/* Center: Mode switcher */}
      <nav className="mode-switcher" role="tablist">
        {Object.values(STUDIO_MODES).map((mode) => (
          <button
            key={mode.id}
            role="tab"
            aria-selected={currentMode === mode.id}
            className={`mode-btn ${currentMode === mode.id ? "active" : ""}`}
            onClick={() => setMode(mode.id)}
            title={`${mode.description} (${mode.shortcut})`}
          >
            <span className="mode-icon">{mode.icon}</span>
            <span className="mode-label">{mode.label}</span>
            {currentMode === mode.id && <span className="mode-indicator" />}
          </button>
        ))}
      </nav>

      {/* Right: Controls */}
      <div className="top-bar-right">
        <button
          className="tb-btn"
          title="Save (Ctrl+S)"
          onClick={() => saveProject()}
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path
              d="M2 2h8l2 2v8H2V2z"
              stroke="currentColor"
              strokeWidth="1.2"
              strokeLinejoin="round"
            />
            <rect
              x="4"
              y="8"
              width="6"
              height="4"
              rx="0.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <rect
              x="4"
              y="2"
              width="5"
              height="3"
              rx="0.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
        </button>
        <button className="tb-btn" title="Load (Ctrl+O)" onClick={loadProject}>
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path
              d="M2 4h4l1.5 1.5H12V12H2V4z"
              stroke="currentColor"
              strokeWidth="1.2"
              strokeLinejoin="round"
            />
          </svg>
        </button>
        <button
          className={`tb-btn ${showModMatrix ? "active" : ""}`}
          title="Modulation Matrix"
          onClick={() => setShowModMatrix((v) => !v)}
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <circle
              cx="3"
              cy="3"
              r="1.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <circle
              cx="11"
              cy="7"
              r="1.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <circle
              cx="3"
              cy="11"
              r="1.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <path
              d="M4.5 3.5L9.5 6.5M4.5 10.5L9.5 7.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
        </button>

        {showModMatrix && (
          <ModulationMatrix onClose={() => setShowModMatrix(false)} />
        )}

        <div className="tb-divider" />

        {/* VU meter */}
        <div className="vu-meter" title="Audio level">
          <div
            className="vu-bar"
            style={{
              width: audioActive ? "70%" : "0%",
              transition: audioActive ? "width 50ms" : "width 300ms",
            }}
          />
        </div>

        {/* Engine status */}
        <div
          className="engine-status"
          style={{ "--status-color": statusColor } as React.CSSProperties}
        >
          <span className="status-dot" />
          <span className="status-label">{statusLabel}</span>
        </div>
      </div>
    </header>
  );
}
