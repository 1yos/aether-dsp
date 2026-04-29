/**
 * ZoneList — list of all sample zones with quick actions.
 */
import { useInstrumentStore } from "../store/instrumentStore";
import { midiNoteName } from "../types";

const ZONE_COLORS = [
  "#4fc3f7",
  "#ce93d8",
  "#ffb74d",
  "#80cbc4",
  "#a5d6a7",
  "#ef9a9a",
  "#fff176",
  "#f48fb1",
];

export function ZoneList() {
  const { instrument, selectedZoneId, selectZone, removeZone } =
    useInstrumentStore();

  if (instrument.zones.length === 0) {
    return (
      <div
        style={{
          padding: "20px 12px",
          color: "#4a6a8a",
          fontSize: 12,
          textAlign: "center",
        }}
      >
        No zones yet.
        <br />
        Drop audio files onto the keyboard or use "Add Sample" above.
      </div>
    );
  }

  return (
    <div style={{ overflowY: "auto", maxHeight: 280 }}>
      {instrument.zones.map((zone, i) => {
        const color = ZONE_COLORS[i % ZONE_COLORS.length];
        const isSelected = zone.id === selectedZoneId;
        const artLabel =
          typeof zone.articulation === "object" && "type" in zone.articulation
            ? zone.articulation.type === "SustainLoop"
              ? "Loop"
              : zone.articulation.type === "SustainRelease"
                ? "Sus+Rel"
                : "OneShot"
            : "OneShot";

        return (
          <div
            key={zone.id}
            onClick={() => selectZone(zone.id)}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              padding: "6px 10px",
              cursor: "pointer",
              background: isSelected ? "#0f2030" : "transparent",
              borderLeft: isSelected
                ? `3px solid ${color}`
                : "3px solid transparent",
              borderBottom: "1px solid #0a1520",
            }}
          >
            <div
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: color,
                flexShrink: 0,
              }}
            />
            <div style={{ flex: 1, minWidth: 0 }}>
              <div
                style={{
                  fontSize: 12,
                  color: "#c0d8f0",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                }}
              >
                {zone.fileName ?? zone.file_path.split("/").pop()}
              </div>
              <div style={{ fontSize: 10, color: "#4a6a8a", marginTop: 1 }}>
                {midiNoteName(zone.note_low)}–{midiNoteName(zone.note_high)} •{" "}
                root: {midiNoteName(zone.root_note)} • {artLabel}
              </div>
            </div>
            <button
              onClick={(e) => {
                e.stopPropagation();
                removeZone(zone.id);
              }}
              style={{
                background: "none",
                border: "none",
                color: "#4a6a8a",
                cursor: "pointer",
                fontSize: 14,
                padding: "0 2px",
                lineHeight: 1,
              }}
              title="Remove zone"
            >
              ×
            </button>
          </div>
        );
      })}
    </div>
  );
}
