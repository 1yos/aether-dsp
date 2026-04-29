/**
 * InstrumentMeta — instrument name, origin, description, tuning, ADSR.
 */
import React from "react";
import { useInstrumentStore } from "../store/instrumentStore";
import { TuningPreset, MIDI_NOTE_NAMES } from "../types";

const PITCH_CLASS_NAMES = MIDI_NOTE_NAMES; // C, C#, D, D#, E, F, F#, G, G#, A, A#, B

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

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div style={{ marginBottom: 8 }}>
      <label style={labelStyle}>{label}</label>
      {children}
    </div>
  );
}

function Slider({
  label,
  value,
  min,
  max,
  step,
  onChange,
  format,
  title,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (v: number) => void;
  format?: (v: number) => string;
  title?: string;
}) {
  return (
    <div style={{ marginBottom: 6 }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          marginBottom: 2,
        }}
      >
        <span style={{ fontSize: 10, color: "#7a9ab5" }}>{label}</span>
        <span style={{ fontSize: 10, color: "#4fc3f7" }}>
          {format ? format(value) : value.toFixed(3)}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        style={{ width: "100%", accentColor: "#4fc3f7" }}
        title={title}
      />
    </div>
  );
}

const TUNING_PRESETS: TuningPreset[] = [
  "12-TET",
  "Ethiopian Tizita",
  "Just Intonation",
  "Custom",
];

export function InstrumentMeta() {
  const {
    instrument,
    setName,
    setOrigin,
    setDescription,
    setAuthor,
    setTuning,
    setCustomTuning,
    setEnvelope,
    setMaxVoices,
  } = useInstrumentStore();

  // Pre-fill custom tuning inputs from notes 60–71 (C4–B4)
  const customFreqs = Array.from(
    { length: 12 },
    (_, i) => instrument.tuning.frequencies[60 + i],
  );

  const isCustom = instrument.tuning.name === "Custom";

  return (
    <div style={{ padding: 12 }}>
      <Field label="Instrument Name">
        <input
          style={inputStyle}
          value={instrument.name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. Ethiopian Krar"
          title="The display name of this instrument"
        />
      </Field>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
        <Field label="Origin / Culture">
          <input
            style={inputStyle}
            value={instrument.origin}
            onChange={(e) => setOrigin(e.target.value)}
            placeholder="e.g. Ethiopian"
            title="The cultural or geographic origin of this instrument"
          />
        </Field>
        <Field label="Author">
          <input
            style={inputStyle}
            value={instrument.author}
            onChange={(e) => setAuthor(e.target.value)}
            placeholder="Your name"
            title="The creator or sampler of this instrument"
          />
        </Field>
      </div>

      <Field label="Description">
        <textarea
          style={{ ...inputStyle, resize: "vertical", minHeight: 48 }}
          value={instrument.description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Describe the instrument..."
          title="A short description of the instrument, its character, and playing technique"
        />
      </Field>

      <Field label="Tuning System">
        <select
          style={{ ...inputStyle, cursor: "pointer" }}
          value={instrument.tuning.name}
          onChange={(e) => setTuning(e.target.value as TuningPreset)}
          title="Select the tuning system used for pitch mapping across all 128 MIDI notes"
        >
          {TUNING_PRESETS.map((p) => (
            <option key={p} value={p}>
              {p}
            </option>
          ))}
        </select>
        <div style={{ fontSize: 10, color: "#4a6a8a", marginTop: 3 }}>
          {instrument.tuning.description}
        </div>
      </Field>

      {isCustom && (
        <div style={{ marginBottom: 8 }}>
          <div style={{ fontSize: 11, color: "#7a9ab5", marginBottom: 6 }}>
            Custom Tuning — C4 to B4 (Hz)
          </div>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: "4px 8px",
            }}
          >
            {PITCH_CLASS_NAMES.map((name, i) => (
              <div
                key={name}
                style={{ display: "flex", alignItems: "center", gap: 4 }}
              >
                <span
                  style={{
                    fontSize: 10,
                    color: "#7a9ab5",
                    width: 22,
                    flexShrink: 0,
                  }}
                >
                  {name}
                </span>
                <input
                  type="number"
                  style={{
                    ...inputStyle,
                    padding: "2px 4px",
                    fontSize: 11,
                    width: "100%",
                  }}
                  value={parseFloat(customFreqs[i].toFixed(4))}
                  step={0.01}
                  min={1}
                  max={20000}
                  title={`Frequency for pitch class ${name}4 in Hz. All octaves of ${name} will be scaled from this value.`}
                  onChange={(e) => {
                    const updated = [...customFreqs];
                    updated[i] = parseFloat(e.target.value) || customFreqs[i];
                    setCustomTuning(updated);
                  }}
                />
              </div>
            ))}
          </div>
        </div>
      )}

      <div
        style={{
          borderTop: "1px solid #1a2a3a",
          paddingTop: 10,
          marginTop: 4,
        }}
      >
        <div style={{ fontSize: 11, color: "#7a9ab5", marginBottom: 8 }}>
          ADSR Envelope
        </div>
        <Slider
          label="Attack"
          value={instrument.attack}
          min={0.001}
          max={5}
          step={0.001}
          onChange={(v) =>
            setEnvelope(
              v,
              instrument.decay,
              instrument.sustain,
              instrument.release,
            )
          }
          format={(v) => `${(v * 1000).toFixed(0)}ms`}
          title="Time for the sound to reach full volume after a note-on event"
        />
        <Slider
          label="Decay"
          value={instrument.decay}
          min={0.001}
          max={5}
          step={0.001}
          onChange={(v) =>
            setEnvelope(
              instrument.attack,
              v,
              instrument.sustain,
              instrument.release,
            )
          }
          format={(v) => `${(v * 1000).toFixed(0)}ms`}
          title="Time for the sound to fall from peak to the sustain level"
        />
        <Slider
          label="Sustain"
          value={instrument.sustain}
          min={0}
          max={1}
          step={0.01}
          onChange={(v) =>
            setEnvelope(
              instrument.attack,
              instrument.decay,
              v,
              instrument.release,
            )
          }
          format={(v) => `${(v * 100).toFixed(0)}%`}
          title="Volume level held while a note is pressed (0% = silent, 100% = full)"
        />
        <Slider
          label="Release"
          value={instrument.release}
          min={0.001}
          max={10}
          step={0.001}
          onChange={(v) =>
            setEnvelope(
              instrument.attack,
              instrument.decay,
              instrument.sustain,
              v,
            )
          }
          format={(v) => `${(v * 1000).toFixed(0)}ms`}
          title="Time for the sound to fade to silence after a note-off event"
        />
      </div>

      <div style={{ marginTop: 8 }}>
        <div style={{ fontSize: 11, color: "#7a9ab5", marginBottom: 4 }}>
          Max Polyphony
        </div>
        <input
          type="number"
          style={{ ...inputStyle, width: 80 }}
          min={1}
          max={64}
          value={instrument.max_voices}
          onChange={(e) => setMaxVoices(parseInt(e.target.value) || 16)}
          title="Maximum number of simultaneous voices (notes) this instrument can play"
        />
      </div>
    </div>
  );
}
