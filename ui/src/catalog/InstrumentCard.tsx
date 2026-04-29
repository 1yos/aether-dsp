import { useState } from "react";
import type { CatalogInstrument } from "./types";

const REGION_COLORS: Record<string, string> = {
  "East Africa": "#ffb74d",
  "West Africa": "#ef9a9a",
  "North Africa / Middle East": "#ce93d8",
  "South Asia": "#80cbc4",
  "East Asia": "#4fc3f7",
  Europe: "#a5d6a7",
  Americas: "#fff176",
  Electronic: "#b39ddb",
};

const FAMILY_ICONS: Record<string, string> = {
  "plucked-string": "♪",
  "bowed-string": "🎻",
  wind: "♫",
  percussion: "●",
  keyboard: "⌨",
  electronic: "⚡",
  voice: "🎤",
};

const COUNTRY_FLAGS: Record<string, string> = {
  Ethiopia: "🇪🇹",
  Eritrea: "🇪🇷",
  "Senegal/Gambia": "🇸🇳",
  "Mali/Guinea": "🇲🇱",
  "Guinea/Mali": "🇬🇳",
  Mali: "🇲🇱",
  "Nigeria/Ghana": "🇳🇬",
  Zimbabwe: "🇿🇼",
  "Nigeria/Cuba": "🇳🇬",
  "Arab World": "🌍",
  "Turkey/Egypt": "🇹🇷",
  "Turkey/Iran": "🇹🇷",
  "Egypt/Turkey": "🇪🇬",
  Turkey: "🇹🇷",
  Egypt: "🇪🇬",
  "Lebanon/Syria": "🇱🇧",
  India: "🇮🇳",
  China: "🇨🇳",
  Japan: "🇯🇵",
  Italy: "🇮🇹",
  France: "🇫🇷",
  "France/Germany": "🇫🇷",
  Scotland: "🏴󠁧󠁢󠁳󠁣󠁴󠁿",
  "Bolivia/Peru": "🇧🇴",
  Venezuela: "🇻🇪",
  Trinidad: "🇹🇹",
  Peru: "🇵🇪",
  "United States": "🇺🇸",
  "Guatemala/Mexico": "🇬🇹",
  Brazil: "🇧🇷",
  Russia: "🇷🇺",
  "United Kingdom": "🇬🇧",
  Germany: "🇩🇪",
  "France (Electronic)": "🇫🇷",
};

interface Props {
  instrument: CatalogInstrument;
  onPreview: () => void;
  onApply: () => void;
  isPlaying: boolean;
  isCustom?: boolean;
  onRemove?: () => void;
}

export function InstrumentCard({
  instrument,
  onPreview,
  onApply,
  isPlaying,
  isCustom,
  onRemove,
}: Props) {
  const [expanded, setExpanded] = useState(false);
  const accent = REGION_COLORS[instrument.region] ?? "#4fc3f7";
  const flag = COUNTRY_FLAGS[instrument.country] ?? "🌍";
  const icon = FAMILY_ICONS[instrument.family] ?? "♪";

  return (
    <div
      onClick={() => setExpanded((e) => !e)}
      style={{
        background: expanded ? "#0d1b2a" : "#0a1520",
        border: `1px solid ${expanded ? accent + "60" : "#1a2a3a"}`,
        borderRadius: 8,
        padding: "10px 12px",
        cursor: "pointer",
        transition: "all 0.15s",
        position: "relative",
      }}
      onMouseEnter={(e) => {
        (e.currentTarget as HTMLDivElement).style.borderColor = accent + "40";
      }}
      onMouseLeave={(e) => {
        if (!expanded)
          (e.currentTarget as HTMLDivElement).style.borderColor = "#1a2a3a";
      }}
    >
      {/* Header row */}
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <div
          style={{
            width: 28,
            height: 28,
            borderRadius: 6,
            background: accent + "18",
            border: `1px solid ${accent}30`,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 14,
            flexShrink: 0,
          }}
        >
          {icon}
        </div>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div
            style={{
              fontSize: 12,
              fontWeight: 600,
              color: "#e2e8f0",
              lineHeight: 1.2,
            }}
          >
            {instrument.name}
            {isCustom && (
              <span
                style={{
                  marginLeft: 6,
                  fontSize: 9,
                  color: accent,
                  background: accent + "20",
                  padding: "1px 5px",
                  borderRadius: 3,
                }}
              >
                CUSTOM
              </span>
            )}
          </div>
          <div style={{ fontSize: 10, color: "#475569", marginTop: 1 }}>
            {flag} {instrument.country}
          </div>
        </div>
        <div
          style={{
            fontSize: 9,
            color: accent,
            background: accent + "15",
            padding: "2px 6px",
            borderRadius: 3,
            flexShrink: 0,
            fontFamily: "monospace",
            letterSpacing: "0.05em",
          }}
        >
          {instrument.tuning.replace("-", " ").toUpperCase()}
        </div>
      </div>

      {/* Expanded content */}
      {expanded && (
        <div style={{ marginTop: 10 }}>
          <p
            style={{
              fontSize: 11,
              color: "#64748b",
              lineHeight: 1.5,
              margin: "0 0 10px",
            }}
          >
            {instrument.description}
          </p>

          {/* Tags */}
          <div
            style={{
              display: "flex",
              flexWrap: "wrap",
              gap: 4,
              marginBottom: 10,
            }}
          >
            {instrument.tags.slice(0, 4).map((tag) => (
              <span
                key={tag}
                style={{
                  fontSize: 9,
                  color: "#334155",
                  background: "#0f1e2e",
                  padding: "2px 6px",
                  borderRadius: 3,
                  fontFamily: "monospace",
                }}
              >
                {tag}
              </span>
            ))}
          </div>

          {/* Action buttons */}
          <div
            style={{ display: "flex", gap: 6 }}
            onClick={(e) => e.stopPropagation()}
          >
            <button
              onClick={onPreview}
              style={{
                flex: 1,
                padding: "6px 0",
                background: isPlaying ? accent + "25" : "#0f1e2e",
                border: `1px solid ${isPlaying ? accent : "#1a2a3a"}`,
                borderRadius: 5,
                color: isPlaying ? accent : "#64748b",
                fontSize: 11,
                cursor: "pointer",
                fontFamily: "monospace",
                animation: isPlaying
                  ? "recPulse 1s ease-in-out infinite"
                  : "none",
              }}
            >
              {isPlaying ? "⏸ Playing" : "▶ Preview"}
            </button>
            <button
              onClick={onApply}
              style={{
                flex: 1,
                padding: "6px 0",
                background: "#0f1e2e",
                border: "1px solid #1a2a3a",
                borderRadius: 5,
                color: "#94a3b8",
                fontSize: 11,
                cursor: "pointer",
                fontFamily: "monospace",
              }}
            >
              Apply to Source
            </button>
            {isCustom && onRemove && (
              <button
                onClick={onRemove}
                style={{
                  padding: "6px 10px",
                  background: "rgba(239,83,80,0.1)",
                  border: "1px solid rgba(239,83,80,0.3)",
                  borderRadius: 5,
                  color: "#ef5350",
                  fontSize: 11,
                  cursor: "pointer",
                }}
              >
                ✕
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
