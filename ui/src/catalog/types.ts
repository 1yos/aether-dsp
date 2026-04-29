/**
 * Aether Instrument Catalog — TypeScript types
 */

export interface TimbreProfile {
  brightness: number; // 0-1: spectral brightness
  warmth: number; // 0-1: low-frequency richness
  attack_character: "sharp" | "soft" | "medium";
  sustain_character: "short" | "medium" | "long" | "looped";
  spectral_centroid_hz: number;
  spectral_rolloff: number; // 0-1
  harmonic_richness: number; // 0-1
  formant_peaks: number[]; // Hz
  envelope: { attack: number; decay: number; sustain: number; release: number };
}

export type InstrumentFamily =
  | "plucked-string"
  | "bowed-string"
  | "wind"
  | "percussion"
  | "keyboard"
  | "electronic"
  | "voice";

export type SourceFamily =
  | "plucked"
  | "bowed"
  | "blown"
  | "struck"
  | "electronic";

export type TuningSystem =
  | "12-tet"
  | "ethiopian-tizita"
  | "ethiopian-bati"
  | "arabic-maqam"
  | "just-intonation"
  | "pentatonic"
  | "gamelan"
  | "custom";

export type CatalogRegion =
  | "East Africa"
  | "West Africa"
  | "North Africa / Middle East"
  | "South Asia"
  | "East Asia"
  | "Europe"
  | "Americas"
  | "Electronic";

export interface CatalogInstrument {
  id: string;
  name: string;
  region: CatalogRegion;
  country: string;
  family: InstrumentFamily;
  description: string;
  tuning: TuningSystem;
  source_family: SourceFamily;
  tags: string[];
  timbre_profile: TimbreProfile;
}

export interface CustomCatalogEntry extends CatalogInstrument {
  is_custom: true;
  added_at: string; // ISO date
}

// Type alias for compatibility
export type Instrument = CatalogInstrument & {
  flag?: string; // Emoji flag for display
};

// Region type alias for mode switcher
export type Region =
  | "east-africa"
  | "west-africa"
  | "middle-east"
  | "south-asia"
  | "east-asia"
  | "europe"
  | "americas"
  | "electronic";
