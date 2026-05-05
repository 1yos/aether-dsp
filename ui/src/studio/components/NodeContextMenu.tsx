/**
 * NodeContextMenu — right-click context menu for nodes in the WebGL canvas.
 *
 * Actions:
 * - Set as Output — marks this node as the graph output
 * - Duplicate — adds a new node of the same type with the same params
 * - Delete — removes the node
 * - Disconnect All — removes all edges connected to this node
 */

import { useCallback, useEffect, useRef } from "react";
import { useEngineStore } from "../store/engineStore";

interface NodeContextMenuProps {
  nodeId: string;
  nodeType: string;
  generation: number;
  x: number; // screen x
  y: number; // screen y
  onClose: () => void;
}

export function NodeContextMenu({
  nodeId,
  nodeType,
  generation,
  x,
  y,
  onClose,
}: NodeContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const removeNode = useEngineStore((s) => s.removeNode);
  const addNode = useEngineStore((s) => s.addNode);
  const edges = useEngineStore((s) => s.edges);
  const setOutputNode = useEngineStore((s) => s.setOutputNode);

  // Close on outside click or Escape
  useEffect(() => {
    const handler = (e: MouseEvent | KeyboardEvent) => {
      if (e instanceof KeyboardEvent && e.key === "Escape") {
        onClose();
        return;
      }
      if (
        e instanceof MouseEvent &&
        menuRef.current &&
        !menuRef.current.contains(e.target as Node)
      ) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handler);
    document.addEventListener("keydown", handler);
    return () => {
      document.removeEventListener("mousedown", handler);
      document.removeEventListener("keydown", handler);
    };
  }, [onClose]);

  const handleSetOutput = useCallback(() => {
    setOutputNode(nodeId, generation);
    onClose();
  }, [nodeId, generation, setOutputNode, onClose]);

  const handleDuplicate = useCallback(() => {
    addNode(nodeType);
    onClose();
  }, [nodeType, addNode, onClose]);

  const handleDelete = useCallback(() => {
    removeNode(nodeId);
    onClose();
  }, [nodeId, removeNode, onClose]);

  const handleDisconnectAll = useCallback(() => {
    // Disconnect all edges where this node is source or target
    const nodeEdges = edges.filter(
      (e) => e.source === nodeId || e.target === nodeId,
    );
    for (const edge of nodeEdges) {
      const slot = parseInt(
        edge.targetHandle?.replace("input-", "") ?? "0",
        10,
      );
      sendIntent?.({
        type: "disconnect",
        dst_id: parseInt(edge.target, 10),
        dst_gen: 0,
        slot,
      });
    }
    onClose();
  }, [edges, nodeId, sendIntent, onClose]);

  const bg = "#0c1420";
  const border = "#1a2a3a";
  const text = "#e0e8f0";
  const dim = "#4a6a8a";
  const hover = "#0f1e2e";

  const items = [
    {
      label: "Set as Output",
      icon: "◉",
      action: handleSetOutput,
      color: "#00e5a0",
    },
    { label: "Duplicate", icon: "⧉", action: handleDuplicate, color: text },
    {
      label: "Disconnect All",
      icon: "⊘",
      action: handleDisconnectAll,
      color: "#ffd54f",
    },
    { label: "Delete", icon: "✕", action: handleDelete, color: "#ef5350" },
  ];

  return (
    <div
      ref={menuRef}
      style={{
        position: "fixed",
        left: x,
        top: y,
        zIndex: 3000,
        background: bg,
        border: `1px solid ${border}`,
        borderRadius: 8,
        boxShadow: "0 8px 32px rgba(0,0,0,0.6)",
        minWidth: 180,
        overflow: "hidden",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "8px 12px 6px",
          borderBottom: `1px solid ${border}`,
          fontSize: 10,
          color: dim,
          fontFamily: "monospace",
          letterSpacing: "0.06em",
        }}
      >
        {nodeType
          .replace(/([A-Z])/g, " $1")
          .trim()
          .toUpperCase()}
      </div>

      {/* Items */}
      {items.map(({ label, icon, action, color }) => (
        <button
          key={label}
          onClick={action}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 10,
            width: "100%",
            padding: "9px 14px",
            background: "transparent",
            border: "none",
            color,
            fontSize: 12,
            cursor: "pointer",
            textAlign: "left",
            transition: "background 0.1s",
          }}
          onMouseEnter={(e) => (e.currentTarget.style.background = hover)}
          onMouseLeave={(e) =>
            (e.currentTarget.style.background = "transparent")
          }
        >
          <span style={{ fontSize: 13, width: 16, textAlign: "center" }}>
            {icon}
          </span>
          <span>{label}</span>
        </button>
      ))}
    </div>
  );
}
