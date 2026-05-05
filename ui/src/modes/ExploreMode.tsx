/**
 * Aether Studio — Explore Mode
 *
 * The world map is the hero. Instruments are placed at their geographic origin.
 * Clicking a region reveals its instruments. Clicking an instrument opens a
 * full-screen immersive view with cultural context and the keyboard player.
 */

import { useState, useEffect, useCallback } from "react";
import { useCatalog } from "../catalog/useCatalog";
import { getCountryFlag } from "../catalog/countryFlags";
import type { CatalogInstrument } from "../catalog/types";
import { KeyboardPlayer } from "../components/KeyboardPlayer";
import { useModeStore } from "../store/useModeStore";
import { useEngineStore } from "../studio/store/engineStore";
import "./ExploreMode.css";

// ── Region definitions with geographic centers and cultural identity ──────────

interface RegionDef {
  id: string;
  label: string;
  catalogRegion: string;
  // SVG path coordinates for the region highlight on the world map
  cx: number; // center x (0-100 percentage of map width)
  cy: number; // center y (0-100 percentage of map height)
  color: string;
  colorDeep: string;
  colorText: string;
  colorBg: string;
  // Cultural tagline
  tagline: string;
}

const REGIONS: RegionDef[] = [
  {
    id: "east-africa",
    label: "East Africa",
    catalogRegion: "East Africa",
    cx: 56,
    cy: 55,
    color: "var(--region-east-africa)",
    colorDeep: "var(--region-east-africa-deep)",
    colorText: "var(--region-east-africa-text)",
    colorBg: "var(--region-east-africa-bg)",
    tagline: "Lyres, flutes & sacred drums of the Horn",
  },
  {
    id: "west-africa",
    label: "West Africa",
    catalogRegion: "West Africa",
    cx: 44,
    cy: 52,
    color: "var(--region-west-africa)",
    colorDeep: "var(--region-west-africa-deep)",
    colorText: "var(--region-west-africa-text)",
    colorBg: "var(--region-west-africa-bg)",
    tagline: "Koras, balafons & the talking drum",
  },
  {
    id: "middle-east",
    label: "Middle East",
    catalogRegion: "North Africa / Middle East",
    cx: 60,
    cy: 42,
    color: "var(--region-middle-east)",
    colorDeep: "var(--region-middle-east-deep)",
    colorText: "var(--region-middle-east-text)",
    colorBg: "var(--region-middle-east-bg)",
    tagline: "Ouds, qanuns & the breath of the ney",
  },
  {
    id: "south-asia",
    label: "South Asia",
    catalogRegion: "South Asia",
    cx: 70,
    cy: 48,
    color: "var(--region-south-asia)",
    colorDeep: "var(--region-south-asia-deep)",
    colorText: "var(--region-south-asia-text)",
    colorBg: "var(--region-south-asia-bg)",
    tagline: "Sitars, tablas & the raga tradition",
  },
  {
    id: "east-asia",
    label: "East Asia",
    catalogRegion: "East Asia",
    cx: 82,
    cy: 40,
    color: "var(--region-east-asia)",
    colorDeep: "var(--region-east-asia-deep)",
    colorText: "var(--region-east-asia-text)",
    colorBg: "var(--region-east-asia-bg)",
    tagline: "Guzheng, erhu & the taiko tradition",
  },
  {
    id: "europe",
    label: "Europe",
    catalogRegion: "Europe",
    cx: 50,
    cy: 32,
    color: "var(--region-europe)",
    colorDeep: "var(--region-europe-deep)",
    colorText: "var(--region-europe-text)",
    colorBg: "var(--region-europe-bg)",
    tagline: "Strings, winds & the orchestral tradition",
  },
  {
    id: "americas",
    label: "Americas",
    catalogRegion: "Americas",
    cx: 22,
    cy: 50,
    color: "var(--region-americas)",
    colorDeep: "var(--region-americas-deep)",
    colorText: "var(--region-americas-text)",
    colorBg: "var(--region-americas-bg)",
    tagline: "Charangos, steel pans & Andean winds",
  },
  {
    id: "electronic",
    label: "Electronic",
    catalogRegion: "Electronic",
    cx: 50,
    cy: 80,
    color: "var(--region-electronic)",
    colorDeep: "var(--region-electronic-deep)",
    colorText: "var(--region-electronic-text)",
    colorBg: "var(--region-electronic-bg)",
    tagline: "Synthesizers, theremins & digital sound",
  },
];

function mapRegionToId(catalogRegion: string): string {
  const r = REGIONS.find((r) => r.catalogRegion === catalogRegion);
  return r?.id || "electronic";
}

function getRegionDef(catalogRegion: string): RegionDef {
  return REGIONS.find((r) => r.catalogRegion === catalogRegion) || REGIONS[7];
}

// ── World Map SVG paths (simplified continents) ───────────────────────────────
// These are simplified continent outlines for the decorative map background

const CONTINENT_PATHS = {
  africa:
    "M 48 42 L 52 40 L 58 41 L 62 44 L 64 50 L 62 58 L 58 64 L 54 66 L 50 64 L 46 58 L 44 52 L 46 46 Z",
  europe: "M 44 28 L 52 26 L 58 28 L 60 32 L 56 34 L 50 34 L 44 32 Z",
  asia: "M 58 28 L 72 26 L 86 28 L 90 34 L 88 40 L 82 44 L 74 46 L 66 44 L 60 40 L 58 34 Z",
  northAmerica:
    "M 10 24 L 28 22 L 34 28 L 32 36 L 26 42 L 18 44 L 12 40 L 8 32 Z",
  southAmerica:
    "M 22 44 L 30 42 L 34 48 L 32 58 L 28 66 L 22 68 L 18 62 L 18 52 Z",
  australia: "M 76 58 L 86 56 L 90 62 L 86 68 L 78 68 L 74 64 Z",
};

// ── Instrument detail panel ───────────────────────────────────────────────────

interface InstrumentDetailProps {
  instrument: CatalogInstrument;
  region: RegionDef;
  onClose: () => void;
  onTryIt: () => void;
  onAddToCanvas: () => void;
}

function InstrumentDetail({
  instrument,
  region,
  onClose,
  onTryIt,
  onAddToCanvas,
}: InstrumentDetailProps) {
  const flag = getCountryFlag(instrument.country);

  return (
    <div
      className="instrument-detail"
      style={
        {
          "--region-color": region.color,
          "--region-text": region.colorText,
          "--region-bg": region.colorBg,
        } as React.CSSProperties
      }
    >
      <div className="detail-backdrop" onClick={onClose} />
      <div className="detail-panel animate-scale-in">
        {/* Close */}
        <button className="detail-close" onClick={onClose}>
          ✕
        </button>

        {/* Header */}
        <div className="detail-header">
          <div className="detail-flag">{flag}</div>
          <div className="detail-title-group">
            <div className="detail-region-label">{region.label}</div>
            <h1 className="detail-name">{instrument.name}</h1>
            <div className="detail-origin">
              {instrument.country} · {instrument.family.replace(/-/g, " ")}
            </div>
          </div>
        </div>

        {/* Visual — instrument illustration area */}
        <div className="detail-visual">
          <div className="detail-visual-inner">
            <div className="detail-instrument-icon">{flag}</div>
            <div className="detail-waveform">
              <svg viewBox="0 0 400 80" preserveAspectRatio="none">
                <defs>
                  <linearGradient
                    id="waveGrad"
                    x1="0%"
                    y1="0%"
                    x2="100%"
                    y2="0%"
                  >
                    <stop
                      offset="0%"
                      stopColor={region.color}
                      stopOpacity="0"
                    />
                    <stop
                      offset="20%"
                      stopColor={region.color}
                      stopOpacity="0.8"
                    />
                    <stop
                      offset="80%"
                      stopColor={region.color}
                      stopOpacity="0.8"
                    />
                    <stop
                      offset="100%"
                      stopColor={region.color}
                      stopOpacity="0"
                    />
                  </linearGradient>
                </defs>
                <path
                  d="M0,40 C20,40 30,10 50,40 C70,70 80,10 100,40 C120,70 130,15 150,40 C170,65 180,20 200,40 C220,60 230,15 250,40 C270,65 280,20 300,40 C320,60 330,25 350,40 C370,55 380,30 400,40"
                  stroke="url(#waveGrad)"
                  strokeWidth="2"
                  fill="none"
                  className="detail-wave-path"
                />
              </svg>
            </div>
          </div>
        </div>

        {/* Description */}
        <div className="detail-description">
          <p>{instrument.description}</p>
        </div>

        {/* Tuning badge */}
        <div className="detail-tuning">
          <span className="tuning-label">Tuning System</span>
          <span className="tuning-value">
            {instrument.tuning.replace(/-/g, " ")}
          </span>
        </div>

        {/* Tags */}
        <div className="detail-tags">
          {instrument.tags.map((tag) => (
            <span key={tag} className="detail-tag">
              {tag}
            </span>
          ))}
        </div>

        {/* Actions */}
        <div className="detail-actions">
          <button className="detail-btn detail-btn-primary" onClick={onTryIt}>
            <span className="btn-icon">🎹</span>
            <span>Try It</span>
            <span className="btn-hint">Play with keyboard</span>
          </button>
          <button
            className="detail-btn detail-btn-secondary"
            onClick={onAddToCanvas}
          >
            <span className="btn-icon">＋</span>
            <span>Add to Canvas</span>
            <span className="btn-hint">Open in Create mode</span>
          </button>
        </div>
      </div>
    </div>
  );
}

// ── Region panel — shows instruments for a selected region ────────────────────

interface RegionPanelProps {
  region: RegionDef;
  instruments: CatalogInstrument[];
  onSelectInstrument: (inst: CatalogInstrument) => void;
  onClose: () => void;
}

function RegionPanel({
  region,
  instruments,
  onSelectInstrument,
  onClose,
}: RegionPanelProps) {
  return (
    <div
      className="region-panel animate-fade-up"
      style={
        {
          "--region-color": region.color,
          "--region-text": region.colorText,
          "--region-bg": region.colorBg,
        } as React.CSSProperties
      }
    >
      <div className="region-panel-header">
        <div>
          <div className="region-panel-label">Region</div>
          <h2 className="region-panel-name">{region.label}</h2>
          <p className="region-panel-tagline">{region.tagline}</p>
        </div>
        <button className="region-panel-close" onClick={onClose}>
          ✕
        </button>
      </div>

      <div className="region-instruments">
        {instruments.map((inst, i) => (
          <button
            key={inst.id}
            className="region-instrument-item"
            onClick={() => onSelectInstrument(inst)}
            style={{ animationDelay: `${i * 40}ms` }}
          >
            <span className="ri-flag">{getCountryFlag(inst.country)}</span>
            <div className="ri-info">
              <span className="ri-name">{inst.name}</span>
              <span className="ri-meta">
                {inst.country} · {inst.family.replace(/-/g, " ")}
              </span>
            </div>
            <span className="ri-arrow">→</span>
          </button>
        ))}
      </div>
    </div>
  );
}

// ── World Map ─────────────────────────────────────────────────────────────────

interface WorldMapProps {
  activeRegion: string | null;
  onSelectRegion: (regionId: string) => void;
  instrumentCounts: Record<string, number>;
}

function WorldMap({
  activeRegion,
  onSelectRegion,
  instrumentCounts,
}: WorldMapProps) {
  const [hoveredRegion, setHoveredRegion] = useState<string | null>(null);

  return (
    <div className="world-map">
      {/* Decorative star field */}
      <div className="map-stars" aria-hidden="true">
        {Array.from({ length: 80 }).map((_, i) => (
          <div
            key={i}
            className="map-star"
            style={{
              left: `${Math.random() * 100}%`,
              top: `${Math.random() * 100}%`,
              width: `${Math.random() * 2 + 1}px`,
              height: `${Math.random() * 2 + 1}px`,
              animationDelay: `${Math.random() * 4}s`,
              animationDuration: `${Math.random() * 3 + 2}s`,
            }}
          />
        ))}
      </div>

      {/* SVG world map */}
      <svg
        className="map-svg"
        viewBox="0 0 100 70"
        preserveAspectRatio="xMidYMid meet"
      >
        {/* Continent outlines — decorative */}
        {Object.entries(CONTINENT_PATHS).map(([name, path]) => (
          <path key={name} d={path} className="continent-path" />
        ))}

        {/* Region hotspots */}
        {REGIONS.map((region) => {
          const isActive = activeRegion === region.id;
          const isHovered = hoveredRegion === region.id;
          const count = instrumentCounts[region.catalogRegion] || 0;

          return (
            <g key={region.id}>
              {/* Outer glow ring */}
              {(isActive || isHovered) && (
                <circle
                  cx={region.cx}
                  cy={region.cy}
                  r={isActive ? 6 : 5}
                  fill="none"
                  stroke={region.color}
                  strokeWidth="0.3"
                  opacity="0.4"
                  className="region-glow-ring"
                />
              )}

              {/* Main hotspot */}
              <circle
                cx={region.cx}
                cy={region.cy}
                r={isActive ? 3.5 : isHovered ? 3 : 2.5}
                fill={
                  isActive || isHovered
                    ? region.color
                    : "rgba(255,255,255,0.15)"
                }
                stroke={region.color}
                strokeWidth="0.4"
                className="region-hotspot"
                style={{ cursor: "pointer", transition: "all 0.2s ease" }}
                onClick={() => onSelectRegion(region.id)}
                onMouseEnter={() => setHoveredRegion(region.id)}
                onMouseLeave={() => setHoveredRegion(null)}
              />

              {/* Instrument count badge */}
              <text
                x={region.cx}
                y={region.cy + 0.4}
                textAnchor="middle"
                dominantBaseline="middle"
                fontSize="1.4"
                fill={isActive || isHovered ? "#000" : region.color}
                fontWeight="bold"
                style={{ pointerEvents: "none", userSelect: "none" }}
              >
                {count}
              </text>

              {/* Region label */}
              <text
                x={region.cx}
                y={region.cy + (region.cy > 50 ? -4.5 : 4.5)}
                textAnchor="middle"
                fontSize="1.8"
                fill={
                  isActive || isHovered
                    ? region.colorText
                    : "rgba(255,255,255,0.5)"
                }
                fontWeight={isActive ? "bold" : "normal"}
                style={{
                  pointerEvents: "none",
                  userSelect: "none",
                  transition: "all 0.2s ease",
                }}
              >
                {region.label}
              </text>
            </g>
          );
        })}
      </svg>

      {/* Map legend */}
      <div className="map-legend">
        <span className="legend-dot" />
        <span className="legend-text">
          Click a region to explore its instruments
        </span>
      </div>
    </div>
  );
}

// ── Main ExploreMode ──────────────────────────────────────────────────────────

export function ExploreMode() {
  const { instruments, searchQuery, setSearchQuery } = useCatalog();
  const [activeRegion, setActiveRegion] = useState<string | null>(null);
  const [selectedInstrument, setSelectedInstrument] =
    useState<CatalogInstrument | null>(null);
  const [keyboardInstrument, setKeyboardInstrument] =
    useState<CatalogInstrument | null>(null);
  const [isSearching, setIsSearching] = useState(false);

  const setMode = useModeStore((s) => s.setMode);
  const loadPatch = useEngineStore((s) => s.loadPatch);

  const handleAddToCanvas = useCallback(
    async (instrument: CatalogInstrument) => {
      // Load the instrument's .aether-instrument preset file
      const presetId = instrument.id.toLowerCase().replace(/\s+/g, "-");
      try {
        const res = await fetch(`/instruments/${presetId}.aether-instrument`);
        if (res.ok) {
          const patch = await res.json();
          loadPatch(patch);
        } else {
          // Fallback: load a generic SamplerNode patch for this instrument
          loadPatch({
            version: "1.0",
            nodes: [{ id: "sampler-0", type: "SamplerNode", params: {} }],
            connections: [],
            output_node: "sampler-0",
          });
        }
      } catch {
        // Network error — still switch to Create mode with empty graph
      }
      setSelectedInstrument(null);
      setKeyboardInstrument(null);
      setMode("create");
    },
    [loadPatch, setMode],
  );

  // Count instruments per region
  const instrumentCounts = instruments.reduce<Record<string, number>>(
    (acc, inst) => {
      acc[inst.region] = (acc[inst.region] || 0) + 1;
      return acc;
    },
    {},
  );

  // Instruments for the active region
  const regionInstruments = activeRegion
    ? instruments.filter((i) => mapRegionToId(i.region) === activeRegion)
    : [];

  // Search results
  const searchResults = searchQuery.trim()
    ? instruments.filter(
        (i) =>
          i.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          i.country.toLowerCase().includes(searchQuery.toLowerCase()) ||
          i.tags.some((t) =>
            t.toLowerCase().includes(searchQuery.toLowerCase()),
          ),
      )
    : [];

  const activeRegionDef = REGIONS.find((r) => r.id === activeRegion);

  const handleSelectRegion = useCallback((regionId: string) => {
    setActiveRegion((prev) => (prev === regionId ? null : regionId));
    setSelectedInstrument(null);
  }, []);

  const handleSelectInstrument = useCallback((inst: CatalogInstrument) => {
    setSelectedInstrument(inst);
  }, []);

  const handleTryIt = useCallback(() => {
    if (selectedInstrument) {
      setKeyboardInstrument(selectedInstrument);
    }
  }, [selectedInstrument]);

  // Close on Escape
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (keyboardInstrument) {
          setKeyboardInstrument(null);
          return;
        }
        if (selectedInstrument) {
          setSelectedInstrument(null);
          return;
        }
        if (activeRegion) {
          setActiveRegion(null);
          return;
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [keyboardInstrument, selectedInstrument, activeRegion]);

  return (
    <div className="explore-mode">
      {/* Top search bar */}
      <div className="explore-topbar">
        <div className="explore-brand">
          <span className="explore-brand-icon">◉</span>
          <span className="explore-brand-text">World Instruments</span>
          <span className="explore-brand-count">
            {instruments.length} instruments
          </span>
        </div>

        <div className={`explore-search ${isSearching ? "active" : ""}`}>
          <span className="search-icon">⌕</span>
          <input
            type="text"
            placeholder="Search instruments, regions, traditions..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onFocus={() => setIsSearching(true)}
            onBlur={() => !searchQuery && setIsSearching(false)}
            className="search-input"
          />
          {searchQuery && (
            <button
              className="search-clear"
              onClick={() => {
                setSearchQuery("");
                setIsSearching(false);
              }}
            >
              ✕
            </button>
          )}
        </div>

        <div className="explore-hint">
          <kbd>1</kbd> Explore · <kbd>2</kbd> Create · <kbd>3</kbd> Arrange ·{" "}
          <kbd>4</kbd> Perform
        </div>
      </div>

      {/* Search results overlay */}
      {searchQuery && (
        <div className="search-results animate-fade-down">
          <div className="search-results-header">
            {searchResults.length} results for "{searchQuery}"
          </div>
          <div className="search-results-grid">
            {searchResults.map((inst) => {
              const region = getRegionDef(inst.region);
              return (
                <button
                  key={inst.id}
                  className="search-result-item"
                  style={
                    { "--region-color": region.color } as React.CSSProperties
                  }
                  onClick={() => {
                    setSelectedInstrument(inst);
                    setActiveRegion(mapRegionToId(inst.region));
                    setSearchQuery("");
                    setIsSearching(false);
                  }}
                >
                  <span className="sr-flag">
                    {getCountryFlag(inst.country)}
                  </span>
                  <div className="sr-info">
                    <span className="sr-name">{inst.name}</span>
                    <span className="sr-meta">
                      {inst.country} · {region.label}
                    </span>
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      )}

      {/* Main content */}
      <div className="explore-main">
        {/* World map */}
        <div className="explore-map-container">
          <WorldMap
            activeRegion={activeRegion}
            onSelectRegion={handleSelectRegion}
            instrumentCounts={instrumentCounts}
          />

          {/* Region stats overlay */}
          <div className="region-stats">
            {REGIONS.map((region) => (
              <button
                key={region.id}
                className={`region-stat-pill ${activeRegion === region.id ? "active" : ""}`}
                style={
                  {
                    "--region-color": region.color,
                    "--region-text": region.colorText,
                  } as React.CSSProperties
                }
                onClick={() => handleSelectRegion(region.id)}
              >
                <span className="rsp-dot" />
                <span className="rsp-label">{region.label}</span>
                <span className="rsp-count">
                  {instrumentCounts[region.catalogRegion] || 0}
                </span>
              </button>
            ))}
          </div>
        </div>

        {/* Region panel — slides in when a region is selected */}
        {activeRegion && activeRegionDef && !selectedInstrument && (
          <RegionPanel
            region={activeRegionDef}
            instruments={regionInstruments}
            onSelectInstrument={handleSelectInstrument}
            onClose={() => setActiveRegion(null)}
          />
        )}
      </div>

      {/* Instrument detail overlay */}
      {selectedInstrument && (
        <InstrumentDetail
          instrument={selectedInstrument}
          region={getRegionDef(selectedInstrument.region)}
          onClose={() => setSelectedInstrument(null)}
          onTryIt={handleTryIt}
          onAddToCanvas={() => handleAddToCanvas(selectedInstrument)}
        />
      )}

      {/* Keyboard player */}
      {keyboardInstrument && (
        <KeyboardPlayer
          instrument={{
            ...keyboardInstrument,
            flag: getCountryFlag(keyboardInstrument.country),
          }}
          onClose={() => setKeyboardInstrument(null)}
          onAddToCanvas={() => handleAddToCanvas(keyboardInstrument)}
        />
      )}
    </div>
  );
}
