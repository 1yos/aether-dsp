/**
 * useTransportStore — shared transport state for Timeline, Step Sequencer, Piano Roll.
 * Tracks BPM, play state, and current beat position.
 */
import { create } from "zustand";

interface TransportStore {
  bpm: number;
  isPlaying: boolean;
  currentBeat: number;

  setBpm: (bpm: number) => void;
  play: () => void;
  stop: () => void;
  advanceBeat: () => void;
}

export const useTransportStore = create<TransportStore>((set) => ({
  bpm: 120,
  isPlaying: false,
  currentBeat: 0,

  setBpm: (bpm) => set({ bpm }),
  play: () => set({ isPlaying: true }),
  stop: () => set({ isPlaying: false, currentBeat: 0 }),
  advanceBeat: () => set((s) => ({ currentBeat: s.currentBeat + 1 })),
}));
