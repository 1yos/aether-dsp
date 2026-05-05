/**
 * Aether Studio — Top Bar
 * Minimal, focused. The mode switcher is the center of gravity.
 */

import { useModeStore } from "../store/useModeStore";
import { STUDIO_MODES } from "../types/modes";
import { useProjectSave } from "../hooks/useProjectSave";
import { useEngineStore } from "../studio/store/engineStore";
import { ModulationMatrix } from "./ModulationMatrix";
import { SampleLibrary } from "./SampleLibrary";
import { SettingsPanel } from "./SettingsPanel";
import { PatchBrowser } from "./PatchBrowser";
import { usePatchShare } from "../hooks/usePatchShare";
import { useState, useEffect, useCallback } from "react";
import "./TopBar.css";

export function TopBar({
  onOpenInstrumentMaker,
}: {
  onOpenInstrumentMaker?: () => void;
}) {
  const { currentMode, setMode } = useModeStore();
  const { saveProject, loadProject } = useProjectSave();
  const wsStatus = useEngineStore((s) => s.wsStatus);
  const audioActive = useEngineStore((s) => s.audioActive);
  const [showModMatrix, setShowModMatrix] = useState(false);
  const [showSampleLibrary, setShowSampleLibrary] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showPatchBrowser, setShowPatchBrowser] = useState(false);
  const [showShareToast, setShowShareToast] = useState(false);
  const [shareUrl, setShareUrl] = useState<string | null>(null);

  const { sharePatch, shareState, checkUrlForSharedPatch } = usePatchShare();

  // Auto-load shared patch from URL on mount
  useEffect(() => {
    checkUrlForSharedPatch();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const handleShare = useCallback(async () => {
    const url = await sharePatch();
    if (url) {
      setShareUrl(url);
      setShowShareToast(true);
      // Copy to clipboard
      try {
        await navigator.clipboard.writeText(url);
      } catch {
        /* ignore */
      }
      setTimeout(() => setShowShareToast(false), 5000);
    }
  }, [sharePatch]);

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

        {/* Sample Library panel */}
        <button
          className={`tb-btn ${showSampleLibrary ? "active" : ""}`}
          title="Sample Library — download instrument packs"
          onClick={() => setShowSampleLibrary((v) => !v)}
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect
              x="1"
              y="2"
              width="12"
              height="2.5"
              rx="0.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <rect
              x="1"
              y="5.75"
              width="12"
              height="2.5"
              rx="0.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <rect
              x="1"
              y="9.5"
              width="12"
              height="2.5"
              rx="0.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <circle cx="11" cy="3.25" r="1" fill="currentColor" />
          </svg>
        </button>

        {showSampleLibrary && (
          <div
            style={{
              position: "fixed",
              top: 48,
              right: 0,
              width: 560,
              height: "calc(100vh - 48px)",
              zIndex: 1000,
              boxShadow: "-8px 0 32px rgba(0,0,0,0.6)",
              borderLeft: "1px solid #0f1e2e",
            }}
          >
            <SampleLibrary onClose={() => setShowSampleLibrary(false)} />
          </div>
        )}

        {/* Instrument Maker */}
        {onOpenInstrumentMaker && (
          <button
            className="tb-btn"
            title="Instrument Maker — record your own instrument samples"
            onClick={onOpenInstrumentMaker}
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <circle
                cx="7"
                cy="4"
                r="2.5"
                stroke="currentColor"
                strokeWidth="1.2"
              />
              <path
                d="M7 6.5V11"
                stroke="currentColor"
                strokeWidth="1.2"
                strokeLinecap="round"
              />
              <path
                d="M4.5 11h5"
                stroke="currentColor"
                strokeWidth="1.2"
                strokeLinecap="round"
              />
              <circle cx="7" cy="4" r="1" fill="currentColor" opacity="0.4" />
            </svg>
          </button>
        )}

        {/* Community patches browser */}
        <button
          className={`tb-btn ${showPatchBrowser ? "active" : ""}`}
          title="Community Patches — browse patches shared by other users"
          onClick={() => setShowPatchBrowser((v) => !v)}
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect
              x="1"
              y="2"
              width="12"
              height="10"
              rx="1.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <path
              d="M4 5h6M4 7.5h4"
              stroke="currentColor"
              strokeWidth="1.2"
              strokeLinecap="round"
            />
          </svg>
        </button>

        {showPatchBrowser && (
          <PatchBrowser onClose={() => setShowPatchBrowser(false)} />
        )}

        {/* Share patch */}
        <button
          className="tb-btn"
          title="Share patch — upload to GitHub Gist and copy link"
          onClick={handleShare}
          disabled={shareState.status === "sharing"}
          style={{ opacity: shareState.status === "sharing" ? 0.5 : 1 }}
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <circle
              cx="11"
              cy="3"
              r="1.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <circle
              cx="3"
              cy="7"
              r="1.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <circle
              cx="11"
              cy="11"
              r="1.5"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <path
              d="M4.4 6.2L9.6 3.8M4.4 7.8L9.6 10.2"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
        </button>

        {/* Share toast */}
        {showShareToast && shareUrl && (
          <div
            style={{
              position: "fixed",
              bottom: 24,
              left: "50%",
              transform: "translateX(-50%)",
              background: "#0c1420",
              border: "1px solid rgba(0,229,160,0.3)",
              borderRadius: 8,
              padding: "10px 16px",
              display: "flex",
              alignItems: "center",
              gap: 10,
              zIndex: 2000,
              boxShadow: "0 8px 32px rgba(0,0,0,0.6)",
              maxWidth: 480,
            }}
          >
            <span style={{ fontSize: 14 }}>✓</span>
            <div>
              <div
                style={{
                  fontSize: 12,
                  color: "#00e5a0",
                  fontWeight: 600,
                  marginBottom: 2,
                }}
              >
                Patch shared — link copied to clipboard
              </div>
              <div
                style={{
                  fontSize: 10,
                  color: "#4a6a8a",
                  fontFamily: "monospace",
                  wordBreak: "break-all",
                }}
              >
                {shareUrl}
              </div>
            </div>
            <button
              onClick={() => setShowShareToast(false)}
              style={{
                background: "none",
                border: "none",
                color: "#4a6a8a",
                cursor: "pointer",
                fontSize: 14,
                marginLeft: 4,
              }}
            >
              ✕
            </button>
          </div>
        )}

        {shareState.status === "error" && shareState.error && (
          <div
            style={{
              position: "fixed",
              bottom: 24,
              left: "50%",
              transform: "translateX(-50%)",
              background: "#0c1420",
              border: "1px solid rgba(239,83,80,0.3)",
              borderRadius: 8,
              padding: "10px 16px",
              fontSize: 12,
              color: "#ef5350",
              zIndex: 2000,
              boxShadow: "0 8px 32px rgba(0,0,0,0.6)",
            }}
          >
            Share failed: {shareState.error}
          </div>
        )}

        {showModMatrix && (
          <ModulationMatrix onClose={() => setShowModMatrix(false)} />
        )}

        <div className="tb-divider" />

        {/* Settings */}
        <button
          className={`tb-btn ${showSettings ? "active" : ""}`}
          title="Settings — audio device, buffer size, MIDI"
          onClick={() => setShowSettings((v) => !v)}
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <circle
              cx="7"
              cy="7"
              r="2"
              stroke="currentColor"
              strokeWidth="1.2"
            />
            <path
              d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13M2.93 2.93l1.06 1.06M10.01 10.01l1.06 1.06M2.93 11.07l1.06-1.06M10.01 3.99l1.06-1.06"
              stroke="currentColor"
              strokeWidth="1.2"
              strokeLinecap="round"
            />
          </svg>
        </button>

        {showSettings && (
          <SettingsPanel onClose={() => setShowSettings(false)} />
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
