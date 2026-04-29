/**
 * Aether Studio v2.0 - Explore Mode
 * Browse and try instruments from around the world
 */

import { useState } from "react";
import { useCatalog } from "../catalog/useCatalog";
import { getCountryFlag } from "../catalog/countryFlags";
import type { CatalogInstrument } from "../catalog/types";
import type { Region } from "../catalog/types";
import { KeyboardPlayer } from "../components/KeyboardPlayer";
import "./ExploreMode.css";

const REGIONS: { id: Region; label: string; color: string }[] = [
  {
    id: "east-africa",
    label: "East Africa",
    color: "var(--region-east-africa)",
  },
  {
    id: "west-africa",
    label: "West Africa",
    color: "var(--region-west-africa)",
  },
  {
    id: "middle-east",
    label: "Middle East",
    color: "var(--region-middle-east)",
  },
  { id: "south-asia", label: "South Asia", color: "var(--region-south-asia)" },
  { id: "east-asia", label: "East Asia", color: "var(--region-east-asia)" },
  { id: "europe", label: "Europe", color: "var(--region-europe)" },
  { id: "americas", label: "Americas", color: "var(--region-americas)" },
  {
    id: "electronic",
    label: "Electronic",
    color: "var(--region-electronic)",
  },
];

// Map catalog regions to our region IDs
function mapRegionToId(catalogRegion: string): Region {
  const mapping: Record<string, Region> = {
    "East Africa": "east-africa",
    "West Africa": "west-africa",
    "North Africa / Middle East": "middle-east",
    "South Asia": "south-asia",
    "East Asia": "east-asia",
    Europe: "europe",
    Americas: "americas",
    Electronic: "electronic",
  };
  return mapping[catalogRegion] || "electronic";
}

export function ExploreMode() {
  const {
    instruments,
    searchQuery,
    setSearchQuery,
    selectedRegion,
    setSelectedRegion,
  } = useCatalog();
  const [heroInstrument, setHeroInstrument] =
    useState<CatalogInstrument | null>(
      instruments.find((i) => i.name === "Krar") || instruments[0] || null,
    );
  const [keyboardPlayerInstrument, setKeyboardPlayerInstrument] =
    useState<CatalogInstrument | null>(null);

  const filteredInstruments = instruments.filter((inst) => {
    const matchesSearch =
      !searchQuery ||
      inst.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      inst.country.toLowerCase().includes(searchQuery.toLowerCase()) ||
      inst.family.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesRegion =
      !selectedRegion || mapRegionToId(inst.region) === selectedRegion;

    return matchesSearch && matchesRegion && inst !== heroInstrument;
  });

  // Add flag to instrument for KeyboardPlayer
  const instrumentWithFlag = (inst: CatalogInstrument) => ({
    ...inst,
    flag: getCountryFlag(inst.country),
    tuning: inst.tuning,
  });

  return (
    <div className="explore-mode">
      {/* Search Bar */}
      <div className="explore-header">
        <h1 className="explore-title">
          <span className="title-icon">🌍</span>
          Instrument Catalog
        </h1>
        <div className="explore-actions">
          <input
            type="text"
            className="search-input"
            placeholder="Search instruments..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
          <button className="add-custom-btn">+ Custom</button>
        </div>
      </div>

      {/* Region Tabs */}
      <div className="region-tabs">
        <button
          className={`region-tab ${!selectedRegion ? "active" : ""}`}
          onClick={() => setSelectedRegion(null)}
        >
          All Regions
        </button>
        {REGIONS.map((region) => (
          <button
            key={region.id}
            className={`region-tab ${selectedRegion === region.id ? "active" : ""}`}
            style={{ "--region-color": region.color } as React.CSSProperties}
            onClick={() => setSelectedRegion(region.id)}
          >
            {region.label}
          </button>
        ))}
      </div>

      {/* Hero Card */}
      {heroInstrument && (
        <div
          className="hero-card"
          style={
            {
              "--region-color": REGIONS.find(
                (r) => r.id === mapRegionToId(heroInstrument.region),
              )?.color,
            } as React.CSSProperties
          }
        >
          <div className="hero-content">
            <div className="hero-header">
              <span className="hero-flag">
                {getCountryFlag(heroInstrument.country)}
              </span>
              <h2 className="hero-name">{heroInstrument.name}</h2>
              <span className="hero-meta">
                {heroInstrument.country} • {heroInstrument.family}
              </span>
            </div>

            <div className="hero-image">
              <div className="hero-image-placeholder">
                <span className="hero-flag-large">
                  {getCountryFlag(heroInstrument.country)}
                </span>
              </div>
            </div>

            <div className="hero-waveform">
              <svg width="100%" height="60" viewBox="0 0 800 60">
                <path
                  d="M0,30 Q50,10 100,30 T200,30 T300,30 T400,30 T500,30 T600,30 T700,30 T800,30"
                  stroke="currentColor"
                  strokeWidth="2"
                  fill="none"
                  opacity="0.6"
                />
              </svg>
            </div>

            <div className="hero-description">
              <p>{heroInstrument.description}</p>
            </div>

            <div className="hero-actions">
              <button className="hero-btn preview-btn">
                <span>▶</span> Preview
              </button>
              <button
                className="hero-btn try-btn"
                onClick={() => setKeyboardPlayerInstrument(heroInstrument)}
              >
                <span>🎹</span> Try It
              </button>
              <button className="hero-btn add-btn">
                <span>➕</span> Add to Canvas
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Instrument Grid */}
      <div className="instrument-grid">
        {filteredInstruments.map((instrument) => (
          <div
            key={instrument.id}
            className="instrument-card"
            style={
              {
                "--region-color": REGIONS.find(
                  (r) => r.id === mapRegionToId(instrument.region),
                )?.color,
              } as React.CSSProperties
            }
            onClick={() => setHeroInstrument(instrument)}
          >
            <div className="card-image">
              <span className="card-flag">
                {getCountryFlag(instrument.country)}
              </span>
            </div>
            <div className="card-content">
              <h3 className="card-name">{instrument.name}</h3>
              <p className="card-meta">{instrument.country}</p>
              <div className="card-badges">
                <span className="badge family-badge">{instrument.family}</span>
                <span className="badge tuning-badge">{instrument.tuning}</span>
              </div>
            </div>
            <div className="card-actions">
              <button
                className="card-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  setKeyboardPlayerInstrument(instrument);
                }}
                title="Try It"
              >
                🎹
              </button>
              <button className="card-btn" title="Add to Canvas">
                ➕
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Keyboard Player Overlay */}
      {keyboardPlayerInstrument && (
        <KeyboardPlayer
          instrument={instrumentWithFlag(keyboardPlayerInstrument)}
          onClose={() => setKeyboardPlayerInstrument(null)}
          onAddToCanvas={() => {
            // TODO: Add to canvas
            setKeyboardPlayerInstrument(null);
          }}
        />
      )}
    </div>
  );
}
