import { useEffect, useRef } from "react";

interface ScopeCanvasProps {
  samples: Float32Array | null;
}

const W = 200;
const H = 60;
const STALE_MS = 200;

export function ScopeCanvas({ samples }: ScopeCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const lastFrameTime = useRef<number>(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const now = performance.now();
    const isStale = samples === null || now - lastFrameTime.current > STALE_MS;
    if (samples !== null) lastFrameTime.current = now;

    ctx.fillStyle = "#0a1520";
    ctx.fillRect(0, 0, W, H);

    // Center line
    ctx.strokeStyle = "#1a2a3a";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, H / 2);
    ctx.lineTo(W, H / 2);
    ctx.stroke();

    ctx.strokeStyle = "#38bdf8";
    ctx.lineWidth = 1.5;
    ctx.beginPath();

    if (isStale || !samples || samples.length === 0) {
      ctx.moveTo(0, H / 2);
      ctx.lineTo(W, H / 2);
    } else {
      const len = samples.length;
      for (let i = 0; i < len; i++) {
        const x = (i / (len - 1)) * W;
        const clamped = Math.max(-1, Math.min(1, samples[i]));
        const y = ((1 - clamped) / 2) * H;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
    }
    ctx.stroke();
  }, [samples]);

  return (
    <canvas
      ref={canvasRef}
      width={W}
      height={H}
      style={{ display: "block", borderRadius: 4 }}
    />
  );
}
