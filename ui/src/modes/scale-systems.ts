/**
 * Scale / tuning systems for piano roll highlighting.
 * Each system defines which pitch classes are in scale and their cent deviations.
 */

export interface ScaleSystem {
  id: string;
  name: string;
  region: string;
  pitchClasses: number[];
  centDeviations: number[];
  description: string;
  color: string;
}

export const SCALE_SYSTEMS: ScaleSystem[] = [
  {
    id: "chromatic",
    name: "Chromatic",
    region: "Universal",
    pitchClasses: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    centDeviations: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    description: "All 12 semitones. No scale highlighting.",
    color: "#64748b",
  },
  {
    id: "major",
    name: "Major (Ionian)",
    region: "Western",
    pitchClasses: [0, 2, 4, 5, 7, 9, 11],
    centDeviations: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    description: "Western major scale. C D E F G A B.",
    color: "#38bdf8",
  },
  {
    id: "minor",
    name: "Natural Minor",
    region: "Western",
    pitchClasses: [0, 2, 3, 5, 7, 8, 10],
    centDeviations: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    description: "Western natural minor scale.",
    color: "#818cf8",
  },
  {
    id: "ethiopian-tizita",
    name: "Ethiopian Tizita",
    region: "East Africa",
    pitchClasses: [0, 2, 4, 7, 9],
    centDeviations: [0, -50, 0, -30, 0, 0, -20, 0, -40, 0, -30, 0],
    description:
      "Ethiopian Tizita major pentatonic. Characteristic flat intervals.",
    color: "#d4a017",
  },
  {
    id: "ethiopian-bati",
    name: "Ethiopian Bati",
    region: "East Africa",
    pitchClasses: [0, 2, 3, 5, 7, 9],
    centDeviations: [0, 0, -20, 0, 0, 0, -30, 0, 0, -20, 0, 0],
    description: "Ethiopian Bati minor pentatonic variant.",
    color: "#f59e0b",
  },
  {
    id: "ethiopian-ambassel",
    name: "Ethiopian Ambassel",
    region: "East Africa",
    pitchClasses: [0, 1, 5, 7, 8],
    centDeviations: [0, -30, 0, 0, 0, 0, 0, 0, -40, 0, 0, 0],
    description: "Ethiopian Ambassel scale. Distinctive minor second interval.",
    color: "#ef4444",
  },
  {
    id: "arabic-maqam-rast",
    name: "Arabic Maqam Rast",
    region: "North Africa / Middle East",
    pitchClasses: [0, 2, 3, 5, 7, 9, 10],
    centDeviations: [0, 0, 0, -50, 0, 0, 0, 0, 0, 0, -50, 0],
    description: "Most common Arabic maqam. Quarter-tone flats on 3rd and 7th.",
    color: "#a78bfa",
  },
  {
    id: "arabic-maqam-bayati",
    name: "Arabic Maqam Bayati",
    region: "North Africa / Middle East",
    pitchClasses: [0, 1, 3, 5, 7, 8, 10],
    centDeviations: [0, -50, 0, -30, 0, 0, 0, 0, 0, 0, -50, 0],
    description: "Second most common Arabic maqam. Half-flat on 2nd degree.",
    color: "#c084fc",
  },
  {
    id: "arabic-maqam-hijaz",
    name: "Arabic Maqam Hijaz",
    region: "North Africa / Middle East",
    pitchClasses: [0, 1, 4, 5, 7, 8, 10],
    centDeviations: [0, -20, 0, 0, 0, 0, 0, 0, -20, 0, 0, 0],
    description: "Maqam Hijaz. Augmented second gives it a dramatic quality.",
    color: "#e879f9",
  },
  {
    id: "indian-raga-yaman",
    name: "Indian Raga Yaman",
    region: "South Asia",
    pitchClasses: [0, 2, 4, 6, 7, 9, 11],
    centDeviations: [0, 0, 3.9, 0, -13.7, 0, -9.8, 2.0, 0, -15.6, 0, -11.7],
    description: "Kalyan thaat. Raised 4th. Evening raga, just intonation.",
    color: "#f97316",
  },
  {
    id: "indian-raga-bhairav",
    name: "Indian Raga Bhairav",
    region: "South Asia",
    pitchClasses: [0, 1, 4, 5, 7, 8, 11],
    centDeviations: [0, -14, 0, 0, 0, 0, 0, 0, -14, 0, 0, 0],
    description:
      "Morning raga. Flat 2nd and 6th. Serious, devotional character.",
    color: "#fb923c",
  },
  {
    id: "gamelan-slendro",
    name: "Gamelan Slendro",
    region: "East Asia",
    pitchClasses: [0, 2, 5, 7, 10],
    centDeviations: [0, 0, -40, 0, -20, 0, 0, -30, 0, -10, 0, 0],
    description: "Javanese 5-tone scale. Approximately equal pentatonic.",
    color: "#34d399",
  },
  {
    id: "gamelan-pelog",
    name: "Gamelan Pelog",
    region: "East Asia",
    pitchClasses: [0, 1, 3, 6, 7, 8, 10],
    centDeviations: [0, -80, 0, -30, 0, 0, -60, 0, -20, 0, 0, -50],
    description: "Javanese 7-tone scale with characteristic unequal intervals.",
    color: "#10b981",
  },
  {
    id: "pentatonic-minor",
    name: "Pentatonic Minor",
    region: "Universal",
    pitchClasses: [0, 3, 5, 7, 10],
    centDeviations: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    description:
      "Universal minor pentatonic. Blues, rock, and many world traditions.",
    color: "#60a5fa",
  },
];

export function getScaleSystem(id: string): ScaleSystem {
  return SCALE_SYSTEMS.find((s) => s.id === id) ?? SCALE_SYSTEMS[0];
}

export function isInScale(pitchClass: number, scale: ScaleSystem): boolean {
  return scale.pitchClasses.includes(pitchClass % 12);
}

export function getCentDeviation(
  pitchClass: number,
  scale: ScaleSystem,
): number {
  return scale.centDeviations[pitchClass % 12] ?? 0;
}
