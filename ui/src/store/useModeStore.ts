/**
 * Aether Studio v2.0 - Mode Store
 * Manages the current studio mode (Explore, Create, Arrange, Perform)
 */

import { create } from "zustand";
import type { StudioMode } from "../types/modes";

interface ModeState {
  currentMode: StudioMode;
  setMode: (mode: StudioMode) => void;
  previousMode: StudioMode | null;
}

export const useModeStore = create<ModeState>((set) => ({
  currentMode: "explore",
  previousMode: null,
  setMode: (mode) =>
    set((state) => ({
      currentMode: mode,
      previousMode: state.currentMode,
    })),
}));
