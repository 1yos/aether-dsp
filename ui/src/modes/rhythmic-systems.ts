/**
 * Non-Western rhythmic systems for the piano roll.
 * Each system defines beats per cycle, subdivision names, and accent patterns.
 */

export interface RhythmicSystem {
  id: string;
  name: string;
  region: string;
  beatsPerCycle: number;
  subdivisions: number;
  accentPattern: number[];
  beatNames?: string[];
  description: string;
}

export const RHYTHMIC_SYSTEMS: RhythmicSystem[] = [
  {
    id: "4-4",
    name: "4/4 Common Time",
    region: "Western",
    beatsPerCycle: 4,
    subdivisions: 4,
    accentPattern: [0],
    description: "Standard Western 4/4 time.",
  },
  {
    id: "3-4",
    name: "3/4 Waltz",
    region: "Western",
    beatsPerCycle: 3,
    subdivisions: 4,
    accentPattern: [0],
    description: "Waltz time. 3 beats per bar.",
  },
  {
    id: "6-8",
    name: "6/8 Compound",
    region: "Western / Ethiopian",
    beatsPerCycle: 6,
    subdivisions: 2,
    accentPattern: [0, 3],
    description: "Compound duple. Used in Ethiopian Tizita and Western 6/8.",
  },
  {
    id: "12-8",
    name: "12/8 Ethiopian",
    region: "East Africa",
    beatsPerCycle: 12,
    subdivisions: 2,
    accentPattern: [0, 3, 6, 9],
    description:
      "Ethiopian 12/8. Used in Tizita, Bati, and Ambassel. Four groups of three.",
  },
  {
    id: "teentaal",
    name: "Teentaal (16 beats)",
    region: "South Asia",
    beatsPerCycle: 16,
    subdivisions: 4,
    accentPattern: [0, 4, 8, 12],
    beatNames: [
      "Dha",
      "Dhin",
      "Dhin",
      "Dha",
      "Dha",
      "Dhin",
      "Dhin",
      "Dha",
      "Dha",
      "Tin",
      "Tin",
      "Ta",
      "Ta",
      "Dhin",
      "Dhin",
      "Dha",
    ],
    description: "Most common North Indian tala. 16 beats in 4 vibhags.",
  },
  {
    id: "rupak",
    name: "Rupak Tala (7 beats)",
    region: "South Asia",
    beatsPerCycle: 7,
    subdivisions: 4,
    accentPattern: [0, 3, 5],
    beatNames: ["Tin", "Tin", "Na", "Dhi", "Na", "Dhi", "Na"],
    description: "7-beat Indian tala in 3+2+2 grouping.",
  },
  {
    id: "jhaptaal",
    name: "Jhaptaal (10 beats)",
    region: "South Asia",
    beatsPerCycle: 10,
    subdivisions: 4,
    accentPattern: [0, 2, 5, 7],
    beatNames: [
      "Dhi",
      "Na",
      "Dhi",
      "Dhi",
      "Na",
      "Ti",
      "Na",
      "Dhi",
      "Dhi",
      "Na",
    ],
    description: "10-beat Indian tala in 2+3+2+3 grouping.",
  },
  {
    id: "maqsum",
    name: "Maqsum (Arabic)",
    region: "North Africa / Middle East",
    beatsPerCycle: 8,
    subdivisions: 2,
    accentPattern: [0, 4],
    beatNames: ["Dum", "—", "Tak", "—", "Dum", "Dum", "Tak", "—"],
    description: "Most common Arabic rhythm. 8 beats with Dum-Tak pattern.",
  },
  {
    id: "wahda",
    name: "Wahda (Arabic)",
    region: "North Africa / Middle East",
    beatsPerCycle: 4,
    subdivisions: 4,
    accentPattern: [0, 2],
    beatNames: ["Dum", "—", "Tak", "—"],
    description: "Simple 4-beat Arabic rhythm.",
  },
  {
    id: "samai",
    name: "Samai (Arabic/Turkish)",
    region: "North Africa / Middle East",
    beatsPerCycle: 10,
    subdivisions: 2,
    accentPattern: [0, 3, 6, 8],
    beatNames: ["Dum", "—", "—", "Tak", "—", "—", "Dum", "—", "Tak", "—"],
    description: "10-beat rhythm in 3+3+2+2 grouping.",
  },
  {
    id: "gamelan-4",
    name: "Gamelan Lancaran (4)",
    region: "East Asia",
    beatsPerCycle: 4,
    subdivisions: 4,
    accentPattern: [0, 2],
    description: "Short Javanese gamelan colotomic cycle.",
  },
  {
    id: "gamelan-8",
    name: "Gamelan Ketawang (8)",
    region: "East Asia",
    beatsPerCycle: 8,
    subdivisions: 4,
    accentPattern: [0, 4],
    description: "Medium Javanese gamelan cycle.",
  },
  {
    id: "gamelan-16",
    name: "Gamelan Ladrang (16)",
    region: "East Asia",
    beatsPerCycle: 16,
    subdivisions: 4,
    accentPattern: [0, 4, 8, 12],
    description: "Long Javanese gamelan cycle with full colotomic structure.",
  },
];

export function getRhythmicSystem(id: string): RhythmicSystem {
  return RHYTHMIC_SYSTEMS.find((r) => r.id === id) ?? RHYTHMIC_SYSTEMS[0];
}
