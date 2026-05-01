/**
 * Aether Studio — Create Mode
 * The node graph. Hardware-inspired, alive with signal.
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
