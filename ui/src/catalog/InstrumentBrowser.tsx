import { useState, useRef, useCallback } from "react";
import { useCatalog } from "./useCatalog";
import { InstrumentCard } from "./InstrumentCard";
import { AddInstrumentForm } from "./AddInstrumentForm";
import { generateSyntheticSource } from "./syntheticSource";
import { applyTimbreProfile } from "./timbreTransfer";
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

export function InstrumentBrowser() {
  const {
    instruments,
    regions,
    search,
    filterByRegion,
    addCustom,
    removeCustom,
  } = useCatalog();
  const [activeRegion, setActiveRegion] = useState<string>("All");
  const [query, setQuery] = useState("");
  const [playingId, setPlayingId] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [applyTarget, setApplyTarget] = useState<CatalogInstrument | null>(
    null,
  );
  const [applyStatus, setApplyStatus] = useState("");
  const audioCtxRef = useRef<AudioContext | null>(null);
  const sourceNodeRef = useRef<AudioBufferSourceNode | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  function getAudioCtx(): AudioContext {
    if (!audioCtxRef.current) audioCtxRef.current = new AudioContext();
    return audioCtxRef.current;
  }

  const visibleInstruments = query.trim()
    ? search(query)
    : activeRegion === "All"
      ? instruments
      : filterByRegion(activeRegion);

  const stopCurrent = useCallback(() => {
    sourceNodeRef.current?.stop();
    sourceNodeRef.current = null;
    setPlayingId(null);
  }, []);

  const handlePreview = useCallback(
    async (instrument: CatalogInstrument) => {
      if (playingId === instrument.id) {
        stopCurrent();
        return;
      }
      stopCurrent();

      try {
        const ctx = getAudioCtx();
        if (ctx.state === "suspended") await ctx.resume();
        setPlayingId(instrument.id);

        const source = await generateSyntheticSource(
          instrument.source_family,
          ctx,
          2.5,
        );
        const processed = await applyTimbreProfile(
          source,
          instrument.timbre_profile,
          ctx,
        );

        const node = ctx.createBufferSource();
        node.buffer = processed;
        node.connect(ctx.destination);
        node.onended = () => setPlayingId(null);
        node.start();
        sourceNodeRef.current = node;
      } catch (e) {
        console.error("Preview failed:", e);
        setPlayingId(null);
      }
    },
    [playingId, stopCurrent],
  );

  const handleApply = useCallback((instrument: CatalogInstrument) => {
    setApplyTarget(instrument);
    setApplyStatus("");
    fileRef.current?.click();
  }, []);

  const handleFileSelected = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file || !applyTarget) return;
      e.target.value = "";

      setApplyStatus("Loading audio...");
      try {
        const ctx = getAudioCtx();
        if (ctx.state === "suspended") await ctx.resume();

        const arrayBuffer = await file.arrayBuffer();
        const decoded = await ctx.decodeAudioData(arrayBuffer);

        setApplyStatus("Applying timbre...");
        const processed = await applyTimbreProfile(
          decoded,
          applyTarget.timbre_profile,
          ctx,
        );

        setApplyStatus("Playing result...");
        stopCurrent();
        const node = ctx.createBufferSource();
        node.buffer = processed;
        node.connect(ctx.destination);
        node.onended = () => setApplyStatus("");
        node.start();
        sourceNodeRef.current = node;
        setPlayingId(applyTarget.id);
      } catch (err) {
        setApplyStatus(
          `Error: ${err instanceof Error ? err.message : String(err)}`,
        );
      }
    },
    [applyTarget, stopCurrent],
  );

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: "#060e18",
        fontFamily: "'Inter', system-ui, sans-serif",
        color: "#e2e8f0",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "10px 14px",
          borderBottom: "1px solid #0f1e2e",
          display: "flex",
          alignItems: "center",
          gap: 10,
          flexShrink: 0,
        }}
      >
        <span style={{ fontSize: 13, fontWeight: 700, color: "#38bdf8" }}>
          🌍 Instrument Catalog
        </span>
        <span style={{ fontSize: 10, color: "#334155" }}>
          {instruments.length} instruments
        </span>
        <div style={{ flex: 1 }} />
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search instruments..."
          style={{
            padding: "5px 10px",
            background: "#0a1520",
            border: "1px solid #1a2a3a",
            borderRadius: 5,
            color: "#e2e8f0",
            fontSize: 11,
            width: 180,
          }}
        />
        <button
          onClick={() => setShowAddForm(true)}
          style={{
            padding: "5px 12px",
            background: "rgba(56,189,248,0.1)",
            border: "1px solid rgba(56,189,248,0.3)",
            borderRadius: 5,
            color: "#38bdf8",
            fontSize: 11,
            cursor: "pointer",
            whiteSpace: "nowrap",
          }}
        >
          + Add Instrument
        </button>
      </div>

      {/* Region tabs */}
      {!query.trim() && (
        <div
          style={{
            display: "flex",
            gap: 2,
            padding: "6px 14px",
            borderBottom: "1px solid #0f1e2e",
            overflowX: "auto",
            flexShrink: 0,
          }}
        >
          {["All", ...regions].map((r) => {
            const active = activeRegion === r;
            const color =
              r === "All" ? "#38bdf8" : (REGION_COLORS[r] ?? "#38bdf8");
            return (
              <button
                key={r}
                onClick={() => setActiveRegion(r)}
                style={{
                  padding: "4px 10px",
                  borderRadius: 4,
                  whiteSpace: "nowrap",
                  background: active ? color + "18" : "transparent",
                  border: `1px solid ${active ? color + "50" : "transparent"}`,
                  color: active ? color : "#475569",
                  fontSize: 10,
                  cursor: "pointer",
                  fontWeight: active ? 600 : 400,
                  transition: "all 0.1s",
                }}
              >
                {r}
              </button>
            );
          })}
        </div>
      )}

      {/* Apply status */}
      {applyStatus && (
        <div
          style={{
            padding: "6px 14px",
            background: "rgba(56,189,248,0.08)",
            fontSize: 11,
            color: "#38bdf8",
          }}
        >
          {applyStatus}
        </div>
      )}

      {/* Instrument grid */}
      <div
        style={{
          flex: 1,
          overflowY: "auto",
          padding: 12,
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))",
          gap: 8,
          alignContent: "start",
        }}
      >
        {visibleInstruments.length === 0 ? (
          <div
            style={{
              gridColumn: "1/-1",
              textAlign: "center",
              color: "#334155",
              padding: 40,
              fontSize: 13,
            }}
          >
            No instruments found for "{query}"
          </div>
        ) : (
          visibleInstruments.map((inst) => (
            <InstrumentCard
              key={inst.id}
              instrument={inst}
              onPreview={() => handlePreview(inst)}
              onApply={() => handleApply(inst)}
              isPlaying={playingId === inst.id}
              isCustom={"is_custom" in inst}
              onRemove={
                "is_custom" in inst ? () => removeCustom(inst.id) : undefined
              }
            />
          ))
        )}
      </div>

      {/* Add form overlay */}
      {showAddForm && (
        <div
          style={{
            position: "absolute",
            inset: 0,
            background: "rgba(6,14,24,0.85)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 200,
            padding: 20,
          }}
        >
          <AddInstrumentForm
            onAdd={(inst) => {
              addCustom(inst as any);
              setShowAddForm(false);
            }}
            onClose={() => setShowAddForm(false)}
          />
        </div>
      )}

      {/* Hidden file input for "Apply to Source" */}
      <input
        ref={fileRef}
        type="file"
        accept="audio/*"
        style={{ display: "none" }}
        onChange={handleFileSelected}
      />
    </div>
  );
}
