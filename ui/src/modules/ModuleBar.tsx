import { useState } from "react";
import type { StudioModuleType } from "./types";
import { useModuleStore } from "./useModuleStore";

interface ModuleButtonDef {
  type: StudioModuleType;
  icon: string;
  label: string;
  color: string;
}

const MODULE_BUTTONS: ModuleButtonDef[] = [
  { type: "Timeline", icon: "⏱", label: "Timeline", color: "#38bdf8" },
  { type: "Mixer", icon: "🎚", label: "Mixer", color: "#a78bfa" },
  { type: "PianoRoll", icon: "🎹", label: "Piano Roll", color: "#34d399" },
  { type: "StepSequencer", icon: "⬛", label: "Step Seq", color: "#fb923c" },
  { type: "Looper", icon: "⟳", label: "Looper", color: "#f472b6" },
  { type: "InstrumentRack", icon: "◈", label: "Rack", color: "#fbbf24" },
];

export function ModuleBar() {
  const addModule = useModuleStore((s) => s.addModule);
  const [hovered, setHovered] = useState<string | null>(null);

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 2,
        padding: "0 16px",
        height: 36,
        background: "#080e18",
        borderBottom: "1px solid #0f1e2e",
        flexShrink: 0,
        overflowX: "auto",
      }}
    >
      <span
        style={{
          fontSize: 9,
          fontFamily: "'Inter', system-ui, sans-serif",
          fontWeight: 600,
          letterSpacing: "0.14em",
          textTransform: "uppercase",
          color: "#334155",
          marginRight: 10,
          whiteSpace: "nowrap",
        }}
      >
        Modules
      </span>

      {MODULE_BUTTONS.map(({ type, icon, label, color }) => {
        const isHovered = hovered === type;
        return (
          <button
            key={type}
            onClick={() => addModule(type)}
            onMouseEnter={() => setHovered(type)}
            onMouseLeave={() => setHovered(null)}
            title={`Open ${label}`}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 5,
              padding: "4px 10px",
              background: isHovered ? `${color}18` : "transparent",
              border: `1px solid ${isHovered ? `${color}30` : "#1a2a3a"}`,
              borderRadius: 5,
              color: isHovered ? color : "#64748b",
              fontSize: 11,
              fontFamily: "'Inter', system-ui, sans-serif",
              fontWeight: 500,
              cursor: "pointer",
              whiteSpace: "nowrap",
              transition: "all 0.12s",
            }}
          >
            <span style={{ fontSize: 12 }}>{icon}</span>
            {label}
          </button>
        );
      })}
    </div>
  );
}
