/**
 * WebSocket connection to the AetherDSP host.
 * Connects on mount, reconnects on disconnect, dispatches snapshots to the store.
 */
import { useEffect, useRef, useCallback } from "react";
import { useEngineStore } from "../store/engineStore";

const WS_URL = "ws://127.0.0.1:9001";
const BACKOFF = [1000, 2000, 4000, 8000, 16000, 30000];

// Pending instrument load — set by useInstrumentEngine.addToCanvas()
// Picked up here when the snapshot arrives with the new node ID.
let _pendingInstrumentLoad: {
  instrumentJson: string;
  autoConnect: boolean;
} | null = null;

export function setPendingInstrumentLoad(
  data: { instrumentJson: string; autoConnect: boolean } | null,
) {
  _pendingInstrumentLoad = data;
}

export function useEngine() {
  const ws = useRef<WebSocket | null>(null);
  const attempt = useRef(0);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const dead = useRef(false);

  const setWsStatus = useEngineStore((s) => s.setWsStatus);
  const setSendIntent = useEngineStore((s) => s.setSendIntent);
  const applySnapshot = useEngineStore((s) => s.applySnapshot);
  const setMidiPorts = useEngineStore((s) => s.setMidiPorts);
  const setConnectedMidiPort = useEngineStore((s) => s.setConnectedMidiPort);
  const setIsRecording = useEngineStore((s) => s.setIsRecording);
  const setRecordingDuration = useEngineStore((s) => s.setRecordingDuration);
  const setScopeFrame = useEngineStore((s) => s.setScopeFrame);

  const connect = useCallback(() => {
    if (dead.current) return;
    setWsStatus("connecting");

    const socket = new WebSocket(WS_URL);
    ws.current = socket;

    const send = (intent: object) => {
      if (socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify(intent));
      }
    };
    setSendIntent(send);

    socket.onopen = () => {
      attempt.current = 0;
      setWsStatus("connected");
    };

    socket.onmessage = (e) => {
      // Handle binary scope frames
      if (e.data instanceof ArrayBuffer) {
        if (e.data.byteLength === 256) {
          setScopeFrame(new Float32Array(e.data));
        }
        // Discard binary frames with wrong byte count silently
        return;
      }

      try {
        const msg = JSON.parse(e.data as string);
        if (msg.type === "snapshot") {
          applySnapshot(msg);

          // If there's a pending instrument load (from "Add to Canvas"),
          // find the newest SamplerNode in the snapshot and load the instrument into it.
          if (_pendingInstrumentLoad) {
            const pending = _pendingInstrumentLoad;
            _pendingInstrumentLoad = null;

            // Find the SamplerNode with the highest ID (most recently added)
            const samplerNodes = (
              msg.nodes as Array<{
                id: number;
                generation: number;
                node_type: string;
              }>
            ).filter((n) => n.node_type === "SamplerNode");

            if (samplerNodes.length > 0) {
              const newest = samplerNodes.reduce((a, b) =>
                a.id > b.id ? a : b,
              );
              const send = useEngineStore.getState().sendIntent;
              if (send) {
                // Load the instrument
                send({
                  type: "load_instrument",
                  node_id: newest.id,
                  generation: newest.generation,
                  instrument_json: pending.instrumentJson,
                });

                // Auto-connect to output node if requested
                if (pending.autoConnect) {
                  const outputNodeId = useEngineStore.getState().outputNodeId;
                  if (outputNodeId) {
                    send({
                      type: "connect",
                      src_id: newest.id,
                      src_gen: newest.generation,
                      dst_id: parseInt(outputNodeId, 10),
                      dst_gen: 0,
                      slot: 0,
                    });
                  }
                }
              }
            }
          }
        } else if (msg.type === "midi_ports") {
          // Response to MidiListPorts intent — update available ports list
          setMidiPorts(msg.ports as string[]);
        } else if (msg.type === "ack" && msg.command === "midi_connect") {
          // Successful MIDI connection — update connected port name from current ports list
          // The port index isn't echoed back, so we mark the last port in the list as connected.
          // TransportBar tracks the pending index itself for a more precise label.
          const ports = useEngineStore.getState().midiPorts;
          if (ports.length > 0) {
            setConnectedMidiPort(ports[ports.length - 1]);
          }
        } else if (msg.type === "recording_started") {
          setIsRecording(true);
          setRecordingDuration(0);
        } else if (msg.type === "recording_stopped") {
          setIsRecording(false);
          setRecordingDuration(msg.duration_secs as number);
        } else if (msg.type === "ack" && msg.command === "load_instrument") {
          // Instrument loaded successfully — pulse the audio activity indicator
          useEngineStore.getState().setAudioActive(true);
          setTimeout(
            () => useEngineStore.getState().setAudioActive(false),
            400,
          );
        } else if (msg.type === "error") {
          console.warn("Host error:", msg.message);
        }
        // ack and error are silently handled
      } catch {
        console.error("WS parse error", e.data);
      }
    };

    socket.onerror = () => setWsStatus("error");

    socket.onclose = () => {
      if (dead.current) return;
      setWsStatus("disconnected");
      const delay = BACKOFF[Math.min(attempt.current, BACKOFF.length - 1)];
      attempt.current++;
      timer.current = setTimeout(connect, delay);
    };
  }, [
    setWsStatus,
    setSendIntent,
    applySnapshot,
    setMidiPorts,
    setConnectedMidiPort,
    setIsRecording,
    setRecordingDuration,
    setScopeFrame,
  ]);

  useEffect(() => {
    dead.current = false;
    connect();
    return () => {
      dead.current = true;
      if (timer.current) clearTimeout(timer.current);
      ws.current?.close();
    };
  }, [connect]);
}
