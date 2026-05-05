/**
 * CpuMeter — per-node CPU usage display.
 *
 * The host sends cpu_usage messages with per-node timing data.
 * This component renders a small bar overlay on each node showing
 * its CPU load as a percentage of the 1.33ms block budget.
 *
 * The host needs to send:
 *   { type: "cpu_usage", nodes: [{ id: number, us: number }] }
 * where `us` is microseconds spent processing that node per block.
 *
 * Budget: 64 samples / 48000 Hz = 1333 µs per block.
 */

import { useEffect, useRef, useState } from "react";
import { useEngineStore } from "../store/engineStore";

const BLOCK_BUDGET_US = 1333; // 64 samples @ 48kHz

interface NodeCpuEntry {
  id: string;
  us: number;
  pct: number;
}

export function useCpuMeter() {
  const [cpuData, setCpuData] = useState<Map<string, NodeCpuEntry>>(new Map());
  const sendIntent = useEngineStore((s) => s.sendIntent);

  // Poll the host for CPU data every 500ms
  useEffect(() => {
    const interval = setInterval(() => {
      sendIntent?.({ type: "get_cpu_usage" });
    }, 500);
    return () => clearInterval(interval);
  }, [sendIntent]);

  // Listen for cpu_usage messages via a custom event dispatched by useEngineSocket
  useEffect(() => {
    const handler = (
      e: CustomEvent<{ nodes: Array<{ id: number; us: number }> }>,
    ) => {
      const map = new Map<string, NodeCpuEntry>();
      for (const entry of e.detail.nodes) {
        const pct = (entry.us / BLOCK_BUDGET_US) * 100;
        map.set(String(entry.id), { id: String(entry.id), us: entry.us, pct });
      }
      setCpuData(map);
    };
    window.addEventListener("aether:cpu_usage", handler as EventListener);
    return () =>
      window.removeEventListener("aether:cpu_usage", handler as EventListener);
  }, []);

  return cpuData;
}

interface CpuBarProps {
  nodeId: string;
  cpuData: Map<string, NodeCpuEntry>;
  x: number;
  y: number;
  width: number;
  zoom: number;
}

export function CpuBar({ nodeId, cpuData, x, y, width, zoom }: CpuBarProps) {
  const entry = cpuData.get(nodeId);
  if (!entry || entry.pct < 0.5) return null; // hide if < 0.5% — not worth showing

  const pct = Math.min(entry.pct, 100);
  const color = pct > 80 ? "#ef4444" : pct > 50 ? "#ffd54f" : "#34d399";
  const barH = Math.max(3, 3 * zoom);

  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: y - barH - 2 * zoom,
        width,
        height: barH,
        background: "#0a1520",
        borderRadius: barH / 2,
        overflow: "hidden",
      }}
      title={`CPU: ${pct.toFixed(1)}% (${entry.us.toFixed(0)}µs)`}
    >
      <div
        style={{
          width: `${pct}%`,
          height: "100%",
          background: color,
          borderRadius: barH / 2,
          transition: "width 0.3s, background 0.3s",
        }}
      />
    </div>
  );
}

/** Master CPU indicator shown in the top bar */
export function MasterCpuIndicator() {
  const cpuRef = useRef(0);
  const [display, setDisplay] = useState(0);

  useEffect(() => {
    const handler = (
      e: CustomEvent<{ nodes: Array<{ id: number; us: number }> }>,
    ) => {
      const total = e.detail.nodes.reduce((sum, n) => sum + n.us, 0);
      cpuRef.current = (total / BLOCK_BUDGET_US) * 100;
      setDisplay(Math.round(cpuRef.current));
    };
    window.addEventListener("aether:cpu_usage", handler as EventListener);
    return () =>
      window.removeEventListener("aether:cpu_usage", handler as EventListener);
  }, []);

  if (display === 0) return null;

  const color = display > 80 ? "#ef4444" : display > 50 ? "#ffd54f" : "#34d399";

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 4,
        fontSize: 10,
        fontFamily: "monospace",
        color,
      }}
      title={`Total DSP CPU: ${display}% of 1.33ms block budget`}
    >
      <span>CPU</span>
      <span style={{ fontWeight: 700 }}>{display}%</span>
    </div>
  );
}
