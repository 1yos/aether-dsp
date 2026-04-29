/**
 * Instrument Maker types — mirrors the Rust aether-sampler data model.
 */

export type ArticulationType =
  | { type: "OneShot" }
  | { type: "SustainLoop"; loop_start: number; loop_end: number }
  | { type: "SustainRelease" };

export interface SampleZone {
  id: string;
  file_path: string;
  root_note: number;
  note_low: number;
  note_high: number;
  velocity_low: number;
  velocity_high: number;
  articulation: ArticulationType;
  volume_db: number;
  tune_cents: number;
  release_file: string | null;
  // UI-only: loaded audio data URL for playback preview
  audioDataUrl?: string;
  audioBuffer?: AudioBuffer;
  fileName?: string;
}

export type TuningPreset =
  | "12-TET"
  | "Ethiopian Tizita"
  | "Just Intonation"
  | "Custom";

export interface TuningTable {
  name: string;
  description: string;
  frequencies: number[]; // 128 values
}

export interface SamplerInstrument {
  name: string;
  origin: string;
  description: string;
  author: string;
  tuning: TuningTable;
  zones: SampleZone[];
  attack: number;
  decay: number;
  sustain: number;
  release: number;
  max_voices: number;
}

// ── Tuning presets ────────────────────────────────────────────────────────────

function buildEqualTemperament(concertA = 440): number[] {
  const freqs: number[] = [];
  for (let note = 0; note < 128; note++) {
    freqs.push(concertA * Math.pow(2, (note - 69) / 12));
  }
  return freqs;
}

function applyOffsets(base: number[], offsets: number[]): number[] {
  return base.map((f, i) => f * Math.pow(2, offsets[i % 12] / 1200));
}

export function buildTuning(preset: TuningPreset, concertA = 440): TuningTable {
  const base = buildEqualTemperament(concertA);
  switch (preset) {
    case "12-TET":
      return {
        name: "12-TET",
        description: "Standard 12-tone equal temperament, A4=440Hz",
        frequencies: base,
      };
    case "Ethiopian Tizita": {
      const offsets = [0, -50, 0, -30, 0, 0, -20, 0, -40, 0, -30, 0];
      return {
        name: "Ethiopian Tizita",
        description: "Approximation of Ethiopian Tizita major pentatonic scale",
        frequencies: applyOffsets(base, offsets),
      };
    }
    case "Just Intonation": {
      const ratios = [
        1,
        16 / 15,
        9 / 8,
        6 / 5,
        5 / 4,
        4 / 3,
        45 / 32,
        3 / 2,
        8 / 5,
        5 / 3,
        9 / 5,
        15 / 8,
      ];
      const tetRatios = Array.from({ length: 12 }, (_, i) =>
        Math.pow(2, i / 12),
      );
      const offsets = ratios.map((r, i) => 1200 * Math.log2(r / tetRatios[i]));
      return {
        name: "Just Intonation",
        description: "Pure harmonic ratios — no beating on perfect intervals",
        frequencies: applyOffsets(base, offsets),
      };
    }
    case "Custom":
      return {
        name: "Custom",
        description: "Custom tuning",
        frequencies: base,
      };
  }
}

export const MIDI_NOTE_NAMES = [
  "C",
  "C#",
  "D",
  "D#",
  "E",
  "F",
  "F#",
  "G",
  "G#",
  "A",
  "A#",
  "B",
];

export function midiNoteName(note: number): string {
  const octave = Math.floor(note / 12) - 1;
  const name = MIDI_NOTE_NAMES[note % 12];
  return `${name}${octave}`;
}

export function newInstrument(): SamplerInstrument {
  return {
    name: "New Instrument",
    origin: "",
    description: "",
    author: "",
    tuning: buildTuning("12-TET"),
    zones: [],
    attack: 0.005,
    decay: 0.1,
    sustain: 0.8,
    release: 0.3,
    max_voices: 16,
  };
}
