/**
 * useAudioPreview — plays back a sample zone at a given MIDI note pitch.
 * Uses the Web Audio API for in-browser preview.
 */
import { useRef, useCallback } from "react";
import { SampleZone, SamplerInstrument } from "../types";

export function useAudioPreview() {
  const ctxRef = useRef<AudioContext | null>(null);

  function getCtx(): AudioContext {
    if (!ctxRef.current || ctxRef.current.state === "closed") {
      ctxRef.current = new AudioContext();
    }
    return ctxRef.current;
  }

  const previewNote = useCallback(
    async (instrument: SamplerInstrument, note: number, velocity = 0.8) => {
      const zone = findZone(instrument, note, Math.round(velocity * 127));
      if (!zone || !zone.audioBuffer) return;

      const ctx = getCtx();
      if (ctx.state === "suspended") await ctx.resume();

      const source = ctx.createBufferSource();
      source.buffer = zone.audioBuffer;

      // Pitch shift: ratio between target note and root note
      const rootFreq = instrument.tuning.frequencies[zone.root_note] ?? 440;
      const targetFreq = instrument.tuning.frequencies[note] ?? 440;
      const ratio = rootFreq > 0 ? targetFreq / rootFreq : 1;
      source.playbackRate.value = ratio * Math.pow(2, zone.tune_cents / 1200);

      // ADSR via gain node
      const gainNode = ctx.createGain();
      const now = ctx.currentTime;
      gainNode.gain.setValueAtTime(0, now);
      gainNode.gain.linearRampToValueAtTime(velocity, now + instrument.attack);
      gainNode.gain.linearRampToValueAtTime(
        velocity * instrument.sustain,
        now + instrument.attack + instrument.decay,
      );

      source.connect(gainNode);
      gainNode.connect(ctx.destination);
      source.start(now);

      // Auto-stop after 2 seconds for preview
      const stopTime = now + Math.min(2, zone.audioBuffer.duration / ratio);
      gainNode.gain.setValueAtTime(velocity * instrument.sustain, stopTime);
      gainNode.gain.linearRampToValueAtTime(0, stopTime + instrument.release);
      source.stop(stopTime + instrument.release + 0.05);
    },
    [],
  );

  return { previewNote };
}

function findZone(
  instrument: SamplerInstrument,
  note: number,
  velocity: number,
): SampleZone | null {
  let best: SampleZone | null = null;
  let bestDist = 255;
  for (const zone of instrument.zones) {
    if (
      note >= zone.note_low &&
      note <= zone.note_high &&
      velocity >= zone.velocity_low &&
      velocity <= zone.velocity_high
    ) {
      const dist = Math.abs(note - zone.root_note);
      if (dist < bestDist) {
        bestDist = dist;
        best = zone;
      }
    }
  }
  return best;
}

/**
 * Load an audio file into a Web Audio AudioBuffer.
 */
export async function loadAudioBuffer(file: File): Promise<AudioBuffer> {
  const ctx = new AudioContext();
  const arrayBuffer = await file.arrayBuffer();
  const audioBuffer = await ctx.decodeAudioData(arrayBuffer);
  await ctx.close();
  return audioBuffer;
}
