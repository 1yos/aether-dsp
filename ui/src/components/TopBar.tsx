/**
 * Aether Studio v2.0 - Top Bar
 * Main navigation and transport controls
 */

import { useModeStore } from "../store/useModeStore";
import { STUDIO_MODES } from "../types/modes";
import "./TopBar.css";

export function TopBar() {
  const { currentMode, setMode } = useModeStore();

  return (
    <div className="top-bar">
      {/* Left: Logo */}
      <div className="top-bar-left">
        <div className="logo">
          <span className="logo-icon">◉</span>
          <span className="logo-text">Aether</span>
          <span className="logo-version">v2.0</span>
        </div>
      </div>

      {/* Center: Mode Switcher */}
      <div className="top-bar-center">
        <div className="mode-switcher">
          {Object.values(STUDIO_MODES).map((mode) => (
            <button
              key={mode.id}
              className={`mode-button ${currentMode === mode.id ? "active" : ""}`}
              onClick={() => setMode(mode.id)}
              title={`${mode.description} (${mode.shortcut})`}
            >
              <span className="mode-icon">{mode.icon}</span>
              <span className="mode-label">{mode.label}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Right: Transport Controls */}
      <div className="top-bar-right">
        <button className="transport-btn play-btn" title="Play/Pause (Space)">
          <span>▶</span>
        </button>
        <button className="transport-btn record-btn" title="Record (R)">
          <span>●</span>
        </button>
        <button className="transport-btn" title="Undo (Ctrl+Z)">
          <span>↩</span>
        </button>
        <button className="transport-btn" title="Redo (Ctrl+Y)">
          <span>↪</span>
        </button>
        <div className="vu-meter">
          <div className="vu-bar" style={{ width: "60%" }}></div>
        </div>
        <div className="live-indicator">
          <span className="live-dot"></span>
          <span>Live</span>
        </div>
      </div>
    </div>
  );
}
