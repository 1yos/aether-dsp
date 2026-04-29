/**
 * Aether Studio v2.0 - Create Mode
 * Build audio patches with node graph
 */

import { StudioCanvas } from "../studio/components/StudioCanvas";
import "./CreateMode.css";

export function CreateMode() {
  return (
    <div className="create-mode">
      <StudioCanvas />
    </div>
  );
}
