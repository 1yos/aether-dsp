import { useEffect, useCallback, useState } from "react";
import type React from "react";
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  BackgroundVariant,
  useReactFlow,
  ReactFlowProvider,
} from "reactflow";
import "reactflow/dist/style.css";
import { useEngineStore } from "../store/engineStore";
import { useEngine } from "../hooks/useEngine";
import { useComputerKeyboard } from "../hooks/useComputerKeyboard";
import StudioNode from "./StudioNode";
import { NodePalette } from "./NodePalette";
import { TransportBar } from "./TransportBar";
import { ModuleBar } from "../../modules/ModuleBar";
import { ModulePanel } from "../../modules/ModulePanel";
import { useModuleStore } from "../../modules/useModuleStore";
import { moduleRegistry } from "../../modules/moduleRegistry";
import { WebGLCanvas } from "./WebGLCanvas";

// Feature flag: set to true to use the WebGL canvas instead of React Flow.
// The WebGL canvas handles 1000+ nodes at 60fps with animated signal flow.
const USE_WEBGL = true;

const nodeTypes = { studioNode: StudioNode };

// ── React Flow inner canvas (fallback) ────────────────────────────────────────

function Canvas() {
  const {
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    connectNodes,
    disconnectEdge,
    setSelectedNode,
    removeNode,
    selectedNodeId,
  } = useEngineStore((s) => ({
    nodes: s.nodes,
    edges: s.edges,
    onNodesChange: s.onNodesChange,
    onEdgesChange: s.onEdgesChange,
    connectNodes: s.connectNodes,
    disconnectEdge: s.disconnectEdge,
    setSelectedNode: s.setSelectedNode,
    removeNode: s.removeNode,
    selectedNodeId: s.selectedNodeId,
  }));
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const { fitView } = useReactFlow();

  useEffect(() => {
    if (nodes.length > 0)
      setTimeout(() => fitView({ padding: 0.15, duration: 400 }), 60);
  }, [nodes.length]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const tag = (e.target as HTMLElement).tagName;
      const isEditable =
        tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
      if ((e.key === "Delete" || e.key === "Backspace") && tag !== "INPUT") {
        if (selectedNodeId) removeNode(selectedNodeId);
      }
      if (e.ctrlKey && !isEditable) {
        if (e.key === "z" || e.key === "Z") {
          e.preventDefault();
          sendIntent?.({ type: "undo" });
        } else if (e.key === "y" || e.key === "Y") {
          e.preventDefault();
          sendIntent?.({ type: "redo" });
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [selectedNodeId, removeNode, sendIntent]);

  const onEdgeClick = useCallback(
    (_: React.MouseEvent, edge: { id: string }) => {
      if (window.confirm("Disconnect this edge?")) disconnectEdge(edge.id);
    },
    [disconnectEdge],
  );

  return (
    <ReactFlow
      nodes={nodes}
      edges={edges}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onConnect={connectNodes}
      onEdgeClick={onEdgeClick}
      onPaneClick={() => setSelectedNode(null)}
      nodeTypes={nodeTypes}
      deleteKeyCode={null}
      fitView
      style={{ background: "#060e18" }}
      defaultEdgeOptions={{
        animated: true,
        style: { stroke: "#38bdf840", strokeWidth: 2 },
      }}
    >
      <Background
        variant={BackgroundVariant.Dots}
        color="#0f1e2e"
        gap={32}
        size={1}
      />
      <Controls
        style={{
          background: "#0c1420",
          border: "1px solid #1a2a3a",
          borderRadius: 8,
        }}
      />
      <MiniMap
        style={{
          background: "#080e18",
          border: "1px solid #0f1e2e",
          borderRadius: 8,
        }}
        nodeColor={(n) => n.data?.color ?? "#1e3a5f"}
        maskColor="rgba(6,14,24,0.85)"
      />
    </ReactFlow>
  );
}

// ── Octave display ────────────────────────────────────────────────────────────

function OctaveDisplay({
  octaveRef,
}: {
  octaveRef: React.MutableRefObject<number>;
}) {
  const [octave, setOctave] = useState(octaveRef.current);
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.code === "KeyZ" || e.code === "KeyX") {
        setTimeout(() => setOctave(octaveRef.current), 0);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [octaveRef]);
  return (
    <span style={{ fontFamily: "'Inter', system-ui, sans-serif" }}>
      Oct: <span style={{ color: "#38bdf8", fontWeight: 700 }}>{octave}</span>
      <span style={{ color: "#1e2d3d" }}>
        {" "}
        (C{octave}={octaveRef.current * 12})
      </span>
    </span>
  );
}

// ── React Flow fallback layout ────────────────────────────────────────────────

function ReactFlowLayout({
  onOpenInstrumentMaker,
}: {
  onOpenInstrumentMaker?: () => void;
}) {
  useEngine();
  const octaveRef = useComputerKeyboard();
  const modules = useModuleStore((s) => s.layout.modules);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        width: "100vw",
        height: "100vh",
        background: "#060e18",
      }}
    >
      <TransportBar onOpenInstrumentMaker={onOpenInstrumentMaker} />
      <ModuleBar />
      <div
        style={{
          padding: "0 16px",
          height: 24,
          background: "#060e18",
          borderBottom: "1px solid #0a1520",
          display: "flex",
          alignItems: "center",
          gap: 16,
          flexShrink: 0,
          fontSize: 10,
          fontFamily: "'Inter', system-ui, sans-serif",
          color: "#1e2d3d",
          userSelect: "none",
        }}
      >
        <span style={{ color: "#334155", fontWeight: 600 }}>⌨ Keyboard</span>
        <span>
          White:{" "}
          <span style={{ color: "#475569", fontFamily: "monospace" }}>
            A S D F G H J K L
          </span>
        </span>
        <span>
          Black:{" "}
          <span style={{ color: "#475569", fontFamily: "monospace" }}>
            W E · T Y U · O P
          </span>
        </span>
        <span>
          Octave:{" "}
          <span style={{ color: "#475569", fontFamily: "monospace" }}>
            Z↓ X↑
          </span>
        </span>
        <OctaveDisplay octaveRef={octaveRef} />
      </div>
      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        <NodePalette />
        <div style={{ flex: 1, position: "relative" }}>
          <ReactFlowProvider>
            <Canvas />
          </ReactFlowProvider>
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

// ── Public export ─────────────────────────────────────────────────────────────

export function StudioCanvas({
  onOpenInstrumentMaker,
}: {
  onOpenInstrumentMaker?: () => void;
}) {
  if (USE_WEBGL) {
    return <WebGLCanvas onOpenInstrumentMaker={onOpenInstrumentMaker} />;
  }
  return <ReactFlowLayout onOpenInstrumentMaker={onOpenInstrumentMaker} />;
}
