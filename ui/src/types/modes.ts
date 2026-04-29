/**
 * Aether Studio v2.0 - Mode Types
 */

export type StudioMode = "explore" | "create" | "arrange" | "perform";

export interface ModeConfig {
  id: StudioMode;
  label: string;
  icon: string;
  description: string;
  shortcut: string;
}

export const STUDIO_MODES: Record<StudioMode, ModeConfig> = {
  explore: {
    id: "explore",
    label: "Explore",
    icon: "🌍",
    description: "Browse and try instruments",
    shortcut: "1",
  },
  create: {
    id: "create",
    label: "Create",
    icon: "🎛",
    description: "Build audio patches",
    shortcut: "2",
  },
  arrange: {
    id: "arrange",
    label: "Arrange",
    icon: "⏱",
    description: "Compose and arrange",
    shortcut: "3",
  },
  perform: {
    id: "perform",
    label: "Perform",
    icon: "🎮",
    description: "Live performance",
    shortcut: "4",
  },
};
