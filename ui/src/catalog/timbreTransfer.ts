import type { TimbreProfile } from "./types";

/**
 * Applies a TimbreProfile to a source AudioBuffer using Web Audio API EQ shaping.
 * Returns a new AudioBuffer with the timbre characteristics of the target instrument.
 */
export async function applyTimbreProfile(
  source: AudioBuffer,
  profile: TimbreProfile,
  _audioContext: AudioContext,
): Promise<AudioBuffer> {
  const dur = source.duration;
  const sr = source.sampleRate;
  const frames = source.length;
  const offline = new OfflineAudioContext(1, frames, sr);

  // Source
  const src = offline.createBufferSource();
  // Convert to mono if needed
  const mono = offline.createBuffer(1, frames, sr);
  const monoData = mono.getChannelData(0);
  for (let ch = 0; ch < source.numberOfChannels; ch++) {
    const chData = source.getChannelData(ch);
    for (let i = 0; i < frames; i++)
      monoData[i] += chData[i] / source.numberOfChannels;
  }
  src.buffer = mono;

  // Brightness — high shelf
  const highShelf = offline.createBiquadFilter();
  highShelf.type = "highshelf";
  highShelf.frequency.value = profile.spectral_centroid_hz;
  // brightness > 0.5 boosts highs, < 0.5 cuts
  highShelf.gain.value = (profile.brightness - 0.5) * 20;

  // Warmth — low shelf
  const lowShelf = offline.createBiquadFilter();
  lowShelf.type = "lowshelf";
  lowShelf.frequency.value = 400;
  lowShelf.gain.value = (profile.warmth - 0.5) * 16;

  // Formant peaks — peaking EQ bands
  const formantFilters = profile.formant_peaks.slice(0, 3).map((freq) => {
    const f = offline.createBiquadFilter();
    f.type = "peaking";
    f.frequency.value = freq;
    f.Q.value = 4;
    f.gain.value = profile.harmonic_richness * 8;
    return f;
  });

  // Output gain
  const outGain = offline.createGain();
  outGain.gain.value = 0.8;

  // Apply envelope
  const { attack, decay, sustain, release } = profile.envelope;
  outGain.gain.setValueAtTime(0, 0);
  outGain.gain.linearRampToValueAtTime(1, attack);
  outGain.gain.linearRampToValueAtTime(sustain, attack + decay);
  const sustainEnd = Math.max(attack + decay, dur - release);
  outGain.gain.setValueAtTime(sustain, sustainEnd);
  outGain.gain.linearRampToValueAtTime(0, dur);

  // Chain: src → highShelf → lowShelf → formant[0] → formant[1] → formant[2] → outGain → dest
  let node: AudioNode = src;
  node.connect(highShelf);
  node = highShelf;
  node.connect(lowShelf);
  node = lowShelf;
  for (const f of formantFilters) {
    node.connect(f);
    node = f;
  }
  node.connect(outGain);
  outGain.connect(offline.destination);

  src.start(0);
  return offline.startRendering();
}
