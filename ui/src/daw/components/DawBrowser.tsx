/**
 * DawBrowser — left panel with Instruments, Samples, Presets tabs.
 * The world instrument catalog lives here as a proper browser.
 */

import { useState } from "react";
import { useDawStore } from "../store/dawStore";
import { useCatalog } from "../../catalog/useCatalog";
import { getCountryFlag } from "../../catalog/countryFlags";
import type { CatalogInstrument } from "../../catalog/types";
import { useEngineStore } from "../../studio/store/engineStore";

const TABS = [
  { id: "instruments" as const, label: "Instruments", icon: "🎹" },
  { id: "samples" as const, label: "Samples", icon: "🎵" },
  { id: "presets" as const, label: "Presets", icon: "⚙" },
  { id: "plugins" as const, label: "Plugins", icon: "🔌" },
];

const REGIONS = [
  "All",
  "East Africa",
  "West Africa",
  "North Africa / Middle East",
  "South Asia",
  "East Asia",
  "Europe",
  "Americas",
  "Electronic",
];

interface DawBrowserProps {
  width: number;
}

export function DawBrowser({ width }: DawBrowserProps) {
  const browserTab = useDawStore((s) => s.browserTab);
  const setBrowserTab = useDawStore((s) => s.setBrowserTab);
  const loadPatch = useEngineStore((s) => s.loadPatch);
  const setView = useDawStore((s) => s.setView);

  const { instruments } = useCatalog();
  const [search, setSearch] = useState("");
  const [region, setRegion] = useState("All");

  const filtered = instruments.filter((i) => {
    const matchRegion = region === "All" || i.region === region;
    const matchSearch =
      !search ||
      i.name.toLowerCase().includes(search.toLowerCase()) ||
      i.country.toLowerCase().includes(search.toLowerCase());
    return matchRegion && matchSearch;
  });

  const handleLoadInstrument = async (inst: CatalogInstrument) => {
    const presetId = inst.id.toLowerCase().replace(/\s+/g, "-");
    try {
      const res = await fetch(`/instruments/${presetId}.aether-instrument`);
      if (res.ok) {
        const patch = await res.json();
        loadPatch(patch);
      } else {
        loadPatch({
          version: "1.0",
          nodes: [{ id: "sampler-0", type: "SamplerNode", params: {} }],
          connections: [],
          output_node: "sampler-0",
        });
      }
    } catch {
      /* ignore */
    }
    setView("patcher");
  };

  const bg = "#080e18";
  const border = "#0f1e2e";
  const text = "#e0e8f0";
  const dim = "#475569";
  const accent = "#4db8ff";

  return (
    <div
      style={{
        width,
        background: bg,
        borderRight: `1px solid ${border}`,
        display: "flex",
        flexDirection: "column",
        flexShrink: 0,
        overflow: "hidden",
      }}
    >
      {/* Tab bar */}
      <div
        style={{
          display: "flex",
          borderBottom: `1px solid ${border}`,
          flexShrink: 0,
        }}
      >
        {TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setBrowserTab(tab.id)}
            title={tab.label}
            style={{
              flex: 1,
              padding: "8px 4px",
              background:
                browserTab === tab.id ? "rgba(77,184,255,0.08)" : "transparent",
              border: "none",
              borderBottom: `2px solid ${browserTab === tab.id ? accent : "transparent"}`,
              color: browserTab === tab.id ? accent : dim,
              fontSize: 14,
              cursor: "pointer",
              transition: "all 0.1s",
            }}
          >
            {tab.icon}
          </button>
        ))}
      </div>

      {/* Search */}
      <div style={{ padding: "8px 8px 4px", flexShrink: 0 }}>
        <input
          type="text"
          placeholder="Search..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={{
            width: "100%",
            background: "#0a1520",
            border: `1px solid ${border}`,
            borderRadius: 5,
            color: text,
            padding: "5px 8px",
            fontSize: 11,
            fontFamily: "var(--font-sans)",
            boxSizing: "border-box",
            outline: "none",
          }}
        />
      </div>

      {/* Instruments tab */}
      {browserTab === "instruments" && (
        <>
          {/* Region filter */}
          <div
            style={{
              padding: "0 8px 6px",
              flexShrink: 0,
            }}
          >
            <select
              value={region}
              onChange={(e) => setRegion(e.target.value)}
              style={{
                width: "100%",
                background: "#0a1520",
                border: `1px solid ${border}`,
                borderRadius: 5,
                color: dim,
                padding: "4px 6px",
                fontSize: 10,
                fontFamily: "var(--font-sans)",
                cursor: "pointer",
              }}
            >
              {REGIONS.map((r) => (
                <option key={r} value={r}>
                  {r}
                </option>
              ))}
            </select>
          </div>

          {/* Instrument list */}
          <div style={{ flex: 1, overflowY: "auto" }}>
            {filtered.map((inst) => (
              <button
                key={inst.id}
                onDoubleClick={() => handleLoadInstrument(inst)}
                style={{
                  width: "100%",
                  display: "flex",
                  alignItems: "center",
                  gap: 7,
                  padding: "6px 10px",
                  background: "transparent",
                  border: "none",
                  borderBottom: `1px solid ${border}`,
                  color: text,
                  fontSize: 11,
                  fontFamily: "var(--font-sans)",
                  cursor: "pointer",
                  textAlign: "left",
                  transition: "background 0.1s",
                }}
                onMouseEnter={(e) =>
                  (e.currentTarget.style.background = "rgba(255,255,255,0.03)")
                }
                onMouseLeave={(e) =>
                  (e.currentTarget.style.background = "transparent")
                }
                title={`${inst.name} — ${inst.country}\nDouble-click to load`}
              >
                <span style={{ fontSize: 14, flexShrink: 0 }}>
                  {getCountryFlag(inst.country)}
                </span>
                <div style={{ minWidth: 0 }}>
                  <div
                    style={{
                      fontSize: 11,
                      fontWeight: 500,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {inst.name}
                  </div>
                  <div
                    style={{
                      fontSize: 9,
                      color: dim,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {inst.country} · {inst.family.replace(/-/g, " ")}
                  </div>
                </div>
              </button>
            ))}
            {filtered.length === 0 && (
              <div
                style={{
                  padding: 16,
                  textAlign: "center",
                  color: dim,
                  fontSize: 11,
                }}
              >
                No instruments found
              </div>
            )}
          </div>

          <div
            style={{
              padding: "6px 10px",
              borderTop: `1px solid ${border}`,
              fontSize: 9,
              color: "#1e2d3d",
            }}
          >
            {filtered.length} instruments · Double-click to load
          </div>
        </>
      )}

      {/* Samples tab */}
      {browserTab === "samples" && (
        <div
          style={{
            flex: 1,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexDirection: "column",
            gap: 8,
            color: dim,
          }}
        >
          <span style={{ fontSize: 24 }}>🎵</span>
          <span style={{ fontSize: 11 }}>No samples installed</span>
          <span
            style={{
              fontSize: 10,
              color: "#1e2d3d",
              textAlign: "center",
              padding: "0 16px",
            }}
          >
            Use the Sample Library to download instrument packs
          </span>
        </div>
      )}

      {/* Presets tab */}
      {browserTab === "presets" && (
        <div
          style={{
            flex: 1,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexDirection: "column",
            gap: 8,
            color: dim,
          }}
        >
          <span style={{ fontSize: 24 }}>⚙</span>
          <span style={{ fontSize: 11 }}>No presets saved</span>
          <span
            style={{
              fontSize: 10,
              color: "#1e2d3d",
              textAlign: "center",
              padding: "0 16px",
            }}
          >
            Save your patches as presets to reuse them
          </span>
        </div>
      )}

      {/* Plugins tab */}
      {browserTab === "plugins" && (
        <div
          style={{
            flex: 1,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexDirection: "column",
            gap: 8,
            color: dim,
          }}
        >
          <span style={{ fontSize: 24 }}>🔌</span>
          <span style={{ fontSize: 11 }}>VST3 / CLAP plugins</span>
          <span
            style={{
              fontSize: 10,
              color: "#1e2d3d",
              textAlign: "center",
              padding: "0 16px",
            }}
          >
            Plugin hosting coming in v0.3
          </span>
        </div>
      )}
    </div>
  );
}
