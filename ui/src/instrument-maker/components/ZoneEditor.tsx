/**
 * ZoneEditor — edit a single sample zone's properties.
 */
import React from "react";
import { useInstrumentStore } from "../store/instrumentStore";
import { ArticulationType, midiNoteName } from "../types";

const labelStyle: React.CSSProperties = {
  fontSize: 11,
  color: "#7a9ab5",
  marginBottom: 2,
  display: "block",
};

const inputStyle: React.CSSProperties = {
  background: "#0a1520",
  border: "1px solid #1a2a3a",
  borderRadius: 4,
  color: "#e0e8f0",
  padding: "4px 8px",
  fontSize: 12,
  width: "100%",
  boxSizing: "border-box",
};

const selectStyle: React.CSSProperties = {
  ...inputStyle,
  cursor: "pointer",
};

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div style={{ marginBottom: 10 }}>
      <label style={labelStyle}>{label}</label>
      {children}
    </div>
  );
}

function NoteSelect({
  value,
  onChange,
}: {
  value: number;
  onChange: (n: number) => void;
}) {
  return (
    <select
      style={selectStyle}
      value={value}
      onChange={(e) => onChange(parseInt(e.target.value))}
    >
      {Array.from({ length: 128 }, (_, i) => (
        <option key={i} value={i}>
          {i} — {midiNoteName(i)}
        </option>
      ))}
    </select>
  );
}

export function ZoneEditor() {
  const { instrument, selectedZoneId, updateZone, removeZone } =
    useInstrumentStore();
  const zone = instrument.zones.find((z) => z.id === selectedZoneId);

  if (!zone) {
    return (
      <div
        style={{
          padding: 16,
          color: "#4a6a8a",
          fontSize: 12,
          textAlign: "center",
        }}
      >
        Select a zone on the keyboard or in the zone list to edit it.
      </div>
    );
  }

  const update = (updates: Parameters<typeof updateZone>[1]) =>
    updateZone(zone.id, updates);

  const artType =
    typeof zone.articulation === "object" && "type" in zone.articulation
      ? zone.articulation.type
      : "OneShot";

  const setArticulation = (type: string) => {
    let art: ArticulationType;
    if (type === "SustainLoop") {
      art = { type: "SustainLoop", loop_start: 0, loop_end: 44100 };
    } else if (type === "SustainRelease") {
      art = { type: "SustainRelease" };
    } else {
      art = { type: "OneShot" };
    }
    update({ articulation: art });
  };

  return (
    <div style={{ padding: 12 }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 12,
        }}
      >
        <span style={{ color: "#e0e8f0", fontSize: 13, fontWeight: 600 }}>
          Zone: {zone.fileName ?? zone.file_path.split("/").pop()}
        </span>
        <button
          onClick={() => removeZone(zone.id)}
          style={{
            background: "#3a1a1a",
            border: "1px solid #5a2a2a",
            borderRadius: 4,
            color: "#ef9a9a",
            padding: "3px 8px",
            fontSize: 11,
            cursor: "pointer",
          }}
        >
          Remove
        </button>
      </div>

      <Field label="Root Note (recorded pitch)">
        <NoteSelect
          value={zone.root_note}
          onChange={(n) => update({ root_note: n })}
        />
      </Field>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
        <Field label="Note Low">
          <NoteSelect
            value={zone.note_low}
            onChange={(n) => update({ note_low: n })}
          />
        </Field>
        <Field label="Note High">
          <NoteSelect
            value={zone.note_high}
            onChange={(n) => update({ note_high: n })}
          />
        </Field>
        <Field label="Velocity Low (0–127)">
          <input
            type="number"
            style={inputStyle}
            min={0}
            max={127}
            value={zone.velocity_low}
            onChange={(e) =>
              update({ velocity_low: parseInt(e.target.value) || 0 })
            }
          />
        </Field>
        <Field label="Velocity High (0–127)">
          <input
            type="number"
            style={inputStyle}
            min={0}
            max={127}
            value={zone.velocity_high}
            onChange={(e) =>
              update({ velocity_high: parseInt(e.target.value) || 127 })
            }
          />
        </Field>
      </div>

      <Field label="Articulation">
        <select
          style={selectStyle}
          value={artType}
          onChange={(e) => setArticulation(e.target.value)}
        >
          <option value="OneShot">One Shot (pluck, staccato)</option>
          <option value="SustainLoop">
            Sustain Loop (hold while key pressed)
          </option>
          <option value="SustainRelease">Sustain + Release tail</option>
        </select>
      </Field>

      {artType === "SustainLoop" &&
        typeof zone.articulation === "object" &&
        "loop_start" in zone.articulation && (
          <div
            style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}
          >
            <Field label="Loop Start (frames)">
              <input
                type="number"
                style={inputStyle}
                min={0}
                value={
                  (
                    zone.articulation as {
                      type: "SustainLoop";
                      loop_start: number;
                      loop_end: number;
                    }
                  ).loop_start
                }
                onChange={(e) =>
                  update({
                    articulation: {
                      ...zone.articulation,
                      loop_start: parseInt(e.target.value) || 0,
                    } as ArticulationType,
                  })
                }
              />
            </Field>
            <Field label="Loop End (frames)">
              <input
                type="number"
                style={inputStyle}
                min={0}
                value={
                  (
                    zone.articulation as {
                      type: "SustainLoop";
                      loop_start: number;
                      loop_end: number;
                    }
                  ).loop_end
                }
                onChange={(e) =>
                  update({
                    articulation: {
                      ...zone.articulation,
                      loop_end: parseInt(e.target.value) || 44100,
                    } as ArticulationType,
                  })
                }
              />
            </Field>
          </div>
        )}

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
        <Field label="Volume (dB)">
          <input
            type="number"
            style={inputStyle}
            step={0.5}
            value={zone.volume_db}
            onChange={(e) =>
              update({ volume_db: parseFloat(e.target.value) || 0 })
            }
          />
        </Field>
        <Field label="Tune (cents)">
          <input
            type="number"
            style={inputStyle}
            step={1}
            min={-100}
            max={100}
            value={zone.tune_cents}
            onChange={(e) =>
              update({ tune_cents: parseFloat(e.target.value) || 0 })
            }
          />
        </Field>
      </div>
    </div>
  );
}
