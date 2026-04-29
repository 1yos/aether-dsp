/**
 * Instrument Rack Module — lists SamplerNode instances from the engine store.
 * Allows loading instruments and opening them in the Instrument Maker.
 */
import React, { useRef } from "react";
import { useEngineStore } from "../../studio/store/engineStore";

const THEME = {
  bg: "#060c12",
  panel: "#0d1a26",
  border: "#1a2a3a",
  text: "#e0e8f0",
  textDim: "#4a6a8a",
  accent: "#4fc3f7",
  highlight: "#1a3a5a",
};

interface InstrumentSlotProps {
  nodeId: string;
  generation: number;
  label: string;
  midiChannel: number;
  tuningName: string;
  onMidiChannelChange: (channel: number) => void;
  onLoadInstrument: () => void;
  onOpenInMaker: () => void;
}

const InstrumentSlot: React.FC<InstrumentSlotProps> = ({
  label,
  midiChannel,
  tuningName,
  onMidiChannelChange,
  onLoadInstrument,
  onOpenInMaker,
}) => {
  return (
    <div
      style={{
        background: THEME.panel,
        border: `1px solid ${THEME.border}`,
        borderRadius: 6,
        padding: "10px 12px",
        display: "flex",
        flexDirection: "column",
        gap: 8,
      }}
    >
      {/* Slot header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <span
          style={{
            fontSize: 13,
            color: THEME.text,
            fontFamily: "monospace",
            fontWeight: "bold",
          }}
        >
          {label}
        </span>
        <span
          style={{
            fontSize: 10,
            color: THEME.textDim,
            background: THEME.bg,
            padding: "2px 6px",
            borderRadius: 3,
            border: `1px solid ${THEME.border}`,
          }}
        >
          {tuningName}
        </span>
      </div>

      {/* MIDI channel selector */}
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <span style={{ fontSize: 11, color: THEME.textDim, width: 80 }}>
          MIDI Ch
        </span>
        <select
          value={midiChannel}
          onChange={(e) => onMidiChannelChange(Number(e.target.value))}
          style={{
            background: THEME.bg,
            border: `1px solid ${THEME.border}`,
            borderRadius: 4,
            color: THEME.text,
            padding: "2px 6px",
            fontSize: 12,
            fontFamily: "monospace",
            flex: 1,
          }}
        >
          {Array.from({ length: 16 }, (_, i) => (
            <option key={i} value={i}>
              Ch {i + 1}
            </option>
          ))}
        </select>
      </div>

      {/* Action buttons */}
      <div style={{ display: "flex", gap: 8 }}>
        <button
          onClick={onLoadInstrument}
          style={{
            flex: 1,
            padding: "5px 0",
            background: THEME.highlight,
            border: `1px solid ${THEME.accent}`,
            borderRadius: 4,
            color: THEME.accent,
            fontSize: 11,
            cursor: "pointer",
            fontFamily: "monospace",
          }}
        >
          Load Instrument
        </button>
        <button
          onClick={onOpenInMaker}
          style={{
            flex: 1,
            padding: "5px 0",
            background: THEME.panel,
            border: `1px solid ${THEME.border}`,
            borderRadius: 4,
            color: THEME.textDim,
            fontSize: 11,
            cursor: "pointer",
            fontFamily: "monospace",
          }}
        >
          Open in Maker
        </button>
      </div>
    </div>
  );
};

export const InstrumentRackModule: React.FC = () => {
  const nodes = useEngineStore((s) => s.nodes);
  const sendIntent = useEngineStore((s) => s.sendIntent);

  // Local MIDI channel state per node
  const [midiChannels, setMidiChannels] = React.useState<
    Record<string, number>
  >({});
  const fileInputRef = useRef<HTMLInputElement>(null);
  const pendingNodeRef = useRef<{ nodeId: string; generation: number } | null>(
    null,
  );

  const samplerNodes = nodes.filter((n) => n.data?.nodeType === "SamplerNode");

  const handleMidiChannelChange = (nodeId: string, channel: number) => {
    setMidiChannels((prev) => ({ ...prev, [nodeId]: channel }));
  };

  const handleLoadInstrument = (nodeId: string, generation: number) => {
    pendingNodeRef.current = { nodeId, generation };
    fileInputRef.current?.click();
  };

  const handleFileSelected = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !pendingNodeRef.current) return;

    const { nodeId, generation } = pendingNodeRef.current;
    pendingNodeRef.current = null;

    try {
      const text = await file.text();
      // Validate JSON
      const parsed = JSON.parse(text);
      if (!parsed.name || !parsed.zones || !parsed.tuning) {
        alert(
          "Invalid instrument file: missing required fields (name, zones, tuning).",
        );
        return;
      }

      sendIntent?.({
        type: "load_instrument",
        node_id: parseInt(nodeId, 10),
        generation,
        instrument_json: text,
      });
    } catch {
      alert("Failed to read instrument file.");
    }

    // Reset file input
    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  const handleOpenInMaker = (nodeId: string) => {
    // Navigate to Instrument Maker — dispatch a custom event that the app can handle
    window.dispatchEvent(
      new CustomEvent("aether:open-instrument-maker", { detail: { nodeId } }),
    );
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: THEME.bg,
        fontFamily: "monospace",
        color: THEME.text,
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "6px 12px",
          borderBottom: `1px solid ${THEME.border}`,
          fontSize: 11,
          color: THEME.textDim,
          letterSpacing: 2,
          textTransform: "uppercase",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <span>Instrument Rack</span>
        <span style={{ color: THEME.accent }}>
          {samplerNodes.length} slot{samplerNodes.length !== 1 ? "s" : ""}
        </span>
      </div>

      {/* Hidden file input */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".aether-instrument,.json"
        style={{ display: "none" }}
        onChange={handleFileSelected}
      />

      {/* Slots */}
      <div
        style={{
          flex: 1,
          overflowY: "auto",
          padding: 12,
          display: "flex",
          flexDirection: "column",
          gap: 10,
        }}
      >
        {samplerNodes.length === 0 ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
              color: THEME.textDim,
              fontSize: 13,
            }}
          >
            No SamplerNode instances in graph.
            <br />
            Add a SamplerNode from the node palette.
          </div>
        ) : (
          samplerNodes.map((node) => {
            const nodeId = node.id;
            const generation = node.data?.generation ?? 0;
            const label = `SamplerNode-${nodeId}`;
            const midiChannel = midiChannels[nodeId] ?? 0;
            // Tuning name would come from loaded instrument — show placeholder
            const tuningName = "12-TET";

            return (
              <InstrumentSlot
                key={nodeId}
                nodeId={nodeId}
                generation={generation}
                label={label}
                midiChannel={midiChannel}
                tuningName={tuningName}
                onMidiChannelChange={(ch) =>
                  handleMidiChannelChange(nodeId, ch)
                }
                onLoadInstrument={() =>
                  handleLoadInstrument(nodeId, generation)
                }
                onOpenInMaker={() => handleOpenInMaker(nodeId)}
              />
            );
          })
        )}
      </div>
    </div>
  );
};

export default InstrumentRackModule;
