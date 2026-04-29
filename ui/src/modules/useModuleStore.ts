/**
 * Module System — Zustand layout store
 * Manages which modules are open, their positions, sizes, and internal state.
 */
import { create } from "zustand";
import type { ModuleInstance, ModuleLayout, StudioModuleType } from "./types";

/** Cascading offset so newly added modules don't stack exactly on top of each other */
const CASCADE_STEP = 30;
const DEFAULT_WIDTH = 400;
const DEFAULT_HEIGHT = 300;
const BASE_X = 80;
const BASE_Y = 80;

function generateId(): string {
  return `module-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
}

interface ModuleStore {
  layout: ModuleLayout;

  /** Add a new module of the given type with default size and cascading position */
  addModule: (type: StudioModuleType) => void;

  /** Remove a module by id */
  removeModule: (id: string) => void;

  /** Merge partial state into a module's state record */
  updateModuleState: (id: string, state: Record<string, unknown>) => void;

  /** Update a module's dimensions */
  resizeModule: (id: string, width: number, height: number) => void;

  /** Update a module's position */
  moveModule: (id: string, x: number, y: number) => void;

  /** Serialize the current layout to a JSON string */
  serializeLayout: () => string;

  /** Replace the current layout from a JSON string */
  deserializeLayout: (json: string) => void;
}

export const useModuleStore = create<ModuleStore>((set, get) => ({
  layout: { modules: [] },

  addModule: (type) => {
    const modules = get().layout.modules;
    const count = modules.length;
    const offset = (count % 10) * CASCADE_STEP;

    const newModule: ModuleInstance = {
      id: generateId(),
      type,
      width: DEFAULT_WIDTH,
      height: DEFAULT_HEIGHT,
      x: BASE_X + offset,
      y: BASE_Y + offset,
      state: {},
    };

    set((s) => ({
      layout: { modules: [...s.layout.modules, newModule] },
    }));
  },

  removeModule: (id) => {
    set((s) => ({
      layout: {
        modules: s.layout.modules.filter((m) => m.id !== id),
      },
    }));
  },

  updateModuleState: (id, state) => {
    set((s) => ({
      layout: {
        modules: s.layout.modules.map((m) =>
          m.id === id ? { ...m, state: { ...m.state, ...state } } : m,
        ),
      },
    }));
  },

  resizeModule: (id, width, height) => {
    set((s) => ({
      layout: {
        modules: s.layout.modules.map((m) =>
          m.id === id ? { ...m, width, height } : m,
        ),
      },
    }));
  },

  moveModule: (id, x, y) => {
    set((s) => ({
      layout: {
        modules: s.layout.modules.map((m) =>
          m.id === id ? { ...m, x, y } : m,
        ),
      },
    }));
  },

  serializeLayout: () => {
    return JSON.stringify(get().layout);
  },

  deserializeLayout: (json) => {
    try {
      const parsed = JSON.parse(json) as ModuleLayout;
      set({ layout: parsed });
    } catch (e) {
      console.error("[ModuleStore] Failed to deserialize layout:", e);
    }
  },
}));
