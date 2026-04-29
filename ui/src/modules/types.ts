/**
 * Module System — shared types
 * Defines the core data structures for the studio module system.
 */

/** All available studio module types */
export type StudioModuleType =
  | "Timeline"
  | "Mixer"
  | "PianoRoll"
  | "StepSequencer"
  | "Looper"
  | "InstrumentRack"
  | "InstrumentBrowser";

/** A single open module instance with its layout and internal state */
export interface ModuleInstance {
  id: string;
  type: StudioModuleType;
  width: number;
  height: number;
  x: number;
  y: number;
  state: Record<string, unknown>;
}

/** The full layout — all currently open module instances */
export interface ModuleLayout {
  modules: ModuleInstance[];
}
