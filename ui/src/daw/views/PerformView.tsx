/**
 * PerformView — live performance mixer.
 *
 * Professional DJ/live-sound style console:
 * - Channel strips with EQ knobs, faders, VU meters
 * - XY performance pad (filter cutoff × reverb wet)
 * - Crossfader A/B
 * - Master section
 * - Input/output routing panel
 */

import { useState, useRef, useCallback } from "react";
import { useDawStore } from "../store/dawStore";
import { useEngineStore } from "../../studio/store/engineStore";

// ── Circular knob ─────────────────────────────────────────────────────────────

function Knob({
  value,
  min,
  max,
  onChange,
  color = "#4db8ff",
  size = 36,
  label,
}: {
  value: number;
  min: number;
  max: number;
  onChange: (v: number) => void;
  color?: string;
  size?: number;
  label?: string;
}) {
  const dragging = useRef(false);
  const startY = useRef(0);
  const startVal = useRef(0);

  const pct = (value - min) / (max - min);
  const angle = -135 + pct * 270;
  const rad = (angle * Math.PI) / 180;
  const cx = size / 2;
  const r = size / 2 - 4;
  const ix = cx + r * Math.sin(rad);
  const iy = cx - r * Math.cos(rad);

  const onMouseDown = (e: React.MouseEvent) => {
    dragging.current = true;
    startY.current = e.clientY;
    startVal.current = value;
    const onMove = (ev: MouseEvent) => {
      if (!dragging.current) return;
      const dy = startY.current - ev.clientY;
      const range = max - min;
      onChange(
        Math.max(min, Math.min(max, startVal.current + (dy / 100) * range)),
      );
    };
    const onUp = () => {
      dragging.current = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 2,
      }}
    >
      <svg
        width={size}
        height={size}
        onMouseDown={onMouseDown}
        style={{ cursor: "ns-resize", userSelect: "none" }}
      >
        <circle
          cx={cx}
          cy={cx}
          r={r}
          fill="#0a1520"
          stroke="#1a2a3a"
          strokeWidth={1.5}
        />
        <circle
          cx={cx}
          cy={cx}
          r={r - 2}
          fill="none"
          stroke={`${color}30`}
          strokeWidth={3}
          strokeDasharray={`${(270 / 360) * 2 * Math.PI * (r - 2)} ${2 * Math.PI * (r - 2)}`}
          strokeDashoffset={`${(45 / 360) * 2 * Math.PI * (r - 2)}`}
          transform={`rotate(-135 ${cx} ${cx})`}
        />
        <circle
          cx={cx}
          cy={cx}
          r={r - 2}
          fill="none"
          stroke={color}
          strokeWidth={3}
          strokeDasharray={`${((pct * 270) / 360) * 2 * Math.PI * (r - 2)} ${2 * Math.PI * (r - 2)}`}
          strokeDashoffset={`${(45 / 360) * 2 * Math.PI * (r - 2)}`}
          transform={`rotate(-135 ${cx} ${cx})`}
        />
        <line
          x1={cx}
          y1={cx}
          x2={ix}
          y2={iy}
          stroke={color}
          strokeWidth={2}
          strokeLinecap="round"
        />
      </svg>
      {label && (
        <span
          style={{
            fontSize: 8,
            color: "#475569",
            fontFamily: "monospace",
            letterSpacing: "0.05em",
          }}
        >
          {label}
        </span>
      )}
    </div>
  );
}

// ── VU meter ──────────────────────────────────────────────────────────────────

function VuMeter({ level, height = 80 }: { level: number; height?: number }) {
  const segments = 12;
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column-reverse",
        gap: 1,
        height,
      }}
    >
      {Array.from({ length: segments }, (_, i) => {
        const threshold = i / segments;
        const active = level > threshold;
        const color = i >= 10 ? "#ef5350" : i >= 8 ? "#ffd54f" : "#34d399";
        return (
          <div
            key={i}
            style={{
              flex: 1,
              background: active ? color : "#0a1520",
              borderRadius: 1,
              transition: "background 0.05s",
            }}
          />
        );
      })}
    </div>
  );
}

// ── Channel strip ─────────────────────────────────────────────────────────────

interface ChannelStripProps {
  name: string;
  color: string;
  volume: number;
  pan: number;
  muted: boolean;
  solo: boolean;
  armed?: boolean;
  group: "A" | "B" | "none";
  level: number;
  onVolume: (v: number) => void;
  onPan: (v: number) => void;
  onMute: () => void;
  onSolo: () => void;
  onGroup: (g: "A" | "B" | "none") => void;
}

function ChannelStrip({
  name,
  color,
  volume,
  pan,
  muted,
  solo,
  group,
  level,
  onVolume,
  onPan,
  onMute,
  onSolo,
  onGroup,
}: ChannelStripProps) {
  const [eq, setEq] = useState({ hi: 0.5, mid: 0.5, lo: 0.5 });
  const [send, setSend] = useState(0);
  const border = "#0f1e2e";
  const dim = "#475569";

  return (
    <div
      style={{
        width: 64,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 4,
        background: "#080e18",
        border: `1px solid ${border}`,
        borderRadius: 6,
        padding: "8px 4px",
        flexShrink: 0,
      }}
    >
      {/* Name */}
      <div
        style={{
          fontSize: 9,
          fontWeight: 700,
          color,
          textAlign: "center",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          width: "100%",
          letterSpacing: "0.06em",
        }}
      >
        {name}
      </div>

      {/* EQ knobs */}
      <Knob
        value={eq.hi}
        min={0}
        max={1}
        onChange={(v) => setEq((e) => ({ ...e, hi: v }))}
        color="#4db8ff"
        size={28}
        label="HI"
      />
      <Knob
        value={eq.mid}
        min={0}
        max={1}
        onChange={(v) => setEq((e) => ({ ...e, mid: v }))}
        color="#a78bfa"
        size={28}
        label="MID"
      />
      <Knob
        value={eq.lo}
        min={0}
        max={1}
        onChange={(v) => setEq((e) => ({ ...e, lo: v }))}
        color="#f97316"
        size={28}
        label="LO"
      />

      {/* FX send */}
      <Knob
        value={send}
        min={0}
        max={1}
        onChange={setSend}
        color="#34d399"
        size={24}
        label="FX"
      />

      {/* Pan */}
      <Knob
        value={(pan + 1) / 2}
        min={0}
        max={1}
        onChange={(v) => onPan(v * 2 - 1)}
        color="#fbbf24"
        size={28}
        label="PAN"
      />

      {/* VU meter */}
      <VuMeter level={muted ? 0 : level * volume} height={60} />

      {/* Fader */}
      <div
        style={{
          position: "relative",
          height: 100,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <input
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={volume}
          onChange={(e) => onVolume(parseFloat(e.target.value))}
          style={{
            writingMode: "vertical-lr",
            direction: "rtl",
            height: 90,
            width: 20,
            cursor: "pointer",
          }}
        />
      </div>

      {/* Volume % */}
      <div style={{ fontSize: 8, color: dim, fontFamily: "monospace" }}>
        {Math.round(volume * 100)}%
      </div>

      {/* CUE / M / S */}
      <div style={{ display: "flex", gap: 2 }}>
        {[
          ["M", muted, "#ef5350", onMute],
          ["S", solo, "#ffd54f", onSolo],
        ].map(([l, a, c, fn]) => (
          <button
            key={l as string}
            onClick={fn as () => void}
            style={{
              padding: "2px 4px",
              background: a ? `${c as string}20` : "transparent",
              border: `1px solid ${a ? `${c as string}50` : border}`,
              borderRadius: 3,
              color: a ? (c as string) : dim,
              fontSize: 8,
              cursor: "pointer",
              fontFamily: "var(--font-sans)",
            }}
          >
            {l as string}
          </button>
        ))}
      </div>

      {/* A/B group */}
      <div style={{ display: "flex", gap: 2 }}>
        {(["A", "B"] as const).map((g) => (
          <button
            key={g}
            onClick={() => onGroup(group === g ? "none" : g)}
            style={{
              padding: "1px 5px",
              background:
                group === g
                  ? g === "A"
                    ? "rgba(77,184,255,0.2)"
                    : "rgba(167,139,250,0.2)"
                  : "transparent",
              border: `1px solid ${group === g ? (g === "A" ? "#4db8ff" : "#a78bfa") : border}`,
              borderRadius: 3,
              color: group === g ? (g === "A" ? "#4db8ff" : "#a78bfa") : dim,
              fontSize: 8,
              cursor: "pointer",
              fontFamily: "var(--font-sans)",
            }}
          >
            {g}
          </button>
        ))}
      </div>
    </div>
  );
}

// ── XY Pad ────────────────────────────────────────────────────────────────────

function XYPad({ onChange }: { onChange: (x: number, y: number) => void }) {
  const [pos, setPos] = useState({ x: 0.5, y: 0.5 });
  const padRef = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);

  const updatePos = useCallback(
    (e: MouseEvent | React.MouseEvent) => {
      if (!padRef.current) return;
      const rect = padRef.current.getBoundingClientRect();
      const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      const y = Math.max(
        0,
        Math.min(1, 1 - (e.clientY - rect.top) / rect.height),
      );
      setPos({ x, y });
      onChange(x, y);
    },
    [onChange],
  );

  const onMouseDown = (e: React.MouseEvent) => {
    dragging.current = true;
    updatePos(e);
    const onMove = (ev: MouseEvent) => {
      if (dragging.current) updatePos(ev);
    };
    const onUp = () => {
      dragging.current = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
      <div
        style={{
          fontSize: 9,
          color: "#475569",
          letterSpacing: "0.1em",
          textAlign: "center",
        }}
      >
        XY PAD
      </div>
      <div
        ref={padRef}
        onMouseDown={onMouseDown}
        style={{
          width: 160,
          height: 160,
          background: "radial-gradient(circle at center, #0a1520, #060c12)",
          border: "1px solid #1a2a3a",
          borderRadius: 8,
          position: "relative",
          cursor: "crosshair",
          overflow: "hidden",
        }}
      >
        {/* Grid lines */}
        <div
          style={{
            position: "absolute",
            left: "50%",
            top: 0,
            bottom: 0,
            width: 1,
            background: "#0f1e2e",
          }}
        />
        <div
          style={{
            position: "absolute",
            top: "50%",
            left: 0,
            right: 0,
            height: 1,
            background: "#0f1e2e",
          }}
        />
        {/* Labels */}
        <span
          style={{
            position: "absolute",
            bottom: 4,
            left: 4,
            fontSize: 7,
            color: "#1e2d3a",
            fontFamily: "monospace",
          }}
        >
          FILTER
        </span>
        <span
          style={{
            position: "absolute",
            top: 4,
            right: 4,
            fontSize: 7,
            color: "#1e2d3a",
            fontFamily: "monospace",
          }}
        >
          REVERB
        </span>
        {/* Dot */}
        <div
          style={{
            position: "absolute",
            left: `${pos.x * 100}%`,
            top: `${(1 - pos.y) * 100}%`,
            width: 14,
            height: 14,
            borderRadius: "50%",
            background: "#4db8ff",
            transform: "translate(-50%, -50%)",
            boxShadow: "0 0 12px #4db8ff, 0 0 24px rgba(77,184,255,0.4)",
            pointerEvents: "none",
          }}
        />
      </div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          fontSize: 8,
          color: "#334155",
          fontFamily: "monospace",
        }}
      >
        <span>X: {Math.round(pos.x * 100)}%</span>
        <span>Y: {Math.round(pos.y * 100)}%</span>
      </div>
    </div>
  );
}

// ── Main PerformView ──────────────────────────────────────────────────────────

export function PerformView() {
  const channels = useDawStore((s) => s.channels);
  const updateChannel = useDawStore((s) => s.updateChannel);
  const transport = useDawStore((s) => s.transport);
  const setBpm = useDawStore((s) => s.setBpm);
  const audioActive = useEngineStore((s) => s.audioActive);

  const [crossfader, setCrossfader] = useState(0.5);
  const [masterVol, setMasterVol] = useState(0.85);
  const [masterLimit, setMasterLimit] = useState(true);
  const [groups, setGroups] = useState<Record<string, "A" | "B" | "none">>({});
  const [routingOpen, setRoutingOpen] = useState(false);
  const tapTimes = useRef<number[]>([]);

  const handleTap = () => {
    const now = performance.now();
    tapTimes.current.push(now);
    if (tapTimes.current.length > 4) tapTimes.current.shift();
    if (tapTimes.current.length >= 2) {
      const intervals = tapTimes.current
        .slice(1)
        .map((t, i) => t - tapTimes.current[i]);
      setBpm(
        Math.round(
          60000 / (intervals.reduce((a, b) => a + b, 0) / intervals.length),
        ),
      );
    }
  };

  const masterLevel = audioActive ? 0.7 + Math.random() * 0.3 : 0;

  const border = "#0f1e2e";
  const dim = "#475569";
  const text = "#e0e8f0";

  return (
    <div
      style={{
        height: "100%",
        background: "#060c12",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Header */}
      <div
        style={{
          height: 36,
          background: "#080e18",
          borderBottom: `1px solid ${border}`,
          display: "flex",
          alignItems: "center",
          padding: "0 16px",
          gap: 12,
          flexShrink: 0,
        }}
      >
        <span style={{ fontSize: 11, fontWeight: 700, color: text }}>
          Live Mixer
        </span>
        <span style={{ fontSize: 10, color: dim }}>
          {channels.length - 1} channels
        </span>
        <div style={{ flex: 1 }} />
        <button
          onClick={() => setRoutingOpen((v) => !v)}
          style={{
            padding: "3px 10px",
            background: routingOpen ? "rgba(77,184,255,0.1)" : "transparent",
            border: `1px solid ${routingOpen ? "#4db8ff" : border}`,
            borderRadius: 4,
            color: routingOpen ? "#4db8ff" : dim,
            fontSize: 10,
            cursor: "pointer",
            fontFamily: "var(--font-sans)",
          }}
        >
          I/O Routing
        </button>
      </div>

      {/* Main mixer area */}
      <div style={{ flex: 1, display: "flex", overflow: "hidden" }}>
        {/* Channel strips */}
        <div
          style={{
            flex: 1,
            display: "flex",
            overflowX: "auto",
            overflowY: "hidden",
            padding: "12px 8px",
            gap: 6,
            alignItems: "flex-end",
          }}
        >
          {channels
            .filter((c) => c.id !== "master")
            .map((ch) => (
              <ChannelStrip
                key={ch.id}
                name={ch.name}
                color={ch.color}
                volume={ch.volume}
                pan={ch.pan}
                muted={ch.muted}
                solo={ch.solo}
                group={groups[ch.id] ?? "none"}
                level={audioActive ? 0.4 + Math.random() * 0.5 : 0}
                onVolume={(v) => updateChannel(ch.id, { volume: v })}
                onPan={(v) => updateChannel(ch.id, { pan: v })}
                onMute={() => updateChannel(ch.id, { muted: !ch.muted })}
                onSolo={() => updateChannel(ch.id, { solo: !ch.solo })}
                onGroup={(g) => setGroups((prev) => ({ ...prev, [ch.id]: g }))}
              />
            ))}
        </div>

        {/* Right panel: XY pad + master */}
        <div
          style={{
            width: 200,
            borderLeft: `1px solid ${border}`,
            display: "flex",
            flexDirection: "column",
            padding: 12,
            gap: 12,
            flexShrink: 0,
            overflowY: "auto",
          }}
        >
          {/* XY Pad */}
          <XYPad
            onChange={(x, y) => {
              // x → filter cutoff, y → reverb wet (future: wire to engine)
              void x;
              void y;
            }}
          />

          {/* BPM + tap */}
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            <div
              style={{
                fontSize: 9,
                color: dim,
                letterSpacing: "0.1em",
                textAlign: "center",
              }}
            >
              BPM
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <span
                style={{
                  fontSize: 20,
                  fontWeight: 800,
                  color: text,
                  fontFamily: "monospace",
                  flex: 1,
                  textAlign: "center",
                }}
              >
                {transport.bpm}
              </span>
              <button
                onClick={handleTap}
                style={{
                  padding: "4px 8px",
                  background: "rgba(77,184,255,0.08)",
                  border: "1px solid rgba(77,184,255,0.2)",
                  borderRadius: 4,
                  color: "#4db8ff",
                  fontSize: 10,
                  cursor: "pointer",
                  fontFamily: "var(--font-sans)",
                }}
              >
                TAP
              </button>
            </div>
          </div>

          {/* Master section */}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 6,
              padding: "10px 8px",
              background: "#080e18",
              border: `1px solid ${border}`,
              borderRadius: 6,
            }}
          >
            <div
              style={{
                fontSize: 9,
                fontWeight: 700,
                color: text,
                letterSpacing: "0.12em",
              }}
            >
              MASTER
            </div>
            <div style={{ display: "flex", gap: 4 }}>
              <VuMeter level={masterLevel} height={80} />
              <VuMeter level={masterLevel * 0.9} height={80} />
            </div>
            <input
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={masterVol}
              onChange={(e) => setMasterVol(parseFloat(e.target.value))}
              style={{
                writingMode: "vertical-lr",
                direction: "rtl",
                height: 80,
                width: 20,
                cursor: "pointer",
              }}
            />
            <div style={{ fontSize: 9, color: dim, fontFamily: "monospace" }}>
              {Math.round(masterVol * 100)}%
            </div>
            <button
              onClick={() => setMasterLimit((v) => !v)}
              style={{
                padding: "2px 8px",
                background: masterLimit
                  ? "rgba(239,83,80,0.15)"
                  : "transparent",
                border: `1px solid ${masterLimit ? "rgba(239,83,80,0.4)" : border}`,
                borderRadius: 4,
                color: masterLimit ? "#ef5350" : dim,
                fontSize: 9,
                cursor: "pointer",
                fontFamily: "var(--font-sans)",
              }}
            >
              LIMIT
            </button>
          </div>
        </div>
      </div>

      {/* Crossfader */}
      <div
        style={{
          height: 52,
          background: "#080e18",
          borderTop: `1px solid ${border}`,
          display: "flex",
          alignItems: "center",
          padding: "0 20px",
          gap: 12,
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontSize: 10,
            fontWeight: 700,
            color: "#4db8ff",
            minWidth: 16,
          }}
        >
          A
        </span>
        <div style={{ flex: 1, position: "relative" }}>
          <input
            type="range"
            min={0}
            max={1}
            step={0.01}
            value={crossfader}
            onChange={(e) => setCrossfader(parseFloat(e.target.value))}
            style={{ width: "100%", height: 20, cursor: "pointer" }}
          />
          <div
            style={{
              position: "absolute",
              left: "50%",
              top: -4,
              transform: "translateX(-50%)",
              fontSize: 8,
              color: dim,
              fontFamily: "monospace",
              pointerEvents: "none",
            }}
          >
            {crossfader === 0.5
              ? "CENTER"
              : crossfader < 0.5
                ? `A ${Math.round((0.5 - crossfader) * 200)}%`
                : `B ${Math.round((crossfader - 0.5) * 200)}%`}
          </div>
        </div>
        <span
          style={{
            fontSize: 10,
            fontWeight: 700,
            color: "#a78bfa",
            minWidth: 16,
          }}
        >
          B
        </span>
      </div>

      {/* I/O Routing panel */}
      {routingOpen && (
        <div
          style={{
            height: 120,
            background: "#080e18",
            borderTop: `1px solid ${border}`,
            padding: "10px 16px",
            flexShrink: 0,
            overflowX: "auto",
          }}
        >
          <div
            style={{
              fontSize: 10,
              color: dim,
              fontWeight: 600,
              letterSpacing: "0.08em",
              marginBottom: 8,
            }}
          >
            INPUT / OUTPUT ROUTING
          </div>
          <div style={{ display: "flex", gap: 24 }}>
            <div>
              <div style={{ fontSize: 9, color: dim, marginBottom: 4 }}>
                INPUTS
              </div>
              {["Mic 1", "Mic 2", "Line In L", "Line In R", "USB Audio"].map(
                (inp) => (
                  <div
                    key={inp}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 6,
                      marginBottom: 3,
                    }}
                  >
                    <input
                      type="checkbox"
                      defaultChecked={inp.startsWith("Line")}
                      style={{ cursor: "pointer" }}
                    />
                    <span
                      style={{
                        fontSize: 10,
                        color: text,
                        fontFamily: "var(--font-sans)",
                      }}
                    >
                      {inp}
                    </span>
                  </div>
                ),
              )}
            </div>
            <div>
              <div style={{ fontSize: 9, color: dim, marginBottom: 4 }}>
                OUTPUTS
              </div>
              {["Main L/R", "Monitor", "Headphones", "Record Bus"].map(
                (out) => (
                  <div
                    key={out}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 6,
                      marginBottom: 3,
                    }}
                  >
                    <input
                      type="checkbox"
                      defaultChecked={out === "Main L/R"}
                      style={{ cursor: "pointer" }}
                    />
                    <span
                      style={{
                        fontSize: 10,
                        color: text,
                        fontFamily: "var(--font-sans)",
                      }}
                    >
                      {out}
                    </span>
                  </div>
                ),
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
