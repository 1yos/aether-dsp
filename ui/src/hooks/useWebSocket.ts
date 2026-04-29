import { useEffect, useRef, useCallback } from "react";
import { useGraphStore } from "./useGraphStore";

const WS_URL = "ws://127.0.0.1:9001";
const RECONNECT_DELAYS = [1000, 2000, 4000, 8000, 16000, 30000];

export function useWebSocket() {
  const ws = useRef<WebSocket | null>(null);
  const reconnectAttempt = useRef(0);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const unmounted = useRef(false);

  const loadSnapshot = useGraphStore((s) => s.loadSnapshot);
  const setWsStatus = useGraphStore((s) => s.setWsStatus);
  const setAudioActive = useGraphStore((s) => s.setAudioActive);

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
      send({ type: "get_snapshot" });
    };

    socket.onmessage = (e) => {
      try {
        const msg = JSON.parse(e.data as string);
        if (msg.type === "snapshot") {
          loadSnapshot({ nodes: msg.nodes, edges: msg.edges });
          // Pulse audio activity indicator
          setAudioActive(true);
          setTimeout(() => setAudioActive(false), 300);
        } else if (msg.type === "ack") {
          // Pulse on param ack — audio is responding
          setAudioActive(true);
          setTimeout(() => setAudioActive(false), 150);
        } else if (msg.type === "clap_exported") {
          // Task 20.6: CLAP export succeeded — show path and file size.
          const sizeKb = Math.round((msg.size_bytes as number) / 1024);
          alert(
            `CLAP plugin exported successfully!\n\nPath: ${msg.path}\nSize: ${sizeKb} KB`,
          );
        } else if (msg.type === "error") {
          // Task 20.6: surface export errors (and any other host errors) to the user.
          console.warn("WS host error:", msg.message);
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
      // Exponential backoff reconnect
      const delay =
        RECONNECT_DELAYS[
          Math.min(reconnectAttempt.current, RECONNECT_DELAYS.length - 1)
        ];
      reconnectAttempt.current++;
      reconnectTimer.current = setTimeout(connect, delay);
    };
  }, [send, loadSnapshot, setWsStatus, setAudioActive]);

  useEffect(() => {
    unmounted.current = false;
    connect();
    return () => {
      unmounted.current = true;
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current);
      ws.current?.close();
    };
  }, [connect]);

  return { send };
}
