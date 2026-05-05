/**
 * DAW Store — single source of truth for the studio layout and session state.
 *
 * Tracks:
 *   - Active view (song / piano-roll / mixer / patcher)
 *   - Song timeline: tracks, clips, playhead position, BPM, loop region
 *   - Mixer: channel strips
 *   - Browser: open/closed, active tab
 *   - Properties panel: open/closed
 */

import { create } from "zustand";

// ── View ──────────────────────────────────────────────────────────────────────

export type DawView = "song" | "piano-roll" | "mixer" | "patcher";

// ── Song / Timeline ───────────────────────────────────────────────────────────

export type TrackType = "instrument" | "audio" | "bus" | "master";

export interface MidiNote {
  id: string;
  pitch: number; // MIDI 0-127
  beat: number; // start beat (float)
  duration: number; // beats
  velocity: number; // 0-127
}

export interface Clip {
  id: string;
  trackId: string;
  name: string;
  startBeat: number;
  lengthBeats: number;
  color: string;
  notes: MidiNote[]; // for MIDI clips
  audioPath?: string; // for audio clips
}

export interface Track {
  id: string;
  name: string;
  type: TrackType;
  color: string;
  muted: boolean;
  solo: boolean;
  volume: number; // 0-1
  pan: number; // -1 to 1
  clips: Clip[];
  instrumentPresetId?: string; // which .aether-instrument is loaded
  channelId: string; // links to mixer channel
}

// ── Mixer ─────────────────────────────────────────────────────────────────────

export interface MixerChannel {
  id: string;
  name: string;
  color: string;
  volume: number;
  pan: number;
  muted: boolean;
  solo: boolean;
  sends: Array<{ targetId: string; amount: number }>;
}

// ── Transport ─────────────────────────────────────────────────────────────────

export interface Transport {
  bpm: number;
  isPlaying: boolean;
  isRecording: boolean;
  playheadBeat: number;
  loopStart: number;
  loopEnd: number;
  loopEnabled: boolean;
  timeSignatureNum: number;
  timeSignatureDen: number;
}

// ── Store ─────────────────────────────────────────────────────────────────────

interface DawStore {
  // Layout
  activeView: DawView;
  browserOpen: boolean;
  browserTab: "instruments" | "samples" | "presets" | "plugins";
  propertiesOpen: boolean;
  browserWidth: number;
  propertiesHeight: number;

  // Piano roll context (which clip is open)
  pianoRollClipId: string | null;
  pianoRollTrackId: string | null;
  scaleId: string;
  rhythmId: string;

  // Song
  tracks: Track[];
  selectedTrackId: string | null;
  selectedClipId: string | null;

  // Mixer
  channels: MixerChannel[];

  // Transport
  transport: Transport;

  // Actions — layout
  setView: (v: DawView) => void;
  openPianoRoll: (clipId: string, trackId: string) => void;
  setBrowserOpen: (v: boolean) => void;
  setBrowserTab: (t: DawStore["browserTab"]) => void;
  setPropertiesOpen: (v: boolean) => void;
  setBrowserWidth: (w: number) => void;
  setPropertiesHeight: (h: number) => void;
  setScaleId: (id: string) => void;
  setRhythmId: (id: string) => void;

  // Actions — tracks
  addTrack: (type: TrackType, name?: string) => void;
  removeTrack: (id: string) => void;
  updateTrack: (id: string, patch: Partial<Track>) => void;
  selectTrack: (id: string | null) => void;

  // Actions — clips
  addClip: (trackId: string, startBeat: number) => void;
  removeClip: (clipId: string) => void;
  updateClip: (clipId: string, patch: Partial<Clip>) => void;
  selectClip: (id: string | null) => void;
  moveClip: (clipId: string, newStart: number) => void;
  resizeClip: (clipId: string, newLength: number) => void;

  // Actions — notes (inside a clip)
  addNote: (clipId: string, note: Omit<MidiNote, "id">) => void;
  removeNote: (clipId: string, noteId: string) => void;
  updateNote: (
    clipId: string,
    noteId: string,
    patch: Partial<MidiNote>,
  ) => void;

  // Actions — mixer
  updateChannel: (id: string, patch: Partial<MixerChannel>) => void;

  // Actions — transport
  setTransport: (patch: Partial<Transport>) => void;
  play: () => void;
  stop: () => void;
  toggleRecord: () => void;
  setBpm: (bpm: number) => void;
}

const TRACK_COLORS = [
  "#4db8ff",
  "#a78bfa",
  "#34d399",
  "#f97316",
  "#f43f5e",
  "#fbbf24",
  "#06b6d4",
  "#8b5cf6",
];

let trackCounter = 0;
let clipCounter = 0;
let noteCounter = 0;
// channelCounter reserved for future use

function makeId(prefix: string) {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
}

function makeChannel(name: string, color: string): MixerChannel {
  return {
    id: makeId("ch"),
    name,
    color,
    volume: 0.8,
    pan: 0,
    muted: false,
    solo: false,
    sends: [],
  };
}

function makeTrack(type: TrackType, name: string): Track {
  const color = TRACK_COLORS[trackCounter % TRACK_COLORS.length];
  trackCounter++;
  const channelId = makeId("ch");
  return {
    id: makeId("track"),
    name,
    type,
    color,
    muted: false,
    solo: false,
    volume: 0.8,
    pan: 0,
    clips: [],
    channelId,
  };
}

// Default session — 4 tracks to start
const defaultTracks: Track[] = [
  makeTrack("instrument", "Kick"),
  makeTrack("instrument", "Bass"),
  makeTrack("instrument", "Melody"),
  makeTrack("instrument", "Pad"),
];

const defaultChannels: MixerChannel[] = [
  ...defaultTracks.map((t) => ({
    ...makeChannel(t.name, t.color),
    id: t.channelId,
  })),
  { ...makeChannel("Master", "#e0e8f0"), id: "master" },
];

export const useDawStore = create<DawStore>((set, get) => ({
  activeView: "song",
  browserOpen: true,
  browserTab: "instruments",
  propertiesOpen: true,
  browserWidth: 220,
  propertiesHeight: 180,
  pianoRollClipId: null,
  pianoRollTrackId: null,
  scaleId: "12-tet",
  rhythmId: "4-4",
  tracks: defaultTracks,
  selectedTrackId: null,
  selectedClipId: null,
  channels: defaultChannels,
  transport: {
    bpm: 120,
    isPlaying: false,
    isRecording: false,
    playheadBeat: 0,
    loopStart: 0,
    loopEnd: 16,
    loopEnabled: false,
    timeSignatureNum: 4,
    timeSignatureDen: 4,
  },

  setView: (v) => set({ activeView: v }),

  openPianoRoll: (clipId, trackId) =>
    set({
      activeView: "piano-roll",
      pianoRollClipId: clipId,
      pianoRollTrackId: trackId,
    }),

  setBrowserOpen: (v) => set({ browserOpen: v }),
  setBrowserTab: (t) => set({ browserTab: t }),
  setPropertiesOpen: (v) => set({ propertiesOpen: v }),
  setBrowserWidth: (w) =>
    set({ browserWidth: Math.max(160, Math.min(400, w)) }),
  setPropertiesHeight: (h) =>
    set({ propertiesHeight: Math.max(100, Math.min(400, h)) }),
  setScaleId: (id) => set({ scaleId: id }),
  setRhythmId: (id) => set({ rhythmId: id }),

  addTrack: (type, name) => {
    const track = makeTrack(type, name ?? `Track ${get().tracks.length + 1}`);
    const channel = makeChannel(track.name, track.color);
    channel.id = track.channelId;
    set((s) => ({
      tracks: [...s.tracks, track],
      channels: [
        ...s.channels.filter((c) => c.id !== "master"),
        channel,
        s.channels.find((c) => c.id === "master")!,
      ],
    }));
  },

  removeTrack: (id) =>
    set((s) => ({
      tracks: s.tracks.filter((t) => t.id !== id),
      selectedTrackId: s.selectedTrackId === id ? null : s.selectedTrackId,
    })),

  updateTrack: (id, patch) =>
    set((s) => ({
      tracks: s.tracks.map((t) => (t.id === id ? { ...t, ...patch } : t)),
    })),

  selectTrack: (id) => set({ selectedTrackId: id }),

  addClip: (trackId, startBeat) => {
    clipCounter++;
    const track = get().tracks.find((t) => t.id === trackId);
    if (!track) return;
    const clip: Clip = {
      id: makeId("clip"),
      trackId,
      name: `Clip ${clipCounter}`,
      startBeat,
      lengthBeats: 4,
      color: track.color,
      notes: [],
    };
    set((s) => ({
      tracks: s.tracks.map((t) =>
        t.id === trackId ? { ...t, clips: [...t.clips, clip] } : t,
      ),
    }));
  },

  removeClip: (clipId) =>
    set((s) => ({
      tracks: s.tracks.map((t) => ({
        ...t,
        clips: t.clips.filter((c) => c.id !== clipId),
      })),
      selectedClipId: s.selectedClipId === clipId ? null : s.selectedClipId,
    })),

  updateClip: (clipId, patch) =>
    set((s) => ({
      tracks: s.tracks.map((t) => ({
        ...t,
        clips: t.clips.map((c) => (c.id === clipId ? { ...c, ...patch } : c)),
      })),
    })),

  selectClip: (id) => set({ selectedClipId: id }),

  moveClip: (clipId, newStart) =>
    set((s) => ({
      tracks: s.tracks.map((t) => ({
        ...t,
        clips: t.clips.map((c) =>
          c.id === clipId ? { ...c, startBeat: Math.max(0, newStart) } : c,
        ),
      })),
    })),

  resizeClip: (clipId, newLength) =>
    set((s) => ({
      tracks: s.tracks.map((t) => ({
        ...t,
        clips: t.clips.map((c) =>
          c.id === clipId ? { ...c, lengthBeats: Math.max(1, newLength) } : c,
        ),
      })),
    })),

  addNote: (clipId, note) => {
    noteCounter++;
    const newNote: MidiNote = { ...note, id: makeId("note") };
    set((s) => ({
      tracks: s.tracks.map((t) => ({
        ...t,
        clips: t.clips.map((c) =>
          c.id === clipId ? { ...c, notes: [...c.notes, newNote] } : c,
        ),
      })),
    }));
  },

  removeNote: (clipId, noteId) =>
    set((s) => ({
      tracks: s.tracks.map((t) => ({
        ...t,
        clips: t.clips.map((c) =>
          c.id === clipId
            ? { ...c, notes: c.notes.filter((n) => n.id !== noteId) }
            : c,
        ),
      })),
    })),

  updateNote: (clipId, noteId, patch) =>
    set((s) => ({
      tracks: s.tracks.map((t) => ({
        ...t,
        clips: t.clips.map((c) =>
          c.id === clipId
            ? {
                ...c,
                notes: c.notes.map((n) =>
                  n.id === noteId ? { ...n, ...patch } : n,
                ),
              }
            : c,
        ),
      })),
    })),

  updateChannel: (id, patch) =>
    set((s) => ({
      channels: s.channels.map((c) => (c.id === id ? { ...c, ...patch } : c)),
    })),

  setTransport: (patch) =>
    set((s) => ({ transport: { ...s.transport, ...patch } })),

  play: () => set((s) => ({ transport: { ...s.transport, isPlaying: true } })),
  stop: () =>
    set((s) => ({
      transport: { ...s.transport, isPlaying: false, playheadBeat: 0 },
    })),
  toggleRecord: () =>
    set((s) => ({
      transport: { ...s.transport, isRecording: !s.transport.isRecording },
    })),
  setBpm: (bpm) =>
    set((s) => ({
      transport: { ...s.transport, bpm: Math.max(20, Math.min(300, bpm)) },
    })),
}));
