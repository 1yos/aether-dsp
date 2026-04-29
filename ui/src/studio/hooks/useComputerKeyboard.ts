/**
 * useComputerKeyboard — maps PC keyboard keys to MIDI notes.
 *
 * Layout (two rows, like a piano):
 *
 *   Black keys:  W  E     T  Y  U     O  P
 *   White keys:  A  S  D  F  G  H  J  K  L  ;
 *
 * This maps to one octave starting at C (default octave 4 = C4 = MIDI 60):
 *   A=C  W=C#  S=D  E=D#  D=E  F=F  T=F#  G=G  Y=G#  H=A  U=A#  J=B  K=C5  O=C#5  L=D5  P=D#5
 *
 * Z = octave down, X = octave up
 * Velocity is fixed at 100 (can be extended with mouse wheel later).
 *
 * Sends inject_midi intents to the host via engineStore.sendIntent.
 * Only fires when the focused element is NOT an input/textarea/select.
 */
import { useEffect, useRef } from "react";
import { useEngineStore } from "../store/engineStore";

// Maps key code → semitone offset from C in the current octave
const KEY_TO_SEMITONE: Record<string, number> = {
  KeyA: 0, // C
  KeyW: 1, // C#
  KeyS: 2, // D
  KeyE: 3, // D#
  KeyD: 4, // E
  KeyF: 5, // F
  KeyT: 6, // F#
  KeyG: 7, // G
  KeyY: 8, // G#
  KeyH: 9, // A
  KeyU: 10, // A#
  KeyJ: 11, // B
  KeyK: 12, // C (next octave)
  KeyO: 13, // C#
  KeyL: 14, // D
  KeyP: 15, // D#
};

function isTypingTarget(e: KeyboardEvent): boolean {
  const tag = (e.target as HTMLElement).tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
}

export function useComputerKeyboard() {
  const sendIntent = useEngineStore((s) => s.sendIntent);
  const octaveRef = useRef(4); // default octave 4 → C4 = MIDI 60
  const pressedRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.repeat) return;
      if (isTypingTarget(e)) return;

      // Octave shift
      if (e.code === "KeyZ") {
        octaveRef.current = Math.max(0, octaveRef.current - 1);
        return;
      }
      if (e.code === "KeyX") {
        octaveRef.current = Math.min(9, octaveRef.current + 1);
        return;
      }

      const semitone = KEY_TO_SEMITONE[e.code];
      if (semitone === undefined) return;
      if (pressedRef.current.has(e.code)) return; // already held

      pressedRef.current.add(e.code);
      const note = octaveRef.current * 12 + semitone;
      if (note < 0 || note > 127) return;

      sendIntent?.({
        type: "inject_midi",
        channel: 0,
        note,
        velocity: 100,
        is_note_on: true,
      });
    };

    const onKeyUp = (e: KeyboardEvent) => {
      if (isTypingTarget(e)) return;

      const semitone = KEY_TO_SEMITONE[e.code];
      if (semitone === undefined) return;
      if (!pressedRef.current.has(e.code)) return;

      pressedRef.current.delete(e.code);
      const note = octaveRef.current * 12 + semitone;
      if (note < 0 || note > 127) return;

      sendIntent?.({
        type: "inject_midi",
        channel: 0,
        note,
        velocity: 0,
        is_note_on: false,
      });
    };

    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    };
  }, [sendIntent]);

  return octaveRef;
}
