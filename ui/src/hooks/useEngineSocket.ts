/**
 * useEngineSocket — WebSocket bridge between the UI and aether-host.
 *
 * Connects to ws://127.0.0.1:9001, writes status/data into useEngineStore,
 * and registers the send function as sendIntent so all store actions work.
 */

import { useEffect, useRef, useCallback } from "react";
import { useEngineStore } from "../studio/store/engineStore";

const WS_URL = "ws://127.0.0.1:9001";
const RECONNECT_DELAYS = [1000, 2000, 4000, 8000, 16000, 30000];

export function useEngineSocket() {
  const ws = useRef<WebSocket | null>(null);
  const reconnectAttempt = useRef(0);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const unmounted = useRef(false);

  const setWsStatus = useEngineStore((s) => s.setWsStatus);
  const setAudioActive = useEngineStore((s) => s.setAudioActive);
  const setSendIntent = useEngineStore((s) => s.setSendIntent);
  const applySnapshot = useEngineStore((s) => s.applySnapshot);
  const setMidiPorts = useEngineStore((s) => s.setMidiPorts);
  const setConnectedMidiPort = useEngineStore((s) => s.setConnectedMidiPort);
  const setScopeFrame = useEngineStore((s) => s.setScopeFrame);
  const setCanUndo = useEngineStore((s) => s.setCanUndo);
  const setCanRedo = useEngineStore((s) => s.setCanRedo);
  const setIsRecording = useEngineStore((s) => s.setIsRecording);

  const send = useCallback((msg: object) => {
    if (ws.current?.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(msg));
    }
  }, []);

  const connect = useCallback(() => {
    if (unmounted.current) return;
    setWsStatus("connecting");

    const socket = new WebSocket(WS_URL);
    ws.current = socket;

    socket.onopen = () => {
      reconnectAttempt.current = 0;
      setWsStatus("connected");
      setSendIntent(send);
      // Request current graph state
      socket.send(JSON.stringify({ type: "get_snapshot" }));
    };

    socket.onmessage = (e) => {
      try {
        const msg = JSON.parse(e.data as string) as Record<string, unknown>;

        switch (msg.type) {
          case "snapshot":
            applySnapshot(msg as Parameters<typeof applySnapshot>[0]);
            break;

          case "ack":
            setAudioActive(true);
            setTimeout(() => setAudioActive(false), 150);
            break;

          case "midi_ports":
            setMidiPorts((msg.ports as string[]) ?? []);
            break;

          case "midi_connected":
            setConnectedMidiPort((msg.port as string) ?? null);
            break;

          case "scope_frame": {
            const raw = msg.samples as number[];
            if (raw) setScopeFrame(new Float32Array(raw));
            break;
          }

          case "undo_state":
            setCanUndo(!!msg.can_undo);
            setCanRedo(!!msg.can_redo);
            break;

          case "recording_started":
            setIsRecording(true);
            break;

          case "recording_stopped":
            setIsRecording(false);
            break;

          case "clap_exported":
            alert(
              `CLAP plugin exported!\n\nPath: ${msg.path}\nSize: ${Math.round((msg.size_bytes as number) / 1024)} KB`,
            );
            break;

          case "error":
            console.warn("[aether-host]", msg.message);
            break;

          default:
            break;
        }
      } catch {
        console.error("WS parse error", e.data);
      }
    };

    socket.onerror = () => {
      setWsStatus("error");
    };

    socket.onclose = () => {
      if (unmounted.current) return;
      setWsStatus("disconnected");
      setSendIntent(send); // keep send registered so queued calls don't crash
      const delay =
        RECONNECT_DELAYS[
          Math.min(reconnectAttempt.current, RECONNECT_DELAYS.length - 1)
        ];
      reconnectAttempt.current++;
      reconnectTimer.current = setTimeout(connect, delay);
    };
  }, [
    send,
    setWsStatus,
    setSendIntent,
    applySnapshot,
    setAudioActive,
    setMidiPorts,
    setConnectedMidiPort,
    setScopeFrame,
    setCanUndo,
    setCanRedo,
    setIsRecording,
  ]);

  useEffect(() => {
    unmounted.current = false;
    connect();
    return () => {
      unmounted.current = true;
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current);
      ws.current?.close();
    };
  }, [connect]);
}
