/**
 * Lookahead Scheduler Hook
 *
 * Web Audio API-based lookahead scheduler for the step sequencer.
 * Replaces the inaccurate setInterval approach with sample-accurate timing.
 *
 * Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.7, 5.8
 */
import { useRef, useCallback, useState } from "react";

// Constants for lookahead scheduling
const LOOKAHEAD_WINDOW = 0.1; // seconds (100ms)
const SCHEDULER_INTERVAL = 25; // milliseconds

export type StepLength = "1/4" | "1/8" | "1/16";

export interface LookaheadSchedulerOptions {
  bpm: number;
  stepLength: StepLength;
  steps: boolean[][]; // TRACK_COUNT × STEPS grid
  notes: number[]; // one MIDI note per track
  sendIntent: (intent: object) => void;
  audioContext: AudioContext;
}

export interface LookaheadSchedulerHandle {
  start: () => void;
  stop: () => void;
  isPlaying: boolean;
  currentStep: number;
}

/**
 * Calculate step duration in seconds based on BPM and step length
 */
const calculateStepDuration = (bpm: number, stepLength: StepLength): number => {
  const beatDuration = 60 / bpm; // duration of one quarter note in seconds
  switch (stepLength) {
    case "1/4":
      return beatDuration;
    case "1/8":
      return beatDuration / 2;
    case "1/16":
      return beatDuration / 4;
  }
};

/**
 * Lookahead scheduler hook for sample-accurate MIDI timing
 */
export function useLookaheadScheduler(
  options: LookaheadSchedulerOptions,
): LookaheadSchedulerHandle {
  const {
    bpm: _bpm,
    stepLength: _stepLength,
    steps: _steps,
    notes: _notes,
    sendIntent: _sendIntent,
    audioContext,
  } = options;

  // Internal state (useRef to avoid re-render on tick)
  const nextStepTimeRef = useRef<number>(0);
  const currentStepRef = useRef<number>(0);
  const schedulerTimerIdRef = useRef<ReturnType<typeof setTimeout> | null>(
    null,
  );

  // Track playing state for the handle
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentStep, setCurrentStep] = useState(0);

  // Store latest options in refs to avoid stale closures
  const optionsRef = useRef(options);
  optionsRef.current = options;

  /**
   * Schedule a single step: send note-on events for active tracks
   * and schedule corresponding note-offs 80ms later
   */
  const scheduleStep = useCallback((step: number, time: number) => {
    const { steps, notes, sendIntent } = optionsRef.current;
    const trackCount = steps.length;

    for (let track = 0; track < trackCount; track++) {
      if (steps[track][step]) {
        const note = notes[track];

        // Send note-on with scheduled time
        sendIntent({
          type: "inject_midi",
          channel: 0,
          note,
          velocity: 100,
          is_note_on: true,
          scheduled_time: time,
        });

        // Schedule note-off after 80ms
        setTimeout(() => {
          sendIntent({
            type: "inject_midi",
            channel: 0,
            note,
            velocity: 0,
            is_note_on: false,
          });
        }, 80);
      }
    }
  }, []);

  /**
   * Main scheduling loop: schedule all steps within the lookahead window
   */
  const scheduleLoop = useCallback(() => {
    const { bpm, stepLength, audioContext } = optionsRef.current;
    const stepDuration = calculateStepDuration(bpm, stepLength);
    const totalSteps = optionsRef.current.steps[0]?.length ?? 16;

    // Schedule all steps that fall within the lookahead window
    while (
      nextStepTimeRef.current <
      audioContext.currentTime + LOOKAHEAD_WINDOW
    ) {
      const step = currentStepRef.current;

      // Schedule this step
      scheduleStep(step, nextStepTimeRef.current);

      // Update UI state (this will cause a re-render)
      setCurrentStep(step);

      // Advance to next step
      currentStepRef.current = (step + 1) % totalSteps;
      nextStepTimeRef.current += stepDuration;
    }

    // Schedule next iteration of the loop
    schedulerTimerIdRef.current = setTimeout(scheduleLoop, SCHEDULER_INTERVAL);
  }, [scheduleStep]);

  /**
   * Start playback
   */
  const start = useCallback(() => {
    if (isPlaying) return;

    // Initialize timing
    currentStepRef.current = 0;
    nextStepTimeRef.current = audioContext.currentTime;

    setIsPlaying(true);
    setCurrentStep(0);

    // Start the scheduling loop
    scheduleLoop();
  }, [isPlaying, audioContext, scheduleLoop]);

  /**
   * Stop playback
   */
  const stop = useCallback(() => {
    if (!isPlaying) return;

    // Clear pending timeout
    if (schedulerTimerIdRef.current !== null) {
      clearTimeout(schedulerTimerIdRef.current);
      schedulerTimerIdRef.current = null;
    }

    // Reset state
    currentStepRef.current = 0;
    nextStepTimeRef.current = 0;

    setIsPlaying(false);
    setCurrentStep(-1);
  }, [isPlaying]);

  return {
    start,
    stop,
    isPlaying,
    currentStep,
  };
}
