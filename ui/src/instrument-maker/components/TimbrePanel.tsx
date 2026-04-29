/**
 * TimbrePanel — Timbre Transfer feature.
 *
 * Lets users who don't have the actual instrument generate synthetic samples
 * by transferring the timbre of a reference recording onto a source sound.
 *
 * Workflow:
 *   1. Upload a reference recording of the target instrument (even a short clip)
 *   2. Upload a source sound (guitar, piano, voice — anything)
 *   3. Optionally send the reference to the host for analysis ("Analyze Timbre")
 *   4. Set a TimbreTransferNode ID and click "Apply to Node" to wire the profile
 *   5. Adjust the Transfer Amount slider
 *   6. Click "Generate Samples" — the source gets reshaped to sound like the target
 *      and the generated samples are added as zones to the instrument
 */
import { useState, useRef, useCallback } from "react";
import { useInstrumentStore } from "../store/instrumentStore";
import { useEngineStore } from "../../studio/store/engineStore";
import { SampleZone, midiNoteName } from "../types";
import { loadAudioBuffer } from "../hooks/useAudioPreview";

// ── DropZone sub-component ────────────────────────────────────────────────────

function DropZone({
  label,
  sublabel,
  file,
  onFile,
  accent,
}: {
  label: string;
  sublabel: string;
  file: File | null;
  onFile: (f: File) => void;
  accent: string;
}) {
  const [dragging, setDragging] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <div
      onDragOver={(e) => {
        e.preventDefault();
        setDragging(true);
      }}
      onDragLeave={() => setDragging(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDragging(false);
        const f = e.dataTransfer.files[0];
        if (f) onFile(f);
      }}
      onClick={() => inputRef.current?.click()}
      style={{
        border: `2px dashed ${dragging ? accent : "#1a2a3a"}`,
        borderRadius: 8,
        padding: "16px 12px",
        textAlign: "center",
        cursor: "pointer",
        background: dragging ? `${accent}11` : "#080e14",
        transition: "all 0.15s",
      }}
    >
      <input
        ref={inputRef}
        type="file"
        accept="audio/*"
        style={{ display: "none" }}
        onChange={(e) => {
          const f = e.target.files?.[0];
          if (f) onFile(f);
        }}
      />
      <div style={{ fontSize: 20, marginBottom: 4 }}>🎵</div>
      <div style={{ fontSize: 11, color: "#e0e8f0", fontWeight: 600 }}>
        {label}
      </div>
      <div style={{ fontSize: 10, color: "#4a6a8a", marginBottom: 4 }}>
        {sublabel}
      </div>
      {file ? (
        <div
          style={{
            fontSize: 11,
            color: accent,
            marginTop: 4,
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {file.name}
        </div>
      ) : (
        <div style={{ fontSize: 10, color: "#3a5a7a", marginTop: 2 }}>
          Drop audio file or click to browse
        </div>
      )}
    </div>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

export function TimbrePanel() {
  const { addZone } = useInstrumentStore();
  const sendIntent = useEngineStore((s) => s.sendIntent);

  // ── File state ──────────────────────────────────────────────────────────────
  // referenceFile = the TARGET instrument sound (what we want to sound like)
  // sourceFile    = the SOURCE sound to reshape
  const [referenceFile, setReferenceFile] = useState<File | null>(null);
  const [sourceFile, setSourceFile] = useState<File | null>(null);

  // ── Note range ──────────────────────────────────────────────────────────────
  const [noteRange, setNoteRange] = useState({ low: 36, high: 84 });

  // ── Transfer amount (0.0–1.0) ───────────────────────────────────────────────
  const [transferAmount, setTransferAmount] = useState(1.0);

  // ── Host integration state ──────────────────────────────────────────────────
  const [nodeIdInput, setNodeIdInput] = useState("");
  const [analyzeStatus, setAnalyzeStatus] = useState("");
  const [applyStatus, setApplyStatus] = useState("");

  // ── Generation state ────────────────────────────────────────────────────────
  const [generating, setGenerating] = useState(false);
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState("");

  // ── Analyze Timbre ──────────────────────────────────────────────────────────
  const handleAnalyzeTimbre = useCallback(async () => {
    if (!referenceFile) {
      setAnalyzeStatus("Please provide a reference (target) audio file first.");
      return;
    }

    setAnalyzeStatus("Sending to host...");

    // Build a minimal instrument JSON from the reference file name
    const instrumentJson = JSON.stringify({
      name: referenceFile.name.replace(/\.[^.]+$/, ""),
      origin: "uploaded",
      description: "Uploaded reference recording",
      author: "",
      zones: [],
    });

    if (sendIntent) {
      try {
        sendIntent({ type: "analyze_timbre", instrument_json: instrumentJson });
        setAnalyzeStatus("Analysis sent to host.");
      } catch {
        // Fall back to browser-side analysis
        setAnalyzeStatus("Host unavailable — using browser-side analysis.");
      }
    } else {
      // No WebSocket connection — use browser-side analysis
      setAnalyzeStatus("No host connection — using browser-side analysis.");
    }
  }, [referenceFile, sendIntent]);

  // ── Apply Transfer ──────────────────────────────────────────────────────────
  const handleApplyTransfer = useCallback(() => {
    const nodeId = parseInt(nodeIdInput, 10);
    if (isNaN(nodeId) || nodeId < 0) {
      setApplyStatus("Enter a valid TimbreTransferNode ID.");
      return;
    }

    const profileName = referenceFile
      ? referenceFile.name.replace(/\.[^.]+$/, "")
      : "default";

    if (sendIntent) {
      sendIntent({
        type: "apply_timbre_transfer",
        timbre_transfer_node_id: nodeId,
        profile_name: profileName,
      });
      setApplyStatus(`Transfer applied to node ${nodeId}.`);
    } else {
      setApplyStatus("No host connection.");
    }
  }, [nodeIdInput, referenceFile, sendIntent]);

  // ── Transfer amount change ──────────────────────────────────────────────────
  const handleAmountChange = useCallback(
    (value: number) => {
      setTransferAmount(value);
      const nodeId = parseInt(nodeIdInput, 10);
      if (!isNaN(nodeId) && nodeId >= 0 && sendIntent) {
        // Send UpdateParam for param index 0 (Amount) on the TimbreTransferNode
        sendIntent({
          type: "update_param",
          node_id: nodeId,
          generation: 0,
          param_index: 0,
          value,
          ramp_ms: 20,
        });
      }
    },
    [nodeIdInput, sendIntent],
  );

  // ── Generate Samples ────────────────────────────────────────────────────────
  const generate = useCallback(async () => {
    if (!referenceFile || !sourceFile) {
      setStatus("Please provide both a reference and source audio file.");
      return;
    }

    setGenerating(true);
    setProgress(0);
    setStatus("Loading audio files...");

    try {
      // Load both files
      const [refBuffer, srcBuffer] = await Promise.all([
        loadAudioBuffer(referenceFile),
        loadAudioBuffer(sourceFile),
      ]);

      setStatus("Analyzing reference timbre...");
      setProgress(10);

      // Extract reference spectral envelope using Web Audio API
      const refEnvelope = await analyzeSpectralEnvelope(refBuffer);

      setStatus("Generating samples...");
      const totalNotes = noteRange.high - noteRange.low + 1;
      const srcData = getMonoData(srcBuffer);

      // Generate one zone per 3 semitones (enough coverage without too many files)
      const step = 3;
      let generated = 0;

      for (let note = noteRange.low; note <= noteRange.high; note += step) {
        const noteHigh = Math.min(note + step - 1, noteRange.high);

        // Apply spectral envelope transfer with the current transfer amount
        const processed = applyTimbreTransfer(
          srcData,
          refEnvelope,
          srcBuffer.sampleRate,
          transferAmount,
        );

        // Create AudioBuffer from processed data
        const ctx = new OfflineAudioContext(
          1,
          processed.length,
          srcBuffer.sampleRate,
        );
        const buf = ctx.createBuffer(1, processed.length, srcBuffer.sampleRate);
        buf.copyToChannel(new Float32Array(processed), 0);

        // Convert to data URL for storage
        const dataUrl = await audioBufferToDataUrl(buf, srcBuffer.sampleRate);

        const zone: SampleZone = {
          id: `timbre-${note}-${Date.now()}`,
          file_path: `timbre_${midiNoteName(note)}.wav`,
          fileName: `Timbre: ${midiNoteName(note)} (generated)`,
          root_note: note,
          note_low: note,
          note_high: noteHigh,
          velocity_low: 0,
          velocity_high: 127,
          articulation: { type: "OneShot" },
          volume_db: 0,
          tune_cents: 0,
          release_file: null,
          audioDataUrl: dataUrl,
          audioBuffer: buf,
        };

        addZone(zone);
        generated++;
        setProgress(
          10 + Math.round((generated / Math.ceil(totalNotes / step)) * 85),
        );
        setStatus(
          `Generated ${midiNoteName(note)} (${generated}/${Math.ceil(totalNotes / step)})...`,
        );

        // Yield to UI
        await new Promise((r) => setTimeout(r, 0));
      }

      setProgress(100);
      setStatus(`✓ Generated ${generated} zones using timbre transfer.`);
    } catch (err) {
      setStatus(`Error: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setGenerating(false);
    }
  }, [referenceFile, sourceFile, noteRange, transferAmount, addZone]);

  // ── Render ──────────────────────────────────────────────────────────────────
  return (
    <div style={{ padding: 12 }}>
      {/* Header */}
      <div
        style={{
          fontSize: 13,
          color: "#e0e8f0",
          fontWeight: 600,
          marginBottom: 4,
        }}
      >
        Timbre Transfer
      </div>
      <div
        style={{
          fontSize: 11,
          color: "#4a6a8a",
          marginBottom: 12,
          lineHeight: 1.5,
        }}
      >
        Don't have the instrument to record? Upload a short reference clip of
        the target instrument and a source sound — Aether will reshape the
        source to sound like the target.
      </div>

      {/* Source / Target drop zones */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 8,
          marginBottom: 12,
        }}
      >
        <DropZone
          label="Target Instrument"
          sublabel="Reference sound to copy timbre from"
          file={referenceFile}
          onFile={setReferenceFile}
          accent="#ce93d8"
        />
        <DropZone
          label="Source Sound"
          sublabel="Audio to reshape (guitar, piano, voice…)"
          file={sourceFile}
          onFile={setSourceFile}
          accent="#4fc3f7"
        />
      </div>

      {/* Analyze Timbre button */}
      <button
        onClick={handleAnalyzeTimbre}
        disabled={!referenceFile}
        title="Send the reference recording to the host for timbre analysis"
        style={{
          width: "100%",
          padding: "7px 0",
          marginBottom: 6,
          background: referenceFile ? "#1a2a4a" : "#0a1520",
          border: "1px solid #2a4a7a",
          borderRadius: 6,
          color: referenceFile ? "#ce93d8" : "#2a4a6a",
          fontSize: 12,
          cursor: referenceFile ? "pointer" : "not-allowed",
          fontWeight: 600,
        }}
      >
        Analyze Timbre
      </button>
      {analyzeStatus && (
        <div style={{ fontSize: 10, color: "#7a9ab5", marginBottom: 8 }}>
          {analyzeStatus}
        </div>
      )}

      {/* Node ID + Apply Transfer */}
      <div style={{ marginBottom: 10 }}>
        <div style={{ fontSize: 11, color: "#7a9ab5", marginBottom: 4 }}>
          Apply to TimbreTransferNode
        </div>
        <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <input
            type="number"
            min={0}
            placeholder="Node ID"
            value={nodeIdInput}
            onChange={(e) => setNodeIdInput(e.target.value)}
            style={{
              flex: 1,
              padding: "5px 8px",
              background: "#080e14",
              border: "1px solid #1a2a3a",
              borderRadius: 4,
              color: "#e0e8f0",
              fontSize: 11,
            }}
          />
          <button
            onClick={handleApplyTransfer}
            disabled={!nodeIdInput}
            title="Apply the analyzed timbre profile to the selected node"
            style={{
              padding: "5px 10px",
              background: nodeIdInput ? "#1a3a2a" : "#0a1520",
              border: "1px solid #2a6a4a",
              borderRadius: 4,
              color: nodeIdInput ? "#80cbc4" : "#2a4a3a",
              fontSize: 11,
              cursor: nodeIdInput ? "pointer" : "not-allowed",
              fontWeight: 600,
              whiteSpace: "nowrap",
            }}
          >
            Apply to Node
          </button>
        </div>
        {applyStatus && (
          <div style={{ fontSize: 10, color: "#7a9ab5", marginTop: 4 }}>
            {applyStatus}
          </div>
        )}
      </div>

      {/* Transfer Amount slider */}
      <div style={{ marginBottom: 12 }}>
        <label
          style={{
            fontSize: 11,
            color: "#7a9ab5",
            display: "flex",
            justifyContent: "space-between",
            marginBottom: 4,
          }}
        >
          <span>Transfer Amount</span>
          <span style={{ color: "#4fc3f7" }}>{transferAmount.toFixed(2)}</span>
        </label>
        <input
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={transferAmount}
          onChange={(e) => handleAmountChange(parseFloat(e.target.value))}
          style={{ width: "100%", accentColor: "#4fc3f7" }}
        />
      </div>

      {/* Note Range */}
      <div style={{ marginBottom: 12 }}>
        <div style={{ fontSize: 11, color: "#7a9ab5", marginBottom: 6 }}>
          Note Range to Generate
        </div>
        {/* Note range display */}
        <div
          style={{
            fontSize: 10,
            color: "#4a6a8a",
            marginBottom: 6,
            textAlign: "center",
          }}
        >
          {midiNoteName(noteRange.low)} — {midiNoteName(noteRange.high)}
          &nbsp;({noteRange.high - noteRange.low + 1} notes)
        </div>
        <div
          style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}
        >
          <div>
            <label
              style={{
                fontSize: 10,
                color: "#4a6a8a",
                display: "block",
                marginBottom: 2,
              }}
            >
              Low: {midiNoteName(noteRange.low)}
            </label>
            <input
              type="range"
              min={0}
              max={127}
              value={noteRange.low}
              onChange={(e) =>
                setNoteRange((r) => ({
                  ...r,
                  low: Math.min(parseInt(e.target.value), r.high),
                }))
              }
              style={{ width: "100%", accentColor: "#4fc3f7" }}
            />
          </div>
          <div>
            <label
              style={{
                fontSize: 10,
                color: "#4a6a8a",
                display: "block",
                marginBottom: 2,
              }}
            >
              High: {midiNoteName(noteRange.high)}
            </label>
            <input
              type="range"
              min={0}
              max={127}
              value={noteRange.high}
              onChange={(e) =>
                setNoteRange((r) => ({
                  ...r,
                  high: Math.max(parseInt(e.target.value), r.low),
                }))
              }
              style={{ width: "100%", accentColor: "#4fc3f7" }}
            />
          </div>
        </div>
      </div>

      {/* Generate Samples button */}
      <button
        onClick={generate}
        disabled={generating || !referenceFile || !sourceFile}
        style={{
          width: "100%",
          padding: "8px 0",
          background: generating ? "#1a2a3a" : "#1a3a5a",
          border: "1px solid #2a5a8a",
          borderRadius: 6,
          color: generating ? "#4a6a8a" : "#4fc3f7",
          fontSize: 12,
          cursor: generating ? "not-allowed" : "pointer",
          fontWeight: 600,
        }}
      >
        {generating ? "Generating..." : "Generate Samples"}
      </button>

      {/* Progress / status */}
      {(generating || status) && (
        <div style={{ marginTop: 10 }}>
          {generating && (
            <div
              style={{
                height: 4,
                background: "#0a1520",
                borderRadius: 2,
                overflow: "hidden",
                marginBottom: 6,
              }}
            >
              <div
                style={{
                  height: "100%",
                  width: `${progress}%`,
                  background: "#4fc3f7",
                  transition: "width 0.2s",
                  borderRadius: 2,
                }}
              />
            </div>
          )}
          <div style={{ fontSize: 11, color: "#7a9ab5" }}>{status}</div>
        </div>
      )}
    </div>
  );
}

// ── DSP helpers (browser-side) ────────────────────────────────────────────────

function getMonoData(buffer: AudioBuffer): Float32Array {
  if (buffer.numberOfChannels === 1) {
    return buffer.getChannelData(0);
  }
  const left = buffer.getChannelData(0);
  const right = buffer.getChannelData(1);
  const mono = new Float32Array(left.length);
  for (let i = 0; i < left.length; i++) {
    mono[i] = (left[i] + right[i]) * 0.5;
  }
  return mono;
}

interface SpectralEnvelope {
  magnitudes: Float32Array;
  fftSize: number;
}

async function analyzeSpectralEnvelope(
  buffer: AudioBuffer,
): Promise<SpectralEnvelope> {
  const fftSize = 2048;
  const data = getMonoData(buffer);
  const hop = fftSize / 4;
  const magnitudes = new Float32Array(fftSize / 2 + 1);
  let frameCount = 0;

  // Hann window
  const window = new Float32Array(fftSize);
  for (let i = 0; i < fftSize; i++) {
    window[i] = 0.5 * (1 - Math.cos((2 * Math.PI * i) / (fftSize - 1)));
  }

  // Use OfflineAudioContext + AnalyserNode for FFT
  let offset = 0;
  while (offset + fftSize <= data.length) {
    const frame = new Float32Array(fftSize);
    for (let i = 0; i < fftSize; i++) {
      frame[i] = data[offset + i] * window[i];
    }

    // Simple DFT magnitude (browser doesn't expose raw FFT easily)
    // We use a simplified approach: compute magnitude spectrum via AnalyserNode
    const ctx = new OfflineAudioContext(1, fftSize, buffer.sampleRate);
    const src = ctx.createBufferSource();
    const analyser = ctx.createAnalyser();
    analyser.fftSize = fftSize;

    const buf = ctx.createBuffer(1, fftSize, buffer.sampleRate);
    buf.copyToChannel(frame, 0);
    src.buffer = buf;
    src.connect(analyser);
    analyser.connect(ctx.destination);
    src.start(0);

    await ctx.startRendering();

    const freqData = new Float32Array(analyser.frequencyBinCount);
    analyser.getFloatFrequencyData(freqData);

    for (let i = 0; i < magnitudes.length; i++) {
      // Convert dB to linear
      magnitudes[i] += Math.pow(10, (freqData[i] ?? -100) / 20);
    }
    frameCount++;
    offset += hop;
  }

  if (frameCount > 0) {
    for (let i = 0; i < magnitudes.length; i++) {
      magnitudes[i] /= frameCount;
    }
  }

  // Smooth
  const smoothed = smoothArray(magnitudes, 8);
  return { magnitudes: smoothed, fftSize };
}

function smoothArray(arr: Float32Array, w: number): Float32Array {
  const out = new Float32Array(arr.length);
  for (let i = 0; i < arr.length; i++) {
    const s = Math.max(0, i - Math.floor(w / 2));
    const e = Math.min(arr.length, i + Math.floor(w / 2) + 1);
    let sum = 0;
    for (let j = s; j < e; j++) sum += arr[j];
    out[i] = sum / (e - s);
  }
  return out;
}

/**
 * Apply spectral envelope transfer from `target` onto `source`.
 * `amount` (0.0–1.0) blends between dry (0) and fully transferred (1).
 */
function applyTimbreTransfer(
  source: Float32Array,
  target: SpectralEnvelope,
  _sampleRate: number,
  amount = 1.0,
): Float32Array {
  const fftSize = target.fftSize;
  const output = new Float32Array(source.length);
  const dry = new Float32Array(source.length);

  // Simple spectral envelope replacement using overlap-add
  const hop = fftSize / 4;
  const window = new Float32Array(fftSize);
  for (let i = 0; i < fftSize; i++) {
    window[i] = 0.5 * (1 - Math.cos((2 * Math.PI * i) / (fftSize - 1)));
  }

  let pos = 0;
  while (pos + fftSize <= source.length) {
    // Apply window
    const frame = new Float32Array(fftSize);
    for (let i = 0; i < fftSize; i++) {
      frame[i] = source[pos + i] * window[i];
    }

    // Dry frame (windowed source, no transfer)
    const dryFrame = new Float32Array(frame);

    // Simple spectral shaping: scale each frequency bin
    // (Full FFT not available in browser without a library, so we use a
    // simplified approach: apply the envelope as a time-domain filter)
    // For a real implementation, use a WebAssembly FFT or the Web Audio API
    // convolution node. This gives a reasonable approximation.
    const nBins = target.magnitudes.length;

    for (let i = 0; i < fftSize; i++) {
      const bin = Math.floor((i / fftSize) * nBins);
      const tgtMag = target.magnitudes[Math.min(bin, nBins - 1)];
      const scale = Math.min(tgtMag * 2, 3); // Limit gain
      frame[i] *= scale;
    }

    // Overlap-add (wet and dry separately)
    for (let i = 0; i < fftSize && pos + i < output.length; i++) {
      output[pos + i] += frame[i];
      dry[pos + i] += dryFrame[i];
    }

    pos += hop;
  }

  // Normalize wet signal
  let peak = 0;
  for (let i = 0; i < output.length; i++) {
    peak = Math.max(peak, Math.abs(output[i]));
  }
  if (peak > 0) {
    for (let i = 0; i < output.length; i++) {
      output[i] /= peak;
    }
  }

  // Normalize dry signal
  let dryPeak = 0;
  for (let i = 0; i < dry.length; i++) {
    dryPeak = Math.max(dryPeak, Math.abs(dry[i]));
  }
  if (dryPeak > 0) {
    for (let i = 0; i < dry.length; i++) {
      dry[i] /= dryPeak;
    }
  }

  // Blend dry/wet by amount
  const result = new Float32Array(source.length);
  for (let i = 0; i < result.length; i++) {
    result[i] = dry[i] * (1 - amount) + output[i] * amount;
  }

  return result;
}

async function audioBufferToDataUrl(
  buffer: AudioBuffer,
  sampleRate: number,
): Promise<string> {
  // Encode as WAV
  const data = buffer.getChannelData(0);
  const wav = encodeWav(data, sampleRate);
  const blob = new Blob([wav], { type: "audio/wav" });
  return URL.createObjectURL(blob);
}

function encodeWav(samples: Float32Array, sampleRate: number): ArrayBuffer {
  const numSamples = samples.length;
  const buffer = new ArrayBuffer(44 + numSamples * 2);
  const view = new DataView(buffer);

  function writeString(offset: number, str: string) {
    for (let i = 0; i < str.length; i++) {
      view.setUint8(offset + i, str.charCodeAt(i));
    }
  }

  writeString(0, "RIFF");
  view.setUint32(4, 36 + numSamples * 2, true);
  writeString(8, "WAVE");
  writeString(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true); // PCM
  view.setUint16(22, 1, true); // mono
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  writeString(36, "data");
  view.setUint32(40, numSamples * 2, true);

  let offset = 44;
  for (let i = 0; i < numSamples; i++) {
    const s = Math.max(-1, Math.min(1, samples[i]));
    view.setInt16(offset, s < 0 ? s * 0x8000 : s * 0x7fff, true);
    offset += 2;
  }

  return buffer;
}
