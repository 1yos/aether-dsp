/**
 * Aether Studio v2.0 - Main Application
 * Complete redesigned UI with 4 modes
 */

import { useEffect } from "react";
import { useModeStore } from "./store/useModeStore";
import { TopBar } from "./components/TopBar";
import { ExploreMode } from "./modes/ExploreMode";
import { CreateMode } from "./modes/CreateMode";
import { ArrangeMode } from "./modes/ArrangeMode";
import { PerformMode } from "./modes/PerformMode";
import { useProjectSave } from "./hooks/useProjectSave";
import "./styles/tokens.css";
import "./App.css";

export default function App() {
  const { currentMode, setMode } = useModeStore();

  // Project save/load (Ctrl+S, Ctrl+O, autosave every 2 min)
  useProjectSave();

  // Keyboard shortcuts for mode switching
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Skip if modifier keys are held (Ctrl+S etc handled by useProjectSave)
      if (e.ctrlKey || e.metaKey || e.altKey) return;
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      if (e.key === "1") setMode("explore");
      if (e.key === "2") setMode("create");
      if (e.key === "3") setMode("arrange");
      if (e.key === "4") setMode("perform");
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setMode]);

  const renderMode = () => {
    switch (currentMode) {
      case "explore":
        return <ExploreMode />;
      case "create":
        return <CreateMode />;
      case "arrange":
        return <ArrangeMode />;
      case "perform":
        return <PerformMode />;
      default:
        return <ExploreMode />;
    }
  };

  return (
    <div className="app">
      <TopBar />
      <div className="app-content">{renderMode()}</div>
    </div>
  );
}
