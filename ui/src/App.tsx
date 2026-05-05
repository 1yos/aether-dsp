/**
 * Aether Studio v3.0 — Real DAW
 *
 * Rebuilt around the standard DAW workflow:
 *   Song View → Piano Roll → Mixer → Patcher (node graph)
 *
 * The world music catalog and microtonal features are integrated
 * into the browser and piano roll, not a separate "Explore" mode.
 */

import { DawShell } from "./daw/DawShell";
import "./styles/tokens.css";

export default function App() {
  return <DawShell />;
}
