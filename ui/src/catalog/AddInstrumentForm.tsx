import { useState, useRef } from "react";
import type {
  CatalogInstrument,
  InstrumentFamily,
  TuningSystem,
  CatalogRegion,
} from "./types";

const FAMILIES: InstrumentFamily[] = [
  "plucked-string",
  "bowed-string",
  "wind",
  "percussion",
  "keyboard",
  "electronic",
  "voice",
];
const TUNINGS: TuningSystem[] = [
  "12-tet",
  "ethiopian-tizita",
  "ethiopian-bati",
  "arabic-maqam",
  "just-intonation",
  "pentatonic",
  "gamelan",
  "custom",
];
const REGIONS: CatalogRegion[] = [
  "East Africa",
  "West Africa",
  "North Africa / Middle East",
  "South Asia",
  "East Asia",
  "Europe",
  "Americas",
  "Electronic",
];

interface Props {
  onAdd: (instrument: Omit<CatalogInstrument, "id"> & { id: string }) => void;
  onClose: () => void;
}

export function AddInstrumentForm({ onAdd, onClose }: Props) {
  const [name, setName] = useState("");
  const [region, setRegion] = useState<CatalogRegion>("East Africa");
  const [country, setCountry] = useState("");
  const [family, setFamily] = useState<InstrumentFamily>("plucked-string");
  const [tuning, setTuning] = useState<TuningSystem>("12-tet");
  const [description, setDescription] = useState("");
  const [tags, setTags] = useState("");
  const [status, setStatus] = useState("");
  const fileRef = useRef<HTMLInputElement>(null);

  const inputStyle = {
    width: "100%",
    padding: "6px 10px",
    boxSizing: "border-box" as const,
    background: "#080e18",
    border: "1px solid #1a2a3a",
    borderRadius: 5,
    color: "#e2e8f0",
    fontSize: 12,
    fontFamily: "'Inter', system-ui, sans-serif",
  };

  const labelStyle = {
    fontSize: 11,
    color: "#475569",
    display: "block" as const,
    marginBottom: 3,
  };

  const handleSubmit = () => {
    if (!name.trim()) {
      setStatus("Name is required");
      return;
    }
    if (!country.trim()) {
      setStatus("Country is required");
      return;
    }
    if (!description.trim()) {
      setStatus("Description is required");
      return;
    }

    const id =
      name
        .toLowerCase()
        .replace(/\s+/g, "-")
        .replace(/[^a-z0-9-]/g, "") +
      "-" +
      Date.now();
    const tagList = tags
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);

    onAdd({
      id,
      name,
      region,
      country,
      family,
      description,
      tuning,
      source_family:
        family === "bowed-string"
          ? "bowed"
          : family === "wind"
            ? "blown"
            : family === "percussion"
              ? "struck"
              : family === "electronic"
                ? "electronic"
                : "plucked",
      tags: tagList,
      timbre_profile: {
        brightness: 0.5,
        warmth: 0.5,
        attack_character: "medium",
        sustain_character: "medium",
        spectral_centroid_hz: 1000,
        spectral_rolloff: 0.6,
        harmonic_richness: 0.6,
        formant_peaks: [400, 1200, 2800],
        envelope: { attack: 0.02, decay: 0.1, sustain: 0.7, release: 0.4 },
      },
    });
    onClose();
  };

  return (
    <div
      style={{
        background: "#0a1520",
        border: "1px solid #1a2a3a",
        borderRadius: 10,
        padding: 20,
        maxWidth: 480,
        width: "100%",
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 16,
        }}
      >
        <span style={{ fontSize: 14, fontWeight: 600, color: "#e2e8f0" }}>
          Add Custom Instrument
        </span>
        <button
          onClick={onClose}
          style={{
            background: "none",
            border: "none",
            color: "#475569",
            cursor: "pointer",
            fontSize: 16,
          }}
        >
          ✕
        </button>
      </div>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 10,
          marginBottom: 10,
        }}
      >
        <div>
          <label style={labelStyle}>Name *</label>
          <input
            style={inputStyle}
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. Krar"
          />
        </div>
        <div>
          <label style={labelStyle}>Country *</label>
          <input
            style={inputStyle}
            value={country}
            onChange={(e) => setCountry(e.target.value)}
            placeholder="e.g. Ethiopia"
          />
        </div>
        <div>
          <label style={labelStyle}>Region</label>
          <select
            style={inputStyle}
            value={region}
            onChange={(e) => setRegion(e.target.value as CatalogRegion)}
          >
            {REGIONS.map((r) => (
              <option key={r} value={r}>
                {r}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label style={labelStyle}>Family</label>
          <select
            style={inputStyle}
            value={family}
            onChange={(e) => setFamily(e.target.value as InstrumentFamily)}
          >
            {FAMILIES.map((f) => (
              <option key={f} value={f}>
                {f}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label style={labelStyle}>Tuning</label>
          <select
            style={inputStyle}
            value={tuning}
            onChange={(e) => setTuning(e.target.value as TuningSystem)}
          >
            {TUNINGS.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label style={labelStyle}>Tags (comma-separated)</label>
          <input
            style={inputStyle}
            value={tags}
            onChange={(e) => setTags(e.target.value)}
            placeholder="e.g. lyre, ethiopian"
          />
        </div>
      </div>

      <div style={{ marginBottom: 10 }}>
        <label style={labelStyle}>Description *</label>
        <textarea
          style={{ ...inputStyle, resize: "vertical", minHeight: 60 }}
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Describe the instrument, its origin, and its sound..."
        />
      </div>

      <div style={{ marginBottom: 14 }}>
        <label style={labelStyle}>
          Reference Audio (optional — improves timbre profile)
        </label>
        <input
          ref={fileRef}
          type="file"
          accept="audio/*"
          style={{ display: "none" }}
        />
        <button
          onClick={() => fileRef.current?.click()}
          style={{
            ...inputStyle,
            cursor: "pointer",
            textAlign: "left",
            color: "#475569",
          }}
        >
          📎 Upload reference recording (WAV/MP3)
        </button>
      </div>

      {status && (
        <div style={{ fontSize: 11, color: "#ef5350", marginBottom: 10 }}>
          {status}
        </div>
      )}

      <div style={{ display: "flex", gap: 8 }}>
        <button
          onClick={handleSubmit}
          style={{
            flex: 1,
            padding: "8px 0",
            background: "rgba(56,189,248,0.1)",
            border: "1px solid rgba(56,189,248,0.3)",
            borderRadius: 5,
            color: "#38bdf8",
            fontSize: 12,
            cursor: "pointer",
            fontWeight: 600,
          }}
        >
          Save to My Catalog
        </button>
        <button
          onClick={onClose}
          style={{
            padding: "8px 16px",
            background: "transparent",
            border: "1px solid #1a2a3a",
            borderRadius: 5,
            color: "#475569",
            fontSize: 12,
            cursor: "pointer",
          }}
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
