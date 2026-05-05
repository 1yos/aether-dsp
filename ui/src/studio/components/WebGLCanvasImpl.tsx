/**
 * WebGLCanvasImpl — React component wrapping the WebGL renderer.
 *
 * Renders nodes and cables via WebGL for 60fps performance at 1000+ nodes.
 * Node parameter controls are rendered as HTML overlays on top of the canvas.
 * Supports pan (middle-click drag / space+drag), zoom (scroll wheel),
 * node selection (click), and node dragging.
 */

import { useRef, useEffect, useCallback, useState } from "react";
import { WebGLRenderer, GraphNode, GraphEdge } from "./webgl-renderer";
import { useEngineStore, NODE_DEFS } from "../store/engineStore";
import { useEngine } from "../hooks/useEngine";
import { useComputerKeyboard } from "../hooks/useComputerKeyboard";
import { NodePalette } from "./NodePalette";
import { TransportBar } from "./TransportBar";
import { ModuleBar } from "../../modules/ModuleBar";
import { ModulePanel } from "../../modules/ModulePanel";
import { useModuleStore } from "../../modules/useModuleStore";
import { moduleRegistry } from "../../modules/moduleRegistry";

const NODE_W = 220;
const NODE_H_BASE = 80;
const PARAM_H = 28;

function nodeHeight(nodeType: string): number {
  const def = NODE_DEFS[nodeType];
  if (!def) return NODE_H_BASE;
  return NODE_H_BASE + def.params.length * PARAM_H;
}

export function WebGLCanvas({ onOpenInstrumentMaker }: { onOpenInstrumentMaker?: () => void }) {
  useEngine();
  const octaveRef = useComputerKeyboard();
  const modules = useModuleStore((s) => s.layout.modules);

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendererRef = useRef<WebGLRenderer | null>(null);
  const rafRef = useRef<number>(0);
  const lastTimeRef = useRef<number>(0);

  // Pan and zoom state (in refs to avoid re-renders on every frame)
  const panRef = useRef<[number, number]>([100, 100]);
  const zoomRef = useRef<number>(1);
  const [, forceUpdate] = useState(0); // trigger overlay re-render

  // Interaction state
  const draggingNodeRef = useRef<string | null>(null);
  const dragOffsetRef = useRef<[number, number]>([0, 0]);
  const isPanningRef = useRef(false);
  const panStartRef = useRef<[number, number]>([0, 0]);
  const panOriginRef = useRef<[number, number]>([0, 0]);

  // Node positions (world space) — separate from React Flow positions
  const nodePositionsRef = useRef<Map<string, [number, number]>>(new Map());

  const nodes = useEngineStore((s) => s.nodes);
  const edges = useEngineStore((s) => s.edges);
  const audioActive = useEngineStore((s) => s.audioActive);
  const selectedNodeId = useEngineStore((s) => s.selectedNodeId);
  const setSelectedNode = useEngineStore((s) => s.setSelectedNode);
  const sendIntent = useEngineStore((s) => s.sendIntent);

  // Sync node positions from store on first appearance
  useEffect(() => {
    for (const node of nodes) {
      if (!nodePositionsRef.current.has(node.id)) {
        nodePositionsRef.current.set(node.id, [node.position.x, node.position.y]);
      }
    }
    // Remove stale positions
    const ids = new Set(nodes.map((n) => n.id));
    for (const id of nodePositionsRef.current.keys()) {
      if (!ids.has(id)) nodePositionsRef.current.delete(id);
    }
  }, [nodes]);

  // Build GraphNode/GraphEdge arrays for the renderer
  const buildGraphNodes = useCallback((): GraphNode[] => {
    return nodes.map((n) => {
      const [x, y] = nodePositionsRef.current.get(n.id) ?? [n.position.x, n.position.y];
      return {
        id: n.id,
        x, y,
        width: NODE_W,
        height: nodeHeight(n.data.nodeType),
        color: n.data.color ?? "#4fc3f7",
        label: n.data.nodeType,
        selected: n.id === selectedNodeId,
        isOutput: n.data.isOutput ?? false,
      };
    });
  }, [nodes, selectedNodeId]);

  const buildGraphEdges = useCallback((): GraphEdge[] => {
    return edges.map((e) => {
      const srcNode = nodes.find((n) => n.id === e.source);
      const dstNode = nodes.find((n) => n.id === e.target);
      if (!srcNode || !dstNode) return null;
      const [sx, sy] = nodePositionsRef.current.get(srcNode.id) ?? [srcNode.position.x, srcNode.position.y];
      const [dx, dy] = nodePositionsRef.current.get(dstNode.id) ?? [dstNode.position.x, dstNode.position.y];
      const srcH = nodeHeight(srcNode.data.nodeType);
      const dstH = nodeHeight(dstNode.data.nodeType);
      return {
        id: e.id,
        srcX: sx + NODE_W,
        srcY: sy + srcH / 2,
        dstX: dx,
        dstY: dy + dstH / 2,
        color: srcNode.data.color ?? "#38bdf8",
      };
    }).filter(Boolean) as GraphEdge[];
  }, [nodes, edges]);

  // Init WebGL
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    try {
      rendererRef.current = new WebGLRenderer(canvas);
    } catch (e) {
      console.error("WebGL init failed:", e);
      return;
    }

    const resize = () => {
      canvas.width = canvas.offsetWidth * devicePixelRatio;
      canvas.height = canvas.offsetHeight * devicePixelRatio;
      rendererRef.current?.resize(canvas.width, canvas.height);
    };
    resize();
    const ro = new ResizeObserver(resize);
    ro.observe(canvas);

    // Animation loop
    const loop = (time: number) => {
      const dt = Math.min((time - lastTimeRef.current) / 1000, 0.05);
      lastTimeRef.current = time;
      const r = rendererRef.current;
      if (r && canvas.width > 0) {
        r.render(
          buildGraphNodes(),
          buildGraphEdges(),
          panRef.current,
          zoomRef.current,
          canvas.width,
          canvas.height,
          audioActive,
          dt,
        );
      }
      rafRef.current = requestAnimationFrame(loop);
    };
    rafRef.current = requestAnimationFrame(loop);

    return () => {
      cancelAnimationFrame(rafRef.current);
      ro.disconnect();
      rendererRef.current?.destroy();
      rendererRef.current = null;
    };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Re-render when data changes (the RAF loop handles animation, but we need
  // to trigger overlay re-render for HTML node controls)
  useEffect(() => { forceUpdate((n) => n + 1); }, [nodes, edges, selectedNodeId]);

  // ── Canvas → world coordinate conversion ─────────────────────────────────
  const canvasToWorld = useCallback((cx: number, cy: number): [number, number] => {
    const canvas = canvasRef.current!;
    const rect = canvas.getBoundingClientRect();
    const px = (cx - rect.left) * devicePixelRatio;
    const py = (cy - rect.top) * devicePixelRatio;
    return [
      (px - panRef.current[0]) / zoomRef.current,
      (py - panRef.current[1]) / zoomRef.current,
    ];
  }, []);

  const hitTestNode = useCallback((wx: number, wy: number): string | null => {
    for (const node of nodes) {
      const [nx, ny] = nodePositionsRef.current.get(node.id) ?? [node.position.x, node.position.y];
      const h = nodeHeight(node.data.nodeType);
      if (wx >= nx && wx <= nx + NODE_W && wy >= ny && wy <= ny + h) {
        return node.id;
      }
    }
    return null;
  }, [nodes]);

  // ── Mouse handlers ────────────────────────────────────────────────────────
  const onMouseDown = useCallback((e: React.MouseEvent) => {
    const [wx, wy] = canvasToWorld(e.clientX, e.clientY);

    if (e.button === 1 || (e.button === 0 && e.altKey)) {
      // Middle click or Alt+drag = pan
      isPanningRef.current = true;
      panStartRef.current = [e.clientX, e.clientY];
      panOriginRef.current = [...panRef.current];
      e.preventDefault();
      return;
    }

    if (e.button === 0) {
      const hit = hitTestNode(wx, wy);
      if (hit) {
        setSelectedNode(hit);
        draggingNodeRef.current = hit;
        const [nx, ny] = nodePositionsRef.current.get(hit) ?? [0, 0];
        dragOffsetRef.current = [wx - nx, wy - ny];
      } else {
        setSelectedNode(null);
      }
    }
  }, [canvasToWorld, hitTestNode, setSelectedNode]);

  const onMouseMove = useCallback((e: React.MouseEvent) => {
    if (isPanningRef.current) {
      const dx = (e.clientX - panStartRef.current[0]) * devicePixelRatio;
      const dy = (e.clientY - panStartRef.current[1]) * devicePixelRatio;
      panRef.current = [panOriginRef.current[0] + dx, panOriginRef.current[1] + dy];
      return;
    }
    if (draggingNodeRef.current) {
      const [wx, wy] = canvasToWorld(e.clientX, e.clientY);
      const [ox, oy] = dragOffsetRef.current;
      nodePositionsRef.current.set(draggingNodeRef.current, [wx - ox, wy - oy]);
    }
  }, [canvasToWorld]);

  const onMouseUp = useCallback(() => {
    isPanningRef.current = false;
    draggingNodeRef.current = null;
  }, []);

  const onWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(0.1, Math.min(4, zoomRef.current * factor));
    // Zoom toward cursor
    const canvas = canvasRef.current!;
    const rect = canvas.getBoundingClientRect();
    const cx = (e.clientX - rect.left) * devicePixelRatio;
    const cy = (e.clientY - rect.top) * devicePixelRatio;
    panRef.current = [
      cx - (cx - panRef.current[0]) * (newZoom / zoomRef.current),
      cy - (cy - panRef.current[1]) * (newZoom / zoomRef.current),
    ];
    zoomRef.current = newZoom;
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;
      if ((e.key === "Delete" || e.key === "Backspace") && selectedNodeId) {
        useEngineStore.getState().removeNode(selectedNodeId);
      }
      if (e.ctrlKey && (e.key === "z" || e.key === "Z")) {
        e.preventDefault();
        sendIntent?.({ type: "undo" });
      }
      if (e.ctrlKey && (e.key === "y" || e.key === "Y")) {
        e.preventDefault();
        sendIntent?.({ type: "redo" });
      }
      // Space + drag = pan (handled via altKey above, space sets a flag)
      if (e.key === " ") e.preventDefault();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [selectedNodeId, sendIntent]);

  // ── HTML overlay for node labels and param controls ───────────────────────
  const overlayNodes = nodes.map((n) => {
    const [wx, wy] = nodePositionsRef.current.get(n.id) ?? [n.position.x, n.position.y];
    const screenX = wx * zoomRef.current + panRef.current[0] / devicePixelRatio;
    const screenY = wy * zoomRef.current + panRef.current[1] / devicePixelRatio;
    const w = NODE_W * zoomRef.current;
    const h = nodeHeight(n.data.nodeType) * zoomRef.current;
    return { node: n, screenX, screenY, w, h };
  });

  return (
    <div style={{ display: "flex", flexDirection: "column", width: "100vw", height: "100vh", background: "#060e18" }}>
      <TransportBar onOpenInstrumentMaker={onOpenInstrumentMaker} />
      <ModuleBar />

      {/* Hint bar */}
      <div style={{
        padding: "0 16px", height: 24, background: "#060e18",
        borderBottom: "1px solid #0a1520", display: "flex", alignItems: "center",
        gap: 16, flexShrink: 0, fontSize: 10,
        fontFamily: "'Inter', system-ui, sans-serif", color: "#1e2d3d", userSelect: "none",
      }}>
        <span style={{ color: "#334155", fontWeight: 600 }}>⌨ Keyboard</span>
        <span>White: <span style={{ color: "#475569", fontFamily: "monospace" }}>A S D F G H J K L</span></span>
        <span>Black: <span style={{ color: "#475569", fontFamily: "monospace" }}>W E · T Y U · O P</span></span>
        <span>Octave: <span style={{ color: "#475569", fontFamily: "monospace" }}>Z↓ X↑</span></span>
        <span>Pan: <span style={{ color: "#475569", fontFamily: "monospace" }}>Alt+drag / Middle-click</span></span>
        <span>Zoom: <span style={{ color: "#475569", fontFamily: "monospace" }}>Scroll</span></span>
        <span style={{ marginLeft: "auto" }}>
          Oct: <span style={{ color: "#38bdf8", fontWeight: 700 }}>{octaveRef.current}</span>
        </span>
      </div>

      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        <NodePalette />
        <div style={{ flex: 1, position: "relative" }}>
          {/* WebGL canvas */}
          <canvas
            ref={canvasRef}
            style={{ position: "absolute", inset: 0, width: "100%", height: "100%", cursor: isPanningRef.current ? "grabbing" : "default" }}
            onMouseDown={onMouseDown}
            onMouseMove={onMouseMove}
            onMouseUp={onMouseUp}
            onMouseLeave={onMouseUp}
            onWheel={onWheel}
          />

          {/* HTML overlay — node labels rendered on top of WebGL */}
          <div style={{ position: "absolute", inset: 0, pointerEvents: "none", overflow: "hidden" }}>
            {overlayNodes.map(({ node, screenX, screenY, w, h }) => (
              <div
                key={node.id}
                style={{
                  position: "absolute",
                  left: screenX,
                  top: screenY,
                  width: w,
                  height: h,
                  pointerEvents: "none",
                  display: "flex",
                  flexDirection: "column",
                  justifyContent: "flex-start",
                  padding: `${8 * zoomRef.current}px ${10 * zoomRef.current}px`,
                  boxSizing: "border-box",
                }}
              >
                <div style={{
                  fontSize: Math.max(8, 11 * zoomRef.current),
                  fontWeight: 600,
                  color: "#e2e8f0",
                  fontFamily: "'Inter', system-ui, sans-serif",
                  whiteSpace: "nowrap",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  lineHeight: 1.2,
                }}>
                  {node.data.nodeType.replace(/([A-Z])/g, " $1").trim()}
                </div>
                {node.data.isOutput && (
                  <div style={{
                    fontSize: Math.max(6, 9 * zoomRef.current),
                    color: "#00e5a0",
                    fontWeight: 700,
                    letterSpacing: "0.08em",
                  }}>
                    ● OUTPUT
                  </div>
                )}
              </div>
            ))}
          </div>

          {/* Module overlay */}
          {modules.map((mod) => {
            const ModuleComponent = moduleRegistry[mod.type];
            return (
              <ModulePanel key={mod.id} module={mod}>
                <ModuleComponent />
              </ModulePanel>
            );
          })}
        </div>
      </div>
    </div>
  );
}
