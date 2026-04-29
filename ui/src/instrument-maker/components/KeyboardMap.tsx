/**
 * KeyboardMap — visual piano keyboard showing zone assignments.
 */
import { useCallback } from "react";
import { useInstrumentStore } from "../store/instrumentStore";
import { midiNoteName, SampleZone } from "../types";

const ZONE_COLORS = [
  "#4fc3f7",
  "#ce93d8",
  "#ffb74d",
  "#80cbc4",
  "#a5d6a7",
  "#ef9a9a",
  "#fff176",
  "#f48fb1",
  "#b39ddb",
  "#80deea",
];

const BLACK_KEYS = new Set([1, 3, 6, 8, 10]);

function isBlack(note: number): boolean {
  return BLACK_KEYS.has(note % 12);
}

function buildLayout(startNote: number, endNote: number): number[] {
  const whites: number[] = [];
  for (let n = startNote; n <= endNote; n++) {
    if (!isBlack(n)) whites.push(n);
  }
  return whites;
}

interface KeyboardMapProps {
  startNote?: number;
  endNote?: number;
  onNoteClick?: (note: number) => void;
}

export function KeyboardMap({
  startNote = 21,
  endNote = 108,
  onNoteClick,
}: KeyboardMapProps) {
  const { instrument, selectedZoneId, selectZone, setPreviewNote } =
    useInstrumentStore();
  const whites = buildLayout(startNote, endNote);
  const whiteWidth = 18;
  const whiteHeight = 80;
  const blackWidth = 11;
  const blackHeight = 50;
  const totalWidth = whites.length * whiteWidth;

  const whiteX: Record<number, number> = {};
  whites.forEach((n, i) => {
    whiteX[n] = i * whiteWidth;
  });

  function noteX(note: number): number {
    if (!isBlack(note)) return whiteX[note] ?? 0;
    const below = note - 1;
    const bx = whiteX[below] ?? 0;
    return bx + whiteWidth - blackWidth / 2;
  }

  const zoneColorMap: Record<string, string> = {};
  instrument.zones.forEach((z, i) => {
    zoneColorMap[z.id] = ZONE_COLORS[i % ZONE_COLORS.length];
  });

  function zonesForNote(note: number): SampleZone[] {
    return instrument.zones.filter(
      (z) => note >= z.note_low && note <= z.note_high,
    );
  }

  const handleKeyClick = useCallback(
    (note: number) => {
      setPreviewNote(note);
      onNoteClick?.(note);
      const covering = zonesForNote(note);
      if (covering.length === 1) {
        selectZone(covering[0].id);
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [instrument.zones, onNoteClick, selectZone, setPreviewNote],
  );

  return (
    <div
      style={{
        overflowX: "auto",
        background: "#060c12",
        borderRadius: 8,
        padding: "8px 4px 4px",
        border: "1px solid #1a2a3a",
      }}
    >
      <svg
        width={totalWidth}
        height={whiteHeight + 24}
        style={{ display: "block" }}
      >
        {/* Zone range overlays */}
        {instrument.zones.map((zone, zi) => {
          const color = ZONE_COLORS[zi % ZONE_COLORS.length];
          const isSelected = zone.id === selectedZoneId;
          const x1 = noteX(zone.note_low);
          const x2 =
            noteX(zone.note_high) +
            (isBlack(zone.note_high) ? blackWidth : whiteWidth);
          return (
            <g
              key={zone.id}
              onClick={() => selectZone(zone.id)}
              style={{ cursor: "pointer" }}
            >
              <rect
                x={x1}
                y={0}
                width={Math.max(0, x2 - x1)}
                height={8}
                fill={color}
                opacity={isSelected ? 1 : 0.6}
                rx={2}
              />
              {isSelected && (
                <rect
                  x={x1}
                  y={0}
                  width={Math.max(0, x2 - x1)}
                  height={8}
                  fill="none"
                  stroke="white"
                  strokeWidth={1}
                  rx={2}
                />
              )}
            </g>
          );
        })}

        {/* White keys */}
        {whites.map((note) => {
          const x = whiteX[note];
          const covering = zonesForNote(note);
          const topColor =
            covering.length > 0
              ? zoneColorMap[covering[covering.length - 1].id]
              : null;
          const isRoot = instrument.zones.some((z) => z.root_note === note);
          return (
            <g
              key={note}
              onClick={() => handleKeyClick(note)}
              style={{ cursor: "pointer" }}
            >
              <rect
                x={x + 0.5}
                y={10}
                width={whiteWidth - 1}
                height={whiteHeight}
                fill={topColor ? `${topColor}33` : "#e8e8e8"}
                stroke="#333"
                strokeWidth={0.5}
                rx={2}
              />
              {isRoot && (
                <circle
                  cx={x + whiteWidth / 2}
                  cy={10 + whiteHeight - 10}
                  r={3}
                  fill={topColor ?? "#4fc3f7"}
                />
              )}
              {note % 12 === 0 && (
                <text
                  x={x + whiteWidth / 2}
                  y={10 + whiteHeight + 14}
                  textAnchor="middle"
                  fontSize={8}
                  fill="#666"
                >
                  {midiNoteName(note)}
                </text>
              )}
            </g>
          );
        })}

        {/* Black keys */}
        {Array.from(
          { length: endNote - startNote + 1 },
          (_, i) => startNote + i,
        )
          .filter(isBlack)
          .map((note) => {
            const x = noteX(note);
            const covering = zonesForNote(note);
            const topColor =
              covering.length > 0
                ? zoneColorMap[covering[covering.length - 1].id]
                : null;
            const isRoot = instrument.zones.some((z) => z.root_note === note);
            return (
              <g
                key={note}
                onClick={() => handleKeyClick(note)}
                style={{ cursor: "pointer" }}
              >
                <rect
                  x={x}
                  y={10}
                  width={blackWidth}
                  height={blackHeight}
                  fill={topColor ? topColor : "#111"}
                  stroke="#000"
                  strokeWidth={0.5}
                  rx={1}
                  opacity={topColor ? 0.85 : 1}
                />
                {isRoot && (
                  <circle
                    cx={x + blackWidth / 2}
                    cy={10 + blackHeight - 8}
                    r={2.5}
                    fill="white"
                  />
                )}
              </g>
            );
          })}
      </svg>
    </div>
  );
}
