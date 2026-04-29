/**
 * Instrument Maker store — manages the instrument being built.
 */
import { create } from "zustand";
import {
  SamplerInstrument,
  SampleZone,
  TuningPreset,
  TuningTable,
  buildTuning,
  newInstrument,
} from "../types";

interface InstrumentStore {
  instrument: SamplerInstrument;
  selectedZoneId: string | null;
  previewNote: number | null;
  isDirty: boolean;

  // Instrument metadata
  setName: (name: string) => void;
  setOrigin: (origin: string) => void;
  setDescription: (desc: string) => void;
  setAuthor: (author: string) => void;
  setTuning: (preset: TuningPreset) => void;
  setCustomTuning: (frequencies: number[]) => void;
  setEnvelope: (
    attack: number,
    decay: number,
    sustain: number,
    release: number,
  ) => void;
  setMaxVoices: (n: number) => void;

  // Zone management
  addZone: (zone: SampleZone) => void;
  updateZone: (id: string, updates: Partial<SampleZone>) => void;
  removeZone: (id: string) => void;
  selectZone: (id: string | null) => void;
  setPreviewNote: (note: number | null) => void;

  // Persistence
  loadInstrument: (inst: SamplerInstrument) => void;
  resetInstrument: () => void;
  exportJson: () => string;
}

export const useInstrumentStore = create<InstrumentStore>((set, get) => ({
  instrument: newInstrument(),
  selectedZoneId: null,
  previewNote: null,
  isDirty: false,

  setName: (name) =>
    set((s) => ({ instrument: { ...s.instrument, name }, isDirty: true })),
  setOrigin: (origin) =>
    set((s) => ({ instrument: { ...s.instrument, origin }, isDirty: true })),
  setDescription: (description) =>
    set((s) => ({
      instrument: { ...s.instrument, description },
      isDirty: true,
    })),
  setAuthor: (author) =>
    set((s) => ({ instrument: { ...s.instrument, author }, isDirty: true })),

  setTuning: (preset) =>
    set((s) => ({
      instrument: { ...s.instrument, tuning: buildTuning(preset) },
      isDirty: true,
    })),

  setCustomTuning: (pitchClassFreqs) => {
    // Build a full 128-note TuningTable from 12 pitch-class frequencies (C–B).
    // For each MIDI note, determine its pitch class and octave relative to
    // the reference octave (notes 60–71 = C4–B4, octave index 5 in MIDI).
    // Scale the pitch-class frequency by 2^(octaveDiff).
    const REFERENCE_OCTAVE = 5; // MIDI notes 60–71 are C4–B4 (octave = floor(note/12) - 1 = 4, but index = floor(note/12) = 5)
    const frequencies: number[] = [];
    for (let note = 0; note < 128; note++) {
      const pitchClass = note % 12;
      const octaveIndex = Math.floor(note / 12);
      const octaveDiff = octaveIndex - REFERENCE_OCTAVE;
      frequencies.push(pitchClassFreqs[pitchClass] * Math.pow(2, octaveDiff));
    }
    const tuning: TuningTable = {
      name: "Custom",
      description: "Custom tuning",
      frequencies,
    };
    set((s) => ({ instrument: { ...s.instrument, tuning }, isDirty: true }));
  },

  setEnvelope: (attack, decay, sustain, release) =>
    set((s) => ({
      instrument: { ...s.instrument, attack, decay, sustain, release },
      isDirty: true,
    })),

  setMaxVoices: (max_voices) =>
    set((s) => ({
      instrument: { ...s.instrument, max_voices },
      isDirty: true,
    })),

  addZone: (zone) =>
    set((s) => ({
      instrument: {
        ...s.instrument,
        zones: [...s.instrument.zones, zone],
      },
      selectedZoneId: zone.id,
      isDirty: true,
    })),

  updateZone: (id, updates) =>
    set((s) => ({
      instrument: {
        ...s.instrument,
        zones: s.instrument.zones.map((z) =>
          z.id === id ? { ...z, ...updates } : z,
        ),
      },
      isDirty: true,
    })),

  removeZone: (id) =>
    set((s) => ({
      instrument: {
        ...s.instrument,
        zones: s.instrument.zones.filter((z) => z.id !== id),
      },
      selectedZoneId: s.selectedZoneId === id ? null : s.selectedZoneId,
      isDirty: true,
    })),

  selectZone: (selectedZoneId) => set({ selectedZoneId }),
  setPreviewNote: (previewNote) => set({ previewNote }),

  loadInstrument: (instrument) =>
    set({ instrument, selectedZoneId: null, isDirty: false }),

  resetInstrument: () =>
    set({ instrument: newInstrument(), selectedZoneId: null, isDirty: false }),

  exportJson: () => {
    const inst = get().instrument;
    // Strip UI-only fields before export
    const exportable = {
      ...inst,
      zones: inst.zones.map(
        ({ audioDataUrl, audioBuffer, fileName, ...z }) => z,
      ),
    };
    return JSON.stringify(exportable, null, 2);
  },
}));
