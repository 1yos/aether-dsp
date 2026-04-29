/**
 * Module Registry
 * Maps each StudioModuleType to its React component.
 */
import type { StudioModuleType } from "./types";
import type React from "react";

// Lazy imports to avoid circular deps and keep initial bundle small
import { MixerModule } from "./Mixer/MixerModule";
import { StepSequencerModule } from "./StepSequencer/StepSequencerModule";
import { PianoRollModule } from "./PianoRoll/PianoRollModule";
import { LooperModule } from "./Looper/LooperModule";
import { InstrumentRackModule } from "./InstrumentRack/InstrumentRackModule";
import { TimelineModule } from "./Timeline/TimelineModule";
import { InstrumentBrowser } from "../catalog/InstrumentBrowser";

/** Registry mapping module type → React component */
export const moduleRegistry: Record<StudioModuleType, React.FC> = {
  Timeline: TimelineModule,
  Mixer: MixerModule,
  PianoRoll: PianoRollModule,
  StepSequencer: StepSequencerModule,
  Looper: LooperModule,
  InstrumentRack: InstrumentRackModule,
  InstrumentBrowser: InstrumentBrowser,
};
