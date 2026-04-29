import type { SourceFamily } from "./types";

/**
 * Generates a 2-second synthetic audio buffer for a given source family
 * using Web Audio API — no WAV files needed.
 */
export async function generateSyntheticSource(
  family: SourceFamily,
  audioContext: AudioContext,
  durationSecs = 2.0,
): Promise<AudioBuffer> {
  const sr = audioContext.sampleRate;
  const frames = Math.floor(sr * durationSecs);
  const offline = new OfflineAudioContext(1, frames, sr);

  switch (family) {
    case "plucked":
      return buildPlucked(offline, durationSecs);
    case "bowed":
      return buildBowed(offline, durationSecs);
    case "blown":
      return buildBlown(offline, durationSecs);
    case "struck":
      return buildStruck(offline, durationSecs);
    case "electronic":
      return buildElectronic(offline, durationSecs);
    default:
      return buildPlucked(offline, durationSecs);
  }
}

// Karplus-Strong plucked string
async function buildPlucked(
  ctx: OfflineAudioContext,
  dur: number,
): Promise<AudioBuffer> {
  const bufSize = Math.floor(ctx.sampleRate / 220); // ~A3
  const noiseBuffer = ctx.createBuffer(1, bufSize, ctx.sampleRate);
  const data = noiseBuffer.getChannelData(0);
  for (let i = 0; i < bufSize; i++) data[i] = Math.random() * 2 - 1;

  const source = ctx.createBufferSource();
  source.buffer = noiseBuffer;
  source.loop = true;

  const delay = ctx.createDelay(1);
  delay.delayTime.value = 1 / 220;

  const filter = ctx.createBiquadFilter();
  filter.type = "lowpass";
  filter.frequency.value = 3000;

  const gain = ctx.createGain();
  gain.gain.setValueAtTime(1, 0);
  gain.gain.exponentialRampToValueAtTime(0.001, dur);

  source.connect(filter);
  filter.connect(delay);
  delay.connect(gain);
  gain.connect(ctx.destination);
  source.start(0);
  source.stop(dur);

  return ctx.startRendering();
}

// Bowed string — sawtooth + vibrato
async function buildBowed(
  ctx: OfflineAudioContext,
  dur: number,
): Promise<AudioBuffer> {
  const osc = ctx.createOscillator();
  osc.type = "sawtooth";
  osc.frequency.value = 220;

  const lfo = ctx.createOscillator();
  lfo.frequency.value = 5.5;
  const lfoGain = ctx.createGain();
  lfoGain.gain.value = 4;
  lfo.connect(lfoGain);
  lfoGain.connect(osc.frequency);

  const filter = ctx.createBiquadFilter();
  filter.type = "lowpass";
  filter.frequency.value = 2000;
  filter.Q.value = 1.5;

  const gain = ctx.createGain();
  gain.gain.setValueAtTime(0, 0);
  gain.gain.linearRampToValueAtTime(0.7, 0.15);
  gain.gain.setValueAtTime(0.7, dur - 0.3);
  gain.gain.linearRampToValueAtTime(0, dur);

  osc.connect(filter);
  filter.connect(gain);
  gain.connect(ctx.destination);
  osc.start(0);
  lfo.start(0);
  osc.stop(dur);
  lfo.stop(dur);

  return ctx.startRendering();
}

// Blown — noise + sine blend
async function buildBlown(
  ctx: OfflineAudioContext,
  dur: number,
): Promise<AudioBuffer> {
  const noiseFrames = Math.floor(ctx.sampleRate * dur);
  const noiseBuf = ctx.createBuffer(1, noiseFrames, ctx.sampleRate);
  const nd = noiseBuf.getChannelData(0);
  for (let i = 0; i < noiseFrames; i++) nd[i] = (Math.random() * 2 - 1) * 0.15;

  const noiseSource = ctx.createBufferSource();
  noiseSource.buffer = noiseBuf;

  const bandpass = ctx.createBiquadFilter();
  bandpass.type = "bandpass";
  bandpass.frequency.value = 880;
  bandpass.Q.value = 8;

  const sine = ctx.createOscillator();
  sine.type = "sine";
  sine.frequency.value = 440;

  const gain = ctx.createGain();
  gain.gain.setValueAtTime(0, 0);
  gain.gain.linearRampToValueAtTime(0.6, 0.2);
  gain.gain.setValueAtTime(0.6, dur - 0.2);
  gain.gain.linearRampToValueAtTime(0, dur);

  noiseSource.connect(bandpass);
  bandpass.connect(gain);
  sine.connect(gain);
  gain.connect(ctx.destination);

  noiseSource.start(0);
  sine.start(0);
  noiseSource.stop(dur);
  sine.stop(dur);

  return ctx.startRendering();
}

// Struck — noise burst with exponential decay
async function buildStruck(
  ctx: OfflineAudioContext,
  dur: number,
): Promise<AudioBuffer> {
  const frames = Math.floor(ctx.sampleRate * dur);
  const buf = ctx.createBuffer(1, frames, ctx.sampleRate);
  const d = buf.getChannelData(0);
  for (let i = 0; i < frames; i++) d[i] = Math.random() * 2 - 1;

  const source = ctx.createBufferSource();
  source.buffer = buf;

  const filter = ctx.createBiquadFilter();
  filter.type = "bandpass";
  filter.frequency.value = 200;
  filter.Q.value = 3;

  const gain = ctx.createGain();
  gain.gain.setValueAtTime(1, 0);
  gain.gain.exponentialRampToValueAtTime(0.001, Math.min(0.8, dur));

  source.connect(filter);
  filter.connect(gain);
  gain.connect(ctx.destination);
  source.start(0);
  source.stop(dur);

  return ctx.startRendering();
}

// Electronic — two detuned sines
async function buildElectronic(
  ctx: OfflineAudioContext,
  dur: number,
): Promise<AudioBuffer> {
  const osc1 = ctx.createOscillator();
  osc1.type = "sine";
  osc1.frequency.value = 220;

  const osc2 = ctx.createOscillator();
  osc2.type = "sine";
  osc2.frequency.value = 220 * Math.pow(2, 3 / 1200); // 3 cents sharp

  const gain = ctx.createGain();
  gain.gain.setValueAtTime(0, 0);
  gain.gain.linearRampToValueAtTime(0.5, 0.05);
  gain.gain.setValueAtTime(0.5, dur - 0.1);
  gain.gain.linearRampToValueAtTime(0, dur);

  osc1.connect(gain);
  osc2.connect(gain);
  gain.connect(ctx.destination);
  osc1.start(0);
  osc2.start(0);
  osc1.stop(dur);
  osc2.stop(dur);

  return ctx.startRendering();
}
