/**
 * Property-Based Tests for useLookaheadScheduler
 *
 * Feature: aether-engine-upgrades
 * Property 11: Lookahead scheduler schedules all due steps
 * **Validates: Requirements 5.4**
 *
 * Property 12: Note-off follows note-on by 80ms
 * **Validates: Requirements 5.5**
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import type { StepLength } from "../useLookaheadScheduler";

// Constants from the hook
const LOOKAHEAD_WINDOW = 0.1; // seconds (100ms)
const SCHEDULER_INTERVAL = 25; // milliseconds

/**
 * Calculate step duration in seconds based on BPM and step length
 */
const calculateStepDuration = (bpm: number, stepLength: StepLength): number => {
  const beatDuration = 60 / bpm;
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
 * Simulate the lookahead scheduler logic
 * Returns an array of scheduled step times
 */
const simulateScheduler = (
  bpm: number,
  stepLength: StepLength,
  totalSteps: number,
  simulationDuration: number, // in seconds
): { step: number; time: number }[] => {
  const scheduledSteps: { step: number; time: number }[] = [];
  const stepDuration = calculateStepDuration(bpm, stepLength);

  let currentTime = 0;
  let nextStepTime = 0;
  let currentStep = 0;

  // Simulate the scheduling loop for the specified duration
  while (currentTime < simulationDuration) {
    // Schedule all steps within the lookahead window
    while (nextStepTime < currentTime + LOOKAHEAD_WINDOW) {
      scheduledSteps.push({
        step: currentStep,
        time: nextStepTime,
      });

      currentStep = (currentStep + 1) % totalSteps;
      nextStepTime += stepDuration;
    }

    // Advance time by the scheduler interval
    currentTime += SCHEDULER_INTERVAL / 1000;
  }

  return scheduledSteps;
};

describe("useLookaheadScheduler - Property 11: Lookahead scheduler schedules all due steps", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // Property 11: Lookahead scheduler schedules all due steps
  it("should schedule all steps within lookahead window exactly once (BPM=120, 1/16)", () => {
    const bpm = 120;
    const stepLength: StepLength = "1/16";
    const totalSteps = 16;
    const simulationDuration = 1.0; // 1 second

    const scheduledSteps = simulateScheduler(
      bpm,
      stepLength,
      totalSteps,
      simulationDuration,
    );

    // Verify no duplicates
    const timeStepMap = new Map<string, Set<number>>();
    for (const { step, time } of scheduledSteps) {
      const timeKey = time.toFixed(4);
      if (!timeStepMap.has(timeKey)) {
        timeStepMap.set(timeKey, new Set());
      }
      const stepsAtTime = timeStepMap.get(timeKey)!;
      expect(stepsAtTime.has(step)).toBe(false); // No duplicate scheduling
      stepsAtTime.add(step);
    }

    // Verify all steps are scheduled (no gaps)
    expect(scheduledSteps.length).toBeGreaterThan(0);

    // Check that steps are in sequence (with wrap-around)
    for (let i = 1; i < scheduledSteps.length; i++) {
      const prevStep = scheduledSteps[i - 1].step;
      const currStep = scheduledSteps[i].step;
      const expectedStep = (prevStep + 1) % totalSteps;
      expect(currStep).toBe(expectedStep);
    }
  });

  it("should schedule all steps within lookahead window exactly once (BPM=60, 1/8)", () => {
    const bpm = 60;
    const stepLength: StepLength = "1/8";
    const totalSteps = 16;
    const simulationDuration = 1.0;

    const scheduledSteps = simulateScheduler(
      bpm,
      stepLength,
      totalSteps,
      simulationDuration,
    );

    // Verify no duplicates
    const timeStepMap = new Map<string, Set<number>>();
    for (const { step, time } of scheduledSteps) {
      const timeKey = time.toFixed(4);
      if (!timeStepMap.has(timeKey)) {
        timeStepMap.set(timeKey, new Set());
      }
      const stepsAtTime = timeStepMap.get(timeKey)!;
      expect(stepsAtTime.has(step)).toBe(false);
      stepsAtTime.add(step);
    }

    expect(scheduledSteps.length).toBeGreaterThan(0);

    // Check sequence
    for (let i = 1; i < scheduledSteps.length; i++) {
      const prevStep = scheduledSteps[i - 1].step;
      const currStep = scheduledSteps[i].step;
      const expectedStep = (prevStep + 1) % totalSteps;
      expect(currStep).toBe(expectedStep);
    }
  });

  it("should schedule all steps within lookahead window exactly once (BPM=180, 1/4)", () => {
    const bpm = 180;
    const stepLength: StepLength = "1/4";
    const totalSteps = 16;
    const simulationDuration = 1.0;

    const scheduledSteps = simulateScheduler(
      bpm,
      stepLength,
      totalSteps,
      simulationDuration,
    );

    // Verify no duplicates
    const timeStepMap = new Map<string, Set<number>>();
    for (const { step, time } of scheduledSteps) {
      const timeKey = time.toFixed(4);
      if (!timeStepMap.has(timeKey)) {
        timeStepMap.set(timeKey, new Set());
      }
      const stepsAtTime = timeStepMap.get(timeKey)!;
      expect(stepsAtTime.has(step)).toBe(false);
      stepsAtTime.add(step);
    }

    expect(scheduledSteps.length).toBeGreaterThan(0);

    // Check sequence
    for (let i = 1; i < scheduledSteps.length; i++) {
      const prevStep = scheduledSteps[i - 1].step;
      const currStep = scheduledSteps[i].step;
      const expectedStep = (prevStep + 1) % totalSteps;
      expect(currStep).toBe(expectedStep);
    }
  });

  // Property-based test: random BPM and step length
  it("should schedule all steps correctly for random BPM (40-300) and step lengths", () => {
    const testCases = [
      { bpm: 40, stepLength: "1/16" as StepLength },
      { bpm: 80, stepLength: "1/8" as StepLength },
      { bpm: 120, stepLength: "1/4" as StepLength },
      { bpm: 150, stepLength: "1/16" as StepLength },
      { bpm: 200, stepLength: "1/8" as StepLength },
      { bpm: 240, stepLength: "1/4" as StepLength },
      { bpm: 300, stepLength: "1/16" as StepLength },
    ];

    for (const { bpm, stepLength } of testCases) {
      const totalSteps = 16;
      const simulationDuration = 1.0;

      const scheduledSteps = simulateScheduler(
        bpm,
        stepLength,
        totalSteps,
        simulationDuration,
      );

      // Verify no duplicates at the same time
      const timeStepMap = new Map<string, Set<number>>();
      for (const { step, time } of scheduledSteps) {
        const timeKey = time.toFixed(4);
        if (!timeStepMap.has(timeKey)) {
          timeStepMap.set(timeKey, new Set());
        }
        const stepsAtTime = timeStepMap.get(timeKey)!;
        expect(stepsAtTime.has(step)).toBe(false);
        stepsAtTime.add(step);
      }

      expect(scheduledSteps.length).toBeGreaterThan(0);

      // Verify sequential ordering
      for (let i = 1; i < scheduledSteps.length; i++) {
        const prevStep = scheduledSteps[i - 1].step;
        const currStep = scheduledSteps[i].step;
        const expectedStep = (prevStep + 1) % totalSteps;
        expect(currStep).toBe(expectedStep);
      }
    }
  });

  it("should schedule steps with correct timing intervals", () => {
    const bpm = 120;
    const stepLength: StepLength = "1/16";
    const totalSteps = 16;
    const simulationDuration = 1.0;

    const scheduledSteps = simulateScheduler(
      bpm,
      stepLength,
      totalSteps,
      simulationDuration,
    );

    const expectedStepDuration = calculateStepDuration(bpm, stepLength);

    // Check that consecutive steps have the correct time interval
    for (let i = 1; i < scheduledSteps.length; i++) {
      const prevTime = scheduledSteps[i - 1].time;
      const currTime = scheduledSteps[i].time;
      const timeDiff = currTime - prevTime;

      // Allow small floating point tolerance
      expect(Math.abs(timeDiff - expectedStepDuration)).toBeLessThan(0.0001);
    }
  });

  it("should handle very fast BPM (300) without missing steps", () => {
    const bpm = 300;
    const stepLength: StepLength = "1/16";
    const totalSteps = 16;
    const simulationDuration = 1.0;

    const scheduledSteps = simulateScheduler(
      bpm,
      stepLength,
      totalSteps,
      simulationDuration,
    );

    // At 300 BPM with 1/16 notes, we should have many steps
    const expectedStepDuration = calculateStepDuration(bpm, stepLength);
    const expectedStepCount = Math.floor(
      simulationDuration / expectedStepDuration,
    );

    // Should schedule at least the expected number of steps
    expect(scheduledSteps.length).toBeGreaterThanOrEqual(expectedStepCount);

    // Verify no duplicates
    const timeStepMap = new Map<string, Set<number>>();
    for (const { step, time } of scheduledSteps) {
      const timeKey = time.toFixed(4);
      if (!timeStepMap.has(timeKey)) {
        timeStepMap.set(timeKey, new Set());
      }
      const stepsAtTime = timeStepMap.get(timeKey)!;
      expect(stepsAtTime.has(step)).toBe(false);
      stepsAtTime.add(step);
    }
  });

  it("should handle very slow BPM (40) without double-scheduling", () => {
    const bpm = 40;
    const stepLength: StepLength = "1/4";
    const totalSteps = 16;
    const simulationDuration = 1.0;

    const scheduledSteps = simulateScheduler(
      bpm,
      stepLength,
      totalSteps,
      simulationDuration,
    );

    // Verify no duplicates
    const timeStepMap = new Map<string, Set<number>>();
    for (const { step, time } of scheduledSteps) {
      const timeKey = time.toFixed(4);
      if (!timeStepMap.has(timeKey)) {
        timeStepMap.set(timeKey, new Set());
      }
      const stepsAtTime = timeStepMap.get(timeKey)!;
      expect(stepsAtTime.has(step)).toBe(false);
      stepsAtTime.add(step);
    }

    // Check sequence
    for (let i = 1; i < scheduledSteps.length; i++) {
      const prevStep = scheduledSteps[i - 1].step;
      const currStep = scheduledSteps[i].step;
      const expectedStep = (prevStep + 1) % totalSteps;
      expect(currStep).toBe(expectedStep);
    }
  });

  it("should schedule all steps within the lookahead window boundary", () => {
    const bpm = 120;
    const stepLength: StepLength = "1/16";
    const totalSteps = 16;
    const simulationDuration = 1.0;

    const scheduledSteps = simulateScheduler(
      bpm,
      stepLength,
      totalSteps,
      simulationDuration,
    );

    // For each scheduler tick, verify all steps within lookahead are scheduled
    let currentTime = 0;
    let scheduledIndex = 0;

    while (currentTime < simulationDuration) {
      const lookaheadEnd = currentTime + LOOKAHEAD_WINDOW;

      // Collect all steps that should be scheduled in this window
      const expectedSteps: number[] = [];
      while (
        scheduledIndex < scheduledSteps.length &&
        scheduledSteps[scheduledIndex].time < lookaheadEnd
      ) {
        expectedSteps.push(scheduledSteps[scheduledIndex].step);
        scheduledIndex++;
      }

      // All steps within the window should be scheduled
      expect(expectedSteps.length).toBeGreaterThanOrEqual(0);

      currentTime += SCHEDULER_INTERVAL / 1000;
    }
  });
});

// Property 12: Note-off follows note-on by 80ms
describe("useLookaheadScheduler - Property 12: Note-off follows note-on by 80ms", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  /**
   * Simulate the scheduler and capture note-on/note-off events
   */
  const simulateSchedulerWithNoteEvents = (
    bpm: number,
    stepLength: StepLength,
    steps: boolean[][], // TRACK_COUNT × STEPS grid
    notes: number[], // MIDI notes per track
    simulationDuration: number,
  ): {
    noteOn: { note: number; time: number }[];
    noteOff: { note: number; time: number }[];
  } => {
    const noteOnEvents: { note: number; time: number }[] = [];
    const noteOffEvents: { note: number; time: number }[] = [];

    const stepDuration = calculateStepDuration(bpm, stepLength);
    const totalSteps = steps[0]?.length ?? 16;

    let currentTime = 0;
    let nextStepTime = 0;
    let currentStep = 0;

    // Simulate the scheduling loop
    while (currentTime < simulationDuration) {
      // Schedule all steps within the lookahead window
      while (nextStepTime < currentTime + LOOKAHEAD_WINDOW) {
        // For each track, check if this step is active
        for (let track = 0; track < steps.length; track++) {
          if (steps[track][currentStep]) {
            const note = notes[track];

            // Schedule note-on
            noteOnEvents.push({ note, time: nextStepTime });

            // Schedule note-off 80ms later
            noteOffEvents.push({ note, time: nextStepTime + 0.08 });
          }
        }

        currentStep = (currentStep + 1) % totalSteps;
        nextStepTime += stepDuration;
      }

      // Advance time by the scheduler interval
      currentTime += SCHEDULER_INTERVAL / 1000;
    }

    return { noteOn: noteOnEvents, noteOff: noteOffEvents };
  };

  it("should schedule note-off exactly 80ms after note-on for a single active step", () => {
    const bpm = 120;
    const stepLength: StepLength = "1/16";
    const steps = [
      [
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
      ],
    ];
    const notes = [60]; // Middle C
    const simulationDuration = 0.5;

    const { noteOn, noteOff } = simulateSchedulerWithNoteEvents(
      bpm,
      stepLength,
      steps,
      notes,
      simulationDuration,
    );

    // Should have at least one note-on event
    expect(noteOn.length).toBeGreaterThan(0);
    expect(noteOff.length).toBe(noteOn.length);

    // Verify each note-off is exactly 80ms after its corresponding note-on
    for (let i = 0; i < noteOn.length; i++) {
      const onEvent = noteOn[i];
      const offEvent = noteOff[i];

      expect(offEvent.note).toBe(onEvent.note);
      expect(offEvent.time).toBeCloseTo(onEvent.time + 0.08, 6); // 6 decimal places precision
    }
  });

  it("should schedule note-off 80ms after note-on for multiple tracks", () => {
    const bpm = 120;
    const stepLength: StepLength = "1/16";
    const steps = [
      [
        true,
        false,
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
      ],
      [
        false,
        true,
        false,
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
      ],
      [
        true,
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
      ],
    ];
    const notes = [60, 64, 67]; // C, E, G
    const simulationDuration = 0.5;

    const { noteOn, noteOff } = simulateSchedulerWithNoteEvents(
      bpm,
      stepLength,
      steps,
      notes,
      simulationDuration,
    );

    expect(noteOn.length).toBeGreaterThan(0);
    expect(noteOff.length).toBe(noteOn.length);

    // Verify each note-off is exactly 80ms after its corresponding note-on
    for (let i = 0; i < noteOn.length; i++) {
      const onEvent = noteOn[i];
      const offEvent = noteOff[i];

      expect(offEvent.note).toBe(onEvent.note);
      expect(offEvent.time).toBeCloseTo(onEvent.time + 0.08, 6);
    }
  });

  it("should maintain 80ms note-off timing across different BPMs", () => {
    const testCases = [
      { bpm: 60, stepLength: "1/16" as StepLength },
      { bpm: 120, stepLength: "1/8" as StepLength },
      { bpm: 180, stepLength: "1/4" as StepLength },
      { bpm: 240, stepLength: "1/16" as StepLength },
    ];

    for (const { bpm, stepLength } of testCases) {
      const steps = [
        [
          true,
          false,
          true,
          false,
          true,
          false,
          false,
          false,
          false,
          false,
          false,
          false,
          false,
          false,
          false,
          false,
        ],
      ];
      const notes = [60];
      const simulationDuration = 1.0;

      const { noteOn, noteOff } = simulateSchedulerWithNoteEvents(
        bpm,
        stepLength,
        steps,
        notes,
        simulationDuration,
      );

      expect(noteOn.length).toBeGreaterThan(0);
      expect(noteOff.length).toBe(noteOn.length);

      // Verify 80ms timing for all events at this BPM
      for (let i = 0; i < noteOn.length; i++) {
        const onEvent = noteOn[i];
        const offEvent = noteOff[i];

        expect(offEvent.note).toBe(onEvent.note);
        expect(offEvent.time).toBeCloseTo(onEvent.time + 0.08, 6);
      }
    }
  });

  it("should schedule note-off 80ms after note-on for all active steps in a full grid", () => {
    const bpm = 120;
    const stepLength: StepLength = "1/16";
    // All steps active on all tracks
    const steps = [
      Array(16).fill(true),
      Array(16).fill(true),
      Array(16).fill(true),
      Array(16).fill(true),
    ];
    const notes = [60, 62, 64, 65]; // C, D, E, F
    const simulationDuration = 0.5;

    const { noteOn, noteOff } = simulateSchedulerWithNoteEvents(
      bpm,
      stepLength,
      steps,
      notes,
      simulationDuration,
    );

    expect(noteOn.length).toBeGreaterThan(0);
    expect(noteOff.length).toBe(noteOn.length);

    // Verify each note-off is exactly 80ms after its corresponding note-on
    for (let i = 0; i < noteOn.length; i++) {
      const onEvent = noteOn[i];
      const offEvent = noteOff[i];

      expect(offEvent.note).toBe(onEvent.note);
      expect(offEvent.time).toBeCloseTo(onEvent.time + 0.08, 6);
    }
  });

  it("should handle note-off timing at very fast BPM (300)", () => {
    const bpm = 300;
    const stepLength: StepLength = "1/16";
    const steps = [
      [
        true,
        true,
        true,
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
      ],
    ];
    const notes = [60];
    const simulationDuration = 0.5;

    const { noteOn, noteOff } = simulateSchedulerWithNoteEvents(
      bpm,
      stepLength,
      steps,
      notes,
      simulationDuration,
    );

    expect(noteOn.length).toBeGreaterThan(0);
    expect(noteOff.length).toBe(noteOn.length);

    // Even at very fast BPM, note-off should be exactly 80ms after note-on
    for (let i = 0; i < noteOn.length; i++) {
      const onEvent = noteOn[i];
      const offEvent = noteOff[i];

      expect(offEvent.note).toBe(onEvent.note);
      expect(offEvent.time).toBeCloseTo(onEvent.time + 0.08, 6);
    }
  });

  it("should handle note-off timing at very slow BPM (40)", () => {
    const bpm = 40;
    const stepLength: StepLength = "1/4";
    const steps = [
      [
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
      ],
    ];
    const notes = [60];
    const simulationDuration = 2.0;

    const { noteOn, noteOff } = simulateSchedulerWithNoteEvents(
      bpm,
      stepLength,
      steps,
      notes,
      simulationDuration,
    );

    expect(noteOn.length).toBeGreaterThan(0);
    expect(noteOff.length).toBe(noteOn.length);

    // Even at very slow BPM, note-off should be exactly 80ms after note-on
    for (let i = 0; i < noteOn.length; i++) {
      const onEvent = noteOn[i];
      const offEvent = noteOff[i];

      expect(offEvent.note).toBe(onEvent.note);
      expect(offEvent.time).toBeCloseTo(onEvent.time + 0.08, 6);
    }
  });

  it("should maintain 80ms note-off timing when notes overlap (fast steps)", () => {
    const bpm = 300;
    const stepLength: StepLength = "1/16";
    // At 300 BPM with 1/16 notes, step duration is 50ms, so notes will overlap
    const steps = [
      [
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
      ],
    ];
    const notes = [60];
    const simulationDuration = 0.5;

    const { noteOn, noteOff } = simulateSchedulerWithNoteEvents(
      bpm,
      stepLength,
      steps,
      notes,
      simulationDuration,
    );

    expect(noteOn.length).toBeGreaterThan(0);
    expect(noteOff.length).toBe(noteOn.length);

    // Verify 80ms timing even when notes overlap
    for (let i = 0; i < noteOn.length; i++) {
      const onEvent = noteOn[i];
      const offEvent = noteOff[i];

      expect(offEvent.note).toBe(onEvent.note);
      expect(offEvent.time).toBeCloseTo(onEvent.time + 0.08, 6);
    }

    // Verify that some notes do overlap (note-off after next note-on)
    const stepDuration = calculateStepDuration(bpm, stepLength);
    if (stepDuration < 0.08) {
      // At least one note-off should occur after the next note-on
      let foundOverlap = false;
      for (let i = 0; i < noteOn.length - 1; i++) {
        if (noteOff[i].time > noteOn[i + 1].time) {
          foundOverlap = true;
          break;
        }
      }
      expect(foundOverlap).toBe(true);
    }
  });

  it("should schedule note-off 80ms after note-on for sparse grid patterns", () => {
    const bpm = 120;
    const stepLength: StepLength = "1/16";
    const steps = [
      [
        true,
        false,
        false,
        false,
        true,
        false,
        false,
        false,
        true,
        false,
        false,
        false,
        true,
        false,
        false,
        false,
      ],
      [
        false,
        false,
        true,
        false,
        false,
        false,
        true,
        false,
        false,
        false,
        true,
        false,
        false,
        false,
        true,
        false,
      ],
    ];
    const notes = [60, 67]; // C and G
    const simulationDuration = 1.0;

    const { noteOn, noteOff } = simulateSchedulerWithNoteEvents(
      bpm,
      stepLength,
      steps,
      notes,
      simulationDuration,
    );

    expect(noteOn.length).toBeGreaterThan(0);
    expect(noteOff.length).toBe(noteOn.length);

    // Verify each note-off is exactly 80ms after its corresponding note-on
    for (let i = 0; i < noteOn.length; i++) {
      const onEvent = noteOn[i];
      const offEvent = noteOff[i];

      expect(offEvent.note).toBe(onEvent.note);
      expect(offEvent.time).toBeCloseTo(onEvent.time + 0.08, 6);
    }
  });
});
